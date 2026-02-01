use chrono::DateTime;
use roxmltree::Document;

use crate::errors::{FileAttributeKind, ParseNzbError};
use crate::subject;
use crate::{File, Meta, Segment};

enum MetaType {
    Title,
    Password,
    Tag,
    Category,
}

impl MetaType {
    fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("title") {
            Some(MetaType::Title)
        } else if s.eq_ignore_ascii_case("password") {
            Some(MetaType::Password)
        } else if s.eq_ignore_ascii_case("tag") {
            Some(MetaType::Tag)
        } else if s.eq_ignore_ascii_case("category") {
            Some(MetaType::Category)
        } else {
            None
        }
    }
}

/// Parse the `<meta>...</meta>` fields present in an NZB.
///
/// ```xml
/// <?xml version="1.0" encoding="iso-8859-1" ?>
/// <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
/// <nzb xmlns="http://www.newzbin.com/DTD/2003/nzb">
///     <head>
///         <meta type="title">Your File!</meta>
///         <meta type="password">secret</meta>
///         <meta type="tag">HD</meta>
///         <meta type="category">TV</meta>
///     </head>
/// </nzb>
/// ```
pub(crate) fn parse_metadata(nzb: &Document) -> Meta {
    let mut title: Option<String> = None;
    let mut passwords: Vec<String> = Vec::new();
    let mut tags: Vec<String> = Vec::new();
    let mut category: Option<String> = None;

    for meta in nzb.descendants().filter(|n| n.has_tag_name("meta")) {
        if let Some(typ) = meta.attribute("type").and_then(MetaType::parse) {
            match typ {
                MetaType::Title => {
                    title = title.or(meta.text().map(String::from));
                }
                MetaType::Password => {
                    if let Some(text) = meta.text().map(String::from) {
                        passwords.push(text);
                    }
                }
                MetaType::Tag => {
                    if let Some(text) = meta.text().map(String::from) {
                        tags.push(text);
                    }
                }
                MetaType::Category => {
                    category = category.or(meta.text().map(String::from));
                }
            }
        }
    }

    Meta {
        title,
        passwords,
        tags,
        category,
    }
}

/// Parses the `<file>...</file>` fields present in an NZB.
///
/// ```xml
/// <?xml version="1.0" encoding="iso-8859-1" ?>
/// <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
/// <nzb xmlns="http://www.newzbin.com/DTD/2003/nzb">
///     <file poster="Joe Bloggs &lt;bloggs@nowhere.example&gt;" date="1071674882" subject="Here's your file!  abc-mr2a.r01 (1/2)">
///         <groups>[...]</groups>
///         <segments>[...]</segments>
///     </file>
/// </nzb>
/// ```
pub(crate) fn parse_files(nzb: &Document) -> Result<Vec<File>, ParseNzbError> {
    let mut files = Vec::new();
    let file_nodes = nzb.descendants().filter(|n| n.has_tag_name("file"));

    for node in file_nodes {
        let poster = node
            .attribute("poster")
            .ok_or(ParseNzbError::FileAttribute {
                attribute: FileAttributeKind::Poster,
            })?
            .to_string();
        let posted_at = node
            .attribute("date")
            .and_then(|d| d.parse::<i64>().ok())
            .and_then(|d| DateTime::from_timestamp(d, 0))
            .ok_or(ParseNzbError::FileAttribute {
                attribute: FileAttributeKind::Date,
            })?;
        let subject = node
            .attribute("subject")
            .ok_or(ParseNzbError::FileAttribute {
                attribute: FileAttributeKind::Subject,
            })?
            .to_string();

        let mut groups = Vec::new();
        let mut segments = Vec::new();

        if let Some(children) = node.descendants().find(|n| n.has_tag_name("groups")) {
            groups.extend(
                children
                    .descendants()
                    .filter(|n| n.has_tag_name("group"))
                    .filter_map(|group| group.text().filter(|text| !text.is_empty()).map(String::from)),
            );
        }

        // There must be at least one group.
        if groups.is_empty() {
            return Err(ParseNzbError::GroupsElement);
        }

        if let Some(children) = node.descendants().find(|n| n.has_tag_name("segments")) {
            segments.extend(
                children
                    .descendants()
                    .filter(|n| n.has_tag_name("segment"))
                    .filter_map(|segment| {
                        let size = segment.attribute("bytes")?.parse::<u32>().ok()?;
                        let number = segment.attribute("number")?.parse::<u32>().ok()?;
                        let message_id = segment.text()?;
                        Some(Segment::new(size, number, message_id))
                    }),
            );
        }

        // There must be at least one segment.
        if segments.is_empty() {
            return Err(ParseNzbError::SegmentsElement);
        }

        // sort for consistency
        groups.sort();
        segments.sort_by_key(|f| f.number);

        files.push(File {
            poster,
            posted_at,
            subject,
            groups,
            segments,
        });
    }

    // There must be at least one file.
    if files.is_empty() {
        return Err(ParseNzbError::FileElement);
    }

    // There must be at least one non-`.par2` file.
    if files.iter().all(File::is_par2) {
        return Err(ParseNzbError::OnlyPar2Files);
    }

    files.sort_by(|a, b| {
        let ka = subject::file_number(&a.subject);
        let kb = subject::file_number(&b.subject);
        ka.cmp(&kb).then_with(|| a.subject.cmp(&b.subject))
    });

    Ok(files)
}
