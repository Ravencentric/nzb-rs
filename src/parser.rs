use crate::errors::{FileAttributeKind, ParseNzbError};
use crate::{File, Meta, Segment, subparsers};
use chrono::DateTime;
use regex::Regex;
use roxmltree::Document;
use std::sync::LazyLock;

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

    files.sort_by_key(|f| subparsers::sort_key_from_subject(&f.subject));

    Ok(files)
}

/// Return [`true`] if the file is obfuscated, [`false`] otherwise.
///
/// This function is pretty much a straight port of the same from `SABnzbd`:
/// <https://github.com/sabnzbd/sabnzbd/blob/297455cd35c71962d39a36b7f99622f905d2234e/sabnzbd/deobfuscate_filenames.py#L104>
///
/// The original accepts either a complete path or just the stem (basename) but
/// this ONLY accepts the latter.
pub(crate) fn sabnzbd_is_obfuscated(filestem: &str) -> bool {
    // In a lot of cases, we do not care about anything other than ASCII characters.
    // So, we can work on the byte level for minor performance gains.
    let filestem_bytes = filestem.as_bytes();
    let length = filestem_bytes.len();

    // First, the patterns that are certainly obfuscated:

    // 32-character hex strings, e.g.
    // ...blabla.H.264/b082fa0beaa644d3aa01045d5b8d0b36.mkv is certainly obfuscated
    if length == 32 && filestem_bytes.iter().all(|b| b.is_ascii_hexdigit()) {
        return true;
    }

    // 40+ chars consisting only of hex digits and dots, e.g.
    // 0675e29e9abfd2.f7d069dab0b853283cc1b069a25f82.6547
    if length >= 40 && filestem_bytes.iter().all(|b| b.is_ascii_hexdigit() || *b == b'.') {
        return true;
    }

    // "[BlaBla] something [More] something 5937bc5e32146e.bef89a622e4a23f07b0d3757ad5e8a.a02b264e [Brrr]"
    // So: square brackets plus 30+ hex digit
    static HEX_DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[a-f0-9]{30}").unwrap());
    static WORDS_IN_SQUARE_BRACKETS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\w+\]").unwrap());
    if HEX_DIGITS.is_match(filestem) && WORDS_IN_SQUARE_BRACKETS.captures_iter(filestem).count() >= 2 {
        return true;
    }

    // /some/thing/abc.xyz.a4c567edbcbf27.BLA is certainly obfuscated
    if filestem_bytes.starts_with(b"abc.xyz") {
        return true;
    }

    // Then, patterns that are not obfuscated but typical, clear names:

    // these are signals for the obfuscation versus non-obfuscation
    let mut decimals: u32 = 0;
    let mut upperchars: u32 = 0;
    let mut lowerchars: u32 = 0;
    let mut spacesdots: u32 = 0;

    for char in filestem.chars() {
        if char.is_ascii_digit() {
            decimals += 1;
        }
        if char.is_uppercase() {
            upperchars += 1;
        }
        if char.is_lowercase() {
            lowerchars += 1;
        }
        if char == ' ' || char == '.' || char == '_' {
            spacesdots += 1;
        }
    }

    // Example: "Great Distro"
    if upperchars >= 2 && lowerchars >= 2 && spacesdots >= 1 {
        return false;
    }

    // Example: "this is a download"
    if spacesdots >= 3 {
        return false;
    }

    // Example: "Beast 2020"
    if (upperchars + lowerchars >= 4) && decimals >= 4 && spacesdots >= 1 {
        return false;
    }

    // Example: "Catullus", starts with a capital, and most letters are lower case
    if filestem.chars().next().is_some_and(char::is_uppercase)
        && lowerchars > 2
        && (upperchars as f64) / (lowerchars as f64) <= 0.25
    {
        return false;
    }
    // Finally: default to obfuscated
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::path::Path;

    // Test cases copied from SABnzbd’s filename deobfuscation tests.
    // Thanks to the SABnzbd project for these examples.
    //
    // The original tests are split here into two functions
    // (test_sabnzbd_is_obfuscated_true / test_sabnzbd_is_obfuscated_false) for convenience.
    //
    // Source:
    // https://github.com/sabnzbd/sabnzbd/blob/11ba9ae12ade8c8f2abb42d44ea35efdd361fae5/tests/test_deobfuscate_filenames.py#L43
    #[rstest]
    #[case("599c1c9e2bdfb5114044bf25152b7eaa.mkv")]
    #[case("/my/blabla/directory/stuff/599c1c9e2bdfb5114044bf25152b7eaa.mkv")]
    #[case("/my/blabla/directory/A Directory Should Not Count 2020/599c1c9e2bdfb5114044bf25152b7eaa.mkv")]
    #[case("/my/blabla/directory/stuff/afgm.avi")]
    #[case("/my/blabla/directory/stuff/afgm2020.avi")]
    #[case("MUGNjK3zi65TtN.mkv")]
    #[case("T306077.avi")]
    #[case("bar10nmbkkjjdfr.mkv")]
    #[case("4rFF-fdtd480p.bin")]
    #[case("e0nFmxBNTprpbQiVQ44WeEwSrBkLlJ7IgaSj3uzFu455FVYG3q.bin")]
    #[case("e0nFmxBNTprpbQiVQ44WeEwSrBkLlJ7IgaSj3uzFu455FVYG3q")] // no ext
    #[case("greatdistro.iso")]
    #[case("my.download.2020")]
    #[case("abc.xyz.a4c567edbcbf27.BLA")] // by definition
    #[case("abc.xyz.iso")] // lazy brother
    #[case("0675e29e9abfd2.f7d069dab0b853283cc1b069a25f82.6547")]
    #[case("[BlaBla] something [More] something b2.bef89a622e4a23f07b0d3757ad5e8a.a0 [Brrr]")]
    fn test_sabnzbd_is_obfuscated_true(#[case] filename: &str) {
        let filestem = Path::new(filename).file_stem().and_then(|f| f.to_str()).unwrap();
        assert!(sabnzbd_is_obfuscated(filestem));
    }

    #[rstest]
    #[case("/my/blabla/directory/stuff/My Favorite Distro S03E04.iso")]
    #[case("/my/blabla/directory/stuff/Great Distro (2020).iso")]
    #[case("ubuntu.2004.iso")]
    #[case("/my/blabla/directory/stuff/GreatDistro2020.iso")]
    #[case("Catullus.avi")]
    #[case("Der.Mechaniker.HDRip.XviD-SG.avi")]
    #[case("Bonjour.1969.FRENCH.BRRiP.XviD.AC3-HuSh.avi")]
    #[case("Bonjour.1969.avi")]
    #[case("This That S01E11")]
    #[case("This_That_S01E11")]
    #[case("this_that_S01E11")]
    #[case("My.Download.2020")]
    #[case("this_that_there_here.avi")]
    #[case("Lorem Ipsum.avi")]
    #[case("Lorem Ipsum")] // no ext
    fn test_sabnzbd_is_obfuscated_false(#[case] filename: &str) {
        let filestem = Path::new(filename).file_stem().and_then(|f| f.to_str()).unwrap();
        assert!(!sabnzbd_is_obfuscated(filestem));
    }

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
