use chrono::DateTime;
use roxmltree::Document;

use crate::errors::{FileAttributeKind, ParseNzbError};
use crate::subject;
use crate::{File, Meta, Segment};

/// NZB `<meta type="...">` values defined by the
/// [`Metadata Defined Types` in the NZB specification][0].
///
/// [0]: https://sabnzbd.org/wiki/extra/nzb-spec
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetaType {
    Title,
    Password,
    Tag,
    Category,
}

impl MetaType {
    /// Parse a string into a [`MetaType`] (case-insensitive).
    ///
    /// Returns `None` if the string does not match a known metadata type.
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

/// Parse `<meta>` fields from an NZB document.
///
/// Extracts metadata from `<meta>` elements under `<head>` according
/// to the [NZB specification’s `Metadata Defined Types`][0].
///
/// Supported types:
/// - `title` (single)
/// - `password` (multiple allowed)
/// - `tag` (multiple allowed)
/// - `category` (single)
///
/// Unknown `<meta type="...">` values are ignored. If multiple
/// `title` or `category` entries are present, the first one wins.
///
/// [0]: <https://sabnzbd.org/wiki/extra/nzb-spec>
///
/// # Example
///
/// ```xml
/// <head>
///     <meta type="title">Your File!</meta>
///     <meta type="password">secret</meta>
///     <meta type="tag">HD</meta>
///     <meta type="category">TV</meta>
/// </head>
/// ```
pub(crate) fn parse_metadata(nzb: &Document) -> Meta {
    let mut title: Option<String> = None;
    let mut passwords: Vec<String> = Vec::new();
    let mut tags: Vec<String> = Vec::new();
    let mut category: Option<String> = None;

    for meta in nzb.descendants().filter(|n| n.has_tag_name("meta")) {
        let Some(typ) = meta.attribute("type").and_then(MetaType::parse) else {
            continue;
        };

        let Some(text) = meta.text().map(String::from) else {
            continue;
        };

        match typ {
            MetaType::Title => title = title.or(Some(text)),
            MetaType::Password => passwords.push(text),
            MetaType::Tag => tags.push(text),
            MetaType::Category => category = category.or(Some(text)),
        }
    }

    Meta {
        title,
        passwords,
        tags,
        category,
    }
}

/// Parse all `<file>` elements from an NZB Document.
///
/// Each `<file>` must contain:
/// - `poster`, `date`, and `subject` attributes
/// - at least one `<group>` inside `<groups>`
/// - at least one `<segment>` inside `<segments>`
///
/// Segments missing required attributes (`bytes`, `number`) or message IDs
/// are skipped rather than causing a hard error.
pub(crate) fn parse_files(nzb: &Document) -> Result<Vec<File>, ParseNzbError> {
    let mut files = Vec::new();

    for node in nzb.root_element().children().filter(|n| n.has_tag_name("file")) {
        let poster = node
            .attribute("poster")
            .ok_or(ParseNzbError::FileAttribute(FileAttributeKind::Poster))?;

        let posted_at = node
            .attribute("date") // Unix timestamp in seconds
            .and_then(|d| d.parse::<i64>().ok())
            .and_then(|d| DateTime::from_timestamp(d, 0))
            .ok_or(ParseNzbError::FileAttribute(FileAttributeKind::Date))?;

        let subject = node
            .attribute("subject")
            .ok_or(ParseNzbError::FileAttribute(FileAttributeKind::Subject))?;

        let mut groups = Vec::new();
        let mut segments = Vec::new();

        for child in node.children() {
            match child.tag_name().name() {
                "groups" => {
                    for group in child.children().filter(|n| n.has_tag_name("group")) {
                        if let Some(text) = group.text()
                            && !text.is_empty()
                        {
                            groups.push(text.to_owned());
                        }
                    }
                }

                "segments" => {
                    for segment in child.children().filter(|n| n.has_tag_name("segment")) {
                        // Message-ID text is required and must be non-empty.
                        if let Some(message_id) = segment.text()
                            && !message_id.is_empty()
                        {
                            // Article size is typically ~700KB and safely fits in u32.
                            let Some(size) = segment.attribute("bytes").and_then(|bytes| bytes.parse::<u32>().ok())
                            else {
                                continue;
                            };
                            let Some(number) = segment
                                .attribute("number")
                                .and_then(|number| number.parse::<u32>().ok())
                            else {
                                continue;
                            };

                            segments.push(Segment {
                                size,
                                number,
                                message_id: message_id.to_owned(),
                            });
                        }
                    }
                }
                _ => continue,
            }
        }

        // A file must belong to at least one group.
        if groups.is_empty() {
            return Err(ParseNzbError::GroupsElement);
        }

        // A file must contain at least one valid segment.
        if segments.is_empty() {
            return Err(ParseNzbError::SegmentsElement);
        }

        // Sort for consistency
        groups.sort_unstable();
        segments.sort_unstable_by_key(|segment| segment.number);

        files.push(File {
            poster: poster.to_owned(),
            posted_at,
            subject: subject.to_owned(),
            groups,
            segments,
        });
    }

    // The NZB must contain at least one <file>.
    if files.is_empty() {
        return Err(ParseNzbError::FileElement);
    }

    // Reject NZBs that contain only PAR2 repair files.
    if files.iter().all(File::is_par2) {
        return Err(ParseNzbError::OnlyPar2Files);
    }

    files.sort_unstable_by(|a, b| {
        let ka = subject::file_number(&a.subject);
        let kb = subject::file_number(&b.subject);
        ka.cmp(&kb).then_with(|| a.subject.cmp(&b.subject))
    });

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::Document;
    use rstest::rstest;

    fn parse_metadata_from_xml(xml: &str) -> Meta {
        let doc = Document::parse(xml.trim()).expect("valid XML");
        parse_metadata(&doc)
    }

    #[rstest]
    #[case("title", MetaType::Title)]
    #[case("password", MetaType::Password)]
    #[case("tag", MetaType::Tag)]
    #[case("category", MetaType::Category)]
    #[case("TITLE", MetaType::Title)]
    #[case("PaSsWoRd", MetaType::Password)]
    #[case("TaG", MetaType::Tag)]
    #[case("CATEGORY", MetaType::Category)]
    fn test_metatype_parses_supported_types_case_insensitive(#[case] input: &str, #[case] expected: MetaType) {
        assert_eq!(MetaType::parse(input), Some(expected));
    }

    #[rstest]
    #[case("x-custom")]
    #[case("unknown")]
    #[case("")]
    #[case(" ")]
    fn test_metatype_rejects_unknown_types(#[case] input: &str) {
        assert_eq!(MetaType::parse(input), None);
    }

    #[test]
    fn test_supported_meta_types() {
        let meta = parse_metadata_from_xml(
            r#"
            <nzb>
                <head>
                    <meta type="title">My Title</meta>
                    <meta type="password">secret</meta>
                    <meta type="tag">HD</meta>
                    <meta type="category">TV</meta>
                </head>
            </nzb>
            "#,
        );

        assert_eq!(meta.title, Some("My Title".into()));
        assert_eq!(meta.passwords, vec!["secret"]);
        assert_eq!(meta.tags, vec!["HD"]);
        assert_eq!(meta.category, Some("TV".into()));
    }

    #[test]
    fn test_multiple_passwords_and_tags_in_order() {
        let meta = parse_metadata_from_xml(
            r#"
            <nzb>
                <head>
                    <meta type="password">one</meta>
                    <meta type="password">two</meta>
                    <meta type="tag">HD</meta>
                    <meta type="tag">x265</meta>
                </head>
            </nzb>
            "#,
        );

        assert_eq!(meta.passwords, vec!["one", "two"]);
        assert_eq!(meta.tags, vec!["HD", "x265"]);
    }

    #[test]
    fn test_first_title_wins() {
        let meta = parse_metadata_from_xml(
            r#"
            <nzb>
                <head>
                    <meta type="title">First</meta>
                    <meta type="title">Second</meta>
                </head>
            </nzb>
            "#,
        );

        assert_eq!(meta.title.as_deref(), Some("First"));
    }

    #[test]
    fn test_first_category_wins() {
        let meta = parse_metadata_from_xml(
            r#"
            <nzb>
                <head>
                    <meta type="category">Movies</meta>
                    <meta type="category">TV</meta>
                </head>
            </nzb>
            "#,
        );

        assert_eq!(meta.category.as_deref(), Some("Movies"));
    }

    #[test]
    fn test_ignores_unknown_meta_types() {
        let meta = parse_metadata_from_xml(
            r#"
            <nzb>
                <head>
                    <meta type="title">Known</meta>
                    <meta type="x-custom">Ignored</meta>
                    <meta type="nonsense">Ignored</meta>
                </head>
            </nzb>
            "#,
        );

        assert_eq!(meta.title.as_deref(), Some("Known"));
        assert!(meta.passwords.is_empty());
        assert!(meta.tags.is_empty());
        assert!(meta.category.is_none());
    }

    #[test]
    fn test_meta_type_is_case_insensitive_in_parser() {
        let meta = parse_metadata_from_xml(
            r#"
            <nzb>
                <head>
                    <meta type="TITLE">Upper</meta>
                    <meta type="PaSsWoRd">secret</meta>
                </head>
            </nzb>
            "#,
        );
        assert_eq!(meta.title.as_deref(), Some("Upper"));
        assert_eq!(meta.passwords, vec!["secret"]);
    }
}
