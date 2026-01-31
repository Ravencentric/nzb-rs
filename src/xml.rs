/// Removes the leading XML declaration and/or DOCTYPE from the input.
///
/// This is intended for use with `roxmltree`, which does not support XML
/// declarations or DOCTYPEs, and strips those constructs from the beginning
/// of the document while leaving the rest unchanged.
fn strip_headers(xml: &str) -> &str {
    let mut s = xml.trim();

    // Strip XML declaration: <?xml ... ?>
    if s.len() >= 5
        && s[..5].eq_ignore_ascii_case("<?xml")
        && let Some(end) = s.find("?>")
    {
        s = s[end + 2..].trim_start();
    }

    // Strip DOCTYPE: <!DOCTYPE ... >
    if s.len() >= 9
        && s[..9].eq_ignore_ascii_case("<!DOCTYPE")
        && let Some(end) = s.find('>')
    {
        s = s[end + 1..].trim_start();
    }

    s
}

/// Thin wrapper around `roxmltree::Document::parse` that strips unsupported
/// XML declarations and DOCTYPEs before parsing.
pub(crate) fn parse_document(xml: &str) -> Result<roxmltree::Document<'_>, roxmltree::Error> {
    let stripped = strip_headers(xml);
    roxmltree::Document::parse(stripped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::Document;

    #[test]
    fn test_strip_headers() {
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

        let stripped = r#"
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

        assert_eq!(strip_headers(original), stripped)
    }

    #[test]
    fn test_parse_document() {
        let nzb = r#"
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

        let control = r#"
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

        assert!(Document::parse(nzb).is_err());
        assert!(Document::parse(control).is_ok());
        assert!(parse_document(nzb).is_ok());
        assert!(parse_document(control).is_ok());

        let subjects = vec!["Here's your file!  abc-mr2a.r01 (1/2)"];

        fn extract_subjects(doc: &Document) -> Vec<String> {
            doc.descendants()
                .filter(|n| n.has_tag_name("file"))
                .filter_map(|n| n.attribute("subject").map(String::from))
                .collect()
        }

        assert_eq!(extract_subjects(&Document::parse(control).unwrap()), subjects);
        assert_eq!(extract_subjects(&parse_document(control).unwrap()), subjects);
    }
}
