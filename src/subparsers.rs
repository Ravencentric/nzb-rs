use lazy_regex::regex;

/// Extract the complete name of the file with it's extension from the subject.
/// May return `None` if it fails to extract the name.
pub(crate) fn extract_filename_from_subject(subject: &str) -> Option<&str> {
    // The order of regular expressions is deliberate; patterns are arranged
    // from most specific to most general to avoid broader patterns incorrectly matching.

    // Case 1: Filename is in quotes.
    // We use a more relaxed version of what SABnzbd does:
    // https://github.com/sabnzbd/sabnzbd/blob/02b4a116dd4b46b2d2f33f7bbf249f2294458f2e/sabnzbd/nzbstuff.py#L104-L106
    if let Some(captured) = regex!(r#""(.*)""#).captures(subject) {
        return captured.get(1).map(|m| m.as_str().trim());
    }

    // Case 2: Subject follows a specific pattern.
    // https://regex101.com/r/B03qZs/2
    // [011/116] - [AC-FFF] Highschool DxD BorN - 02 [BD][1080p-Hi10p] FLAC][Dual-Audio][442E5446].mkv yEnc (1/2401) 1720916370
    if let Some(captured) =
        regex!(r"^(?:\[|\()(?:\d+/\d+)(?:\]|\))\s-\s(.*)\syEnc\s(?:\[|\()(?:\d+/\d+)(?:\]|\))\s\d+").captures(subject)
    {
        return captured.get(1).map(|m| m.as_str().trim());
    }

    // Case 3: Something that might look like a filename.
    // https://github.com/sabnzbd/sabnzbd/blob/02b4a116dd4b46b2d2f33f7bbf249f2294458f2e/sabnzbd/nzbstuff.py#L104-L106
    if let Some(captured) =
        regex!(r"\b([\w\-+()' .,]+(?:\[[\w\-/+()' .,]*][\w\-+()' .,]*)*\.[A-Za-z0-9]{2,4})\b").captures(subject)
    {
        return captured.get(1).map(|m| m.as_str().trim());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        "[011/116] - [AC-FFF] Highschool DxD BorN - 02 [BD][1080p-Hi10p] FLAC][Dual-Audio][442E5446].mkv yEnc (1/2401) 1720916370",
        "[AC-FFF] Highschool DxD BorN - 02 [BD][1080p-Hi10p] FLAC][Dual-Audio][442E5446].mkv"
    )]
    #[case(
        "[010/108] - [SubsPlease] Ijiranaide, Nagatoro-san - 02 (1080p) [6E8E8065].mkv yEnc (1/2014) 1443366873",
        "[SubsPlease] Ijiranaide, Nagatoro-san - 02 (1080p) [6E8E8065].mkv"
    )]
    #[case(
        r#"[1/8] - "TenPuru - No One Can Live on Loneliness v05 {+ "Book of Earthly Desires" pamphlet} (2021) (Digital) (KG Manga).cbz" yEnc (1/230) 164676947"#, 
        r#"TenPuru - No One Can Live on Loneliness v05 {+ "Book of Earthly Desires" pamphlet} (2021) (Digital) (KG Manga).cbz"#
    )]
    fn test_extract_filename_from_subject(#[case] subject: &str, #[case] filename: &str) {
        assert_eq!(extract_filename_from_subject(subject), Some(filename));
    }
}
