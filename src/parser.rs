use crate::{File, InvalidNZBError, Meta, Segment};
use chrono::DateTime;
use lazy_regex::regex;
use roxmltree::Document;

pub(crate) fn sanitize_xml(xml: &str) -> &str {
    let xml_heading_re = regex!(r"^(?i)<\?xml\s+version.*\?>");
    let xml_doctype_re = regex!(r"^(?i)<!DOCTYPE.*>");
    let mut content = xml.trim();
    if let Some(found) = xml_heading_re.find(content) {
        content = &content[found.end()..];
        content = content.trim_start();
    }
    if let Some(found) = xml_doctype_re.find(content) {
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
                    if let Some(text) = meta.text().map(String::from) {
                        if !passwords.contains(&text) {
                            passwords.push(text);
                        }
                    }
                }
                "tag" => {
                    if let Some(text) = meta.text().map(String::from) {
                        if !tags.contains(&text) {
                            tags.push(text);
                        }
                    }
                }
                "category" => {
                    category = category.or(meta.text().map(String::from));
                }
                _ => {}
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
pub(crate) fn parse_files(nzb: &Document) -> Result<Vec<File>, InvalidNZBError> {
    let mut files = Vec::new();
    let file_nodes = nzb.descendants().filter(|n| n.has_tag_name("file"));

    for node in file_nodes {
        // These are guraranteed to exist, so we can unwrap them.
        let poster = node.attribute("poster").unwrap().to_string();
        let date = node.attribute("date").unwrap().parse::<i64>().unwrap();
        let datetime = DateTime::from_timestamp(date, 0).unwrap();
        let subject = node.attribute("subject").unwrap().to_string();

        let mut groups = Vec::new();
        let mut segments = Vec::new();

        if let Some(groups_node) = node.descendants().find(|n| n.has_tag_name("groups")) {
            for group in groups_node.descendants().filter(|n| n.has_tag_name("group")) {
                groups.push(group.text().unwrap().to_string());
            }
        }

        // There must be at least one group.
        if groups.is_empty() {
            return Err(InvalidNZBError::new("Missing or malformed <groups>...</groups>!"));
        }

        if let Some(segment_node) = node.descendants().find(|n| n.has_tag_name("segments")) {
            for segment in segment_node.descendants().filter(|n| n.has_tag_name("segment")) {
                let size = match segment.attribute("bytes") {
                    Some(attr) => match attr.parse::<u32>() {
                        Ok(size) => size,
                        Err(_) => continue, // Skip this segment if parsing fails
                    },
                    None => continue, // Skip this segment if the attribute is missing
                };

                let number = match segment.attribute("number") {
                    Some(attr) => match attr.parse::<u32>() {
                        Ok(number) => number,
                        Err(_) => continue, // Skip this segment if parsing fails
                    },
                    None => continue, // Skip this segment if the attribute is missing
                };

                let message_id = match segment.text() {
                    Some(text) => text,
                    None => continue, // Skip this segment if the text is missing
                };

                segments.push(Segment::new(size, number, message_id));
            }
        }

        // There must be at least one segment.
        if segments.is_empty() {
            return Err(InvalidNZBError::new("Missing or malformed <segments>...</segments>!"));
        }

        files.push(File {
            poster,
            datetime,
            subject,
            groups,
            segments,
        });
    }

    // There must be at least one file.
    if files.is_empty() {
        return Err(InvalidNZBError::new("Missing or malformed <file>...</file>!"));
    }

    Ok(files)
}

/// Return [`true`] if the file is obfuscated, [`false`] otherwise.
///
/// This function is pretty much a straight port of the same from SABnzbd:
/// https://github.com/sabnzbd/sabnzbd/blob/297455cd35c71962d39a36b7f99622f905d2234e/sabnzbd/deobfuscate_filenames.py#L104
///
/// The original accepts either a complete path or just the stem (basename) but
/// this ONLY accepts the latter.
pub(crate) fn sabnzbd_is_obfuscated(filestem: &str) -> bool {
    // First: the patterns that are certainly obfuscated:

    // ...blabla.H.264/b082fa0beaa644d3aa01045d5b8d0b36.mkv is certainly obfuscated
    if regex!(r"^[a-f0-9]{32}$").is_match(filestem) {
        return true;
    }

    // 0675e29e9abfd2.f7d069dab0b853283cc1b069a25f82.6547
    if regex!(r"^[a-f0-9.]{40,}$").is_match(filestem) {
        return true;
    }

    // "[BlaBla] something [More] something 5937bc5e32146e.bef89a622e4a23f07b0d3757ad5e8a.a02b264e [Brrr]"
    // So: square brackets plus 30+ hex digit
    if regex!(r"[a-f0-9]{30}").is_match(filestem)
        && regex!(r"\[\w+\]").find_iter(filestem).collect::<Vec<_>>().len() >= 2
    {
        return true;
    }

    // /some/thing/abc.xyz.a4c567edbcbf27.BLA is certainly obfuscated
    if regex!(r"^abc\.xyz").is_match(filestem) {
        return true;
    }

    // Then: patterns that are not obfuscated but typical, clear names:

    // these are signals for the obfuscation versus non-obfuscation
    let decimals = filestem.chars().filter(|c| c.is_numeric()).count();
    let upperchars = filestem.chars().filter(|c| c.is_uppercase()).count();
    let lowerchars = filestem.chars().filter(|c| c.is_lowercase()).count();
    let spacesdots = filestem.chars().filter(|&c| c == ' ' || c == '.' || c == '_').count();

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
    if filestem.chars().next().is_some_and(|c| c.is_uppercase())
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
    use std::path::Path;

    fn get_stem(p: &str) -> &str {
        Path::new(p).file_stem().map(|f| f.to_str()).flatten().unwrap()
    }

    /// https://github.com/sabnzbd/sabnzbd/blob/42c00dda8455c82d691615259775a30661a752bd/tests/test_deobfuscate_filenames.py#L43
    #[test]
    fn test_sabnzbd_is_obfuscated() {
        assert!(sabnzbd_is_obfuscated(get_stem("599c1c9e2bdfb5114044bf25152b7eaa.mkv")));
        assert!(sabnzbd_is_obfuscated(get_stem(
            "/my/blabla/directory/stuff/599c1c9e2bdfb5114044bf25152b7eaa.mkv"
        )));
        assert!(sabnzbd_is_obfuscated(get_stem(
            "/my/blabla/directory/A Directory Should Not Count 2020/599c1c9e2bdfb5114044bf25152b7eaa.mkv"
        )));
        assert!(sabnzbd_is_obfuscated(get_stem("/my/blabla/directory/stuff/afgm.avi")));
        assert!(sabnzbd_is_obfuscated(get_stem(
            "/my/blabla/directory/stuff/afgm2020.avi"
        )));
        assert!(sabnzbd_is_obfuscated(get_stem("MUGNjK3zi65TtN.mkv")));
        assert!(sabnzbd_is_obfuscated(get_stem("T306077.avi")));
        assert!(sabnzbd_is_obfuscated(get_stem("bar10nmbkkjjdfr.mkv")));
        assert!(sabnzbd_is_obfuscated(get_stem("4rFF-fdtd480p.bin")));
        assert!(sabnzbd_is_obfuscated(get_stem(
            "e0nFmxBNTprpbQiVQ44WeEwSrBkLlJ7IgaSj3uzFu455FVYG3q.bin"
        )));
        assert!(sabnzbd_is_obfuscated(get_stem(
            "e0nFmxBNTprpbQiVQ44WeEwSrBkLlJ7IgaSj3uzFu455FVYG3q"
        ))); // no ext
        assert!(sabnzbd_is_obfuscated(get_stem("greatdistro.iso")));
        assert!(sabnzbd_is_obfuscated(get_stem("my.download.2020")));
        assert!(sabnzbd_is_obfuscated(get_stem("abc.xyz.a4c567edbcbf27.BLA")));
        assert!(sabnzbd_is_obfuscated(get_stem("abc.xyz.iso")));
        assert!(sabnzbd_is_obfuscated(get_stem(
            "0675e29e9abfd2.f7d069dab0b853283cc1b069a25f82.6547"
        )));
        assert!(sabnzbd_is_obfuscated(get_stem(
            "[BlaBla] something [More] something b2.bef89a622e4a23f07b0d3757ad5e8a.a0 [Brrr]"
        )));

        // non-obfuscated names:
        assert!(!sabnzbd_is_obfuscated(get_stem(
            "/my/blabla/directory/stuff/My Favorite Distro S03E04.iso"
        )));
        assert!(!sabnzbd_is_obfuscated(get_stem(
            "/my/blabla/directory/stuff/Great Distro (2020).iso"
        )));
        assert!(!sabnzbd_is_obfuscated(get_stem("ubuntu.2004.iso")));
        assert!(!sabnzbd_is_obfuscated(get_stem(
            "/my/blabla/directory/stuff/GreatDistro2020.iso"
        )));
        assert!(!sabnzbd_is_obfuscated(get_stem("Catullus.avi")));
        assert!(!sabnzbd_is_obfuscated(get_stem("Der.Mechaniker.HDRip.XviD-SG.avi")));
        assert!(!sabnzbd_is_obfuscated(get_stem(
            "Bonjour.1969.FRENCH.BRRiP.XviD.AC3-HuSh.avi"
        )));
        assert!(!sabnzbd_is_obfuscated(get_stem("Bonjour.1969.avi")));
        assert!(!sabnzbd_is_obfuscated(get_stem("This That S01E11")));
        assert!(!sabnzbd_is_obfuscated(get_stem("This_That_S01E11")));
        assert!(!sabnzbd_is_obfuscated(get_stem("this_that_S01E11")));
        assert!(!sabnzbd_is_obfuscated(get_stem("My.Download.2020")));
        assert!(!sabnzbd_is_obfuscated(get_stem("this_that_there_here.avi")));
        assert!(!sabnzbd_is_obfuscated(get_stem("Lorem Ipsum.avi")));
        assert!(!sabnzbd_is_obfuscated(get_stem("Lorem Ipsum"))); // no ext
    }
}
