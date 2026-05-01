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

/// Represents optional creator-definable metadata in an NZB.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Meta {
    title: Option<String>,
    passwords: Vec<String>,
    tags: Vec<String>,
    category: Option<String>,
}

impl Meta {
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
    pub(crate) fn parse(nzb: &roxmltree::Document) -> Self {
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

    /// Creates a new [`Meta`] instance.
    #[must_use]
    pub fn new(
        title: Option<impl Into<String>>,
        passwords: impl IntoIterator<Item = impl Into<String>>,
        tags: impl IntoIterator<Item = impl Into<String>>,
        category: Option<impl Into<String>>,
    ) -> Self {
        Self {
            title: title.map(Into::into),
            passwords: passwords.into_iter().map(Into::into).collect(),
            tags: tags.into_iter().map(Into::into).collect(),
            category: category.map(Into::into),
        }
    }

    /// Human-readable title associated with the NZB.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Password entries associated with the NZB.
    #[must_use]
    pub fn passwords(&self) -> &[String] {
        &self.passwords
    }

    /// Tags associated with the NZB.
    #[must_use]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Category associated with the NZB.
    #[must_use]
    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use roxmltree::Document;
    use rstest::rstest;

    use super::*;

    fn parse_metadata_from_xml(xml: &str) -> Meta {
        let doc = Document::parse(xml.trim()).expect("valid XML");
        Meta::parse(&doc)
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

        assert_eq!(meta.title(), Some("My Title"));
        assert_eq!(meta.passwords(), vec!["secret"]);
        assert_eq!(meta.tags(), vec!["HD"]);
        assert_eq!(meta.category(), Some("TV"));
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

        assert_eq!(meta.passwords(), vec!["one", "two"]);
        assert_eq!(meta.tags(), vec!["HD", "x265"]);
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

        assert_eq!(meta.category(), Some("Movies"));
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

        assert_eq!(meta.title(), Some("Known"));
        assert!(meta.passwords().is_empty());
        assert!(meta.tags().is_empty());
        assert!(meta.category().is_none());
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
        assert_eq!(meta.title(), Some("Upper"));
        assert_eq!(meta.passwords(), vec!["secret"]);
    }
}
