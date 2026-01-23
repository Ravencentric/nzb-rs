use std::sync::LazyLock;

use chrono::DateTime;
use regex::Regex;
use roxmltree::Document;

use crate::errors::{FileAttributeKind, ParseNzbError};
use crate::{File, Meta, Segment, subparsers};

pub(crate) fn sanitize_xml(xml: &str) -> &str {
    // roxmltree doesn't support XML declarations or DOCTYPEs, so we need to remove them.
    static XML_HEADING_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?i)<\?xml\s+version.*?\?>").unwrap());
    static XML_DOCTYPE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?i)<!DOCTYPE.*?>").unwrap());

    let mut content = xml.trim();
    if let Some(found) = XML_HEADING_RE.find(content) {
        content = &content[found.end()..];
        content = content.trim_start();
    }
    if let Some(found) = XML_DOCTYPE_RE.find(content) {
        content = &content[found.end()..];
        content = content.trim_start();
    }
    content
}

/// Parse the `<meta>...</meta>` field present in an NZB.
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
        if let Some(typ) = meta.attribute("type").map(str::to_lowercase).as_deref() {
            match typ {
                "title" => {
                    title = title.or(meta.text().map(String::from));
                }
                "password" => {
                    if let Some(text) = meta.text().map(String::from)
                        && !passwords.contains(&text)
                    {
                        passwords.push(text);
                    }
                }
                "tag" => {
                    if let Some(text) = meta.text().map(String::from)
                        && !tags.contains(&text)
                    {
                        tags.push(text);
                    }
                }
                "category" => {
                    category = category.or(meta.text().map(String::from));
                }
                _ => {} // Do not error on unknown meta types because the spec specifies that clients should ignore them.
            }
        }
    }

    Meta::new(title, passwords, tags, category)
}

/// Parses the `<file>...</file>` field present in an NZB.
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
        let ka = subparsers::file_number(&a.subject);
        let kb = subparsers::file_number(&b.subject);
        ka.cmp(&kb).then_with(|| a.subject.cmp(&b.subject))
    });

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_xml() {
        let original = r#"
        <?xml version="1.0" encoding="iso-8859-1" ?>
        <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
        <nzb xmlns="http://www.newzbin.com/DTD/2003/nzb">
            <head>
                <meta type="title">Your File!</meta>
                <meta type="password">secret</meta>
                <meta type="tag">HD</meta>
                <meta type="category">TV</meta>
            </head>
            <file poster="Joe Bloggs &lt;bloggs@nowhere.example&gt;" date="1071674882" subject="Here's your file!  abc-mr2a.r01 (1/2)">
                <groups>
                    <group>alt.binaries.newzbin</group>
                    <group>alt.binaries.mojo</group>
                </groups>
                <segments>
                    <segment bytes="102394" number="1">123456789abcdef@news.newzbin.com</segment>
                    <segment bytes="4501" number="2">987654321fedbca@news.newzbin.com</segment>
                </segments>
            </file>
        </nzb>
        "#.trim();

        let sanitized = r#"
        <nzb xmlns="http://www.newzbin.com/DTD/2003/nzb">
            <head>
                <meta type="title">Your File!</meta>
                <meta type="password">secret</meta>
                <meta type="tag">HD</meta>
                <meta type="category">TV</meta>
            </head>
            <file poster="Joe Bloggs &lt;bloggs@nowhere.example&gt;" date="1071674882" subject="Here's your file!  abc-mr2a.r01 (1/2)">
                <groups>
                    <group>alt.binaries.newzbin</group>
                    <group>alt.binaries.mojo</group>
                </groups>
                <segments>
                    <segment bytes="102394" number="1">123456789abcdef@news.newzbin.com</segment>
                    <segment bytes="4501" number="2">987654321fedbca@news.newzbin.com</segment>
                </segments>
            </file>
        </nzb>
        "#.trim();

        assert_eq!(sanitize_xml(original), sanitized)
    }
}
