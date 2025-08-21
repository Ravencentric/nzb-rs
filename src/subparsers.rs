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
        let trimmed = captured.get(1).map(|m| m.as_str().trim());
        if let Some(s) = trimmed
            && !s.is_empty()
        {
            return Some(s);
        }
    }

    // Case 2: Subject follows a specific pattern.
    // https://regex101.com/r/B03qZs/2
    // [011/116] - [AC-FFF] Highschool DxD BorN - 02 [BD][1080p-Hi10p] FLAC][Dual-Audio][442E5446].mkv yEnc (1/2401) 1720916370
    if let Some(captured) =
        regex!(r"^(?:\[|\()(?:\d+/\d+)(?:\]|\))\s-\s(.*)\syEnc\s(?:\[|\()(?:\d+/\d+)(?:\]|\))\s\d+").captures(subject)
    {
        let trimmed = captured.get(1).map(|m| m.as_str().trim());
        if let Some(s) = trimmed
            && !s.is_empty()
        {
            return Some(s);
        }
    }

    // Case 3: Something that might look like a filename.
    // https://github.com/sabnzbd/sabnzbd/blob/02b4a116dd4b46b2d2f33f7bbf249f2294458f2e/sabnzbd/nzbstuff.py#L104-L106
    if let Some(captured) =
        regex!(r"\b([\w\-+()' .,]+(?:\[[\w\-/+()' .,]*][\w\-+()' .,]*)*\.[A-Za-z0-9]{2,4})\b").captures(subject)
    {
        let trimmed = captured.get(1).map(|m| m.as_str().trim());
        if let Some(s) = trimmed
            && !s.is_empty()
        {
            return Some(s);
        }
    }

    None
}

/// Splits a filename into a stem and an extension based on a specific pattern.
/// `Path.extension()` has too many false positives, so we use a custom regex.
///
/// Returns a tuple containing the `(stem, Option<extension>)`.
/// If no valid extension is found, the extension is `None`.
pub(crate) fn split_filename_at_extension(filename: &str) -> (&str, Option<&str>) {
    let re = regex!(r"(\.[a-z]\w{2,5})$"i);

    if let Some(found) = re.find(filename) {
        // +1 to skip the dot in the extension to match the behavior of `Path::extension()`,
        // which does not include the dot in the returned extension.
        let start = found.start();
        let extension = &filename[start + 1..];
        let stem = &filename[..start];
        (stem, Some(extension))
    } else {
        (filename, None)
    }
}

/// Returns a normalized subject string with "[1/10]" → "[01/10]" style zero-padding.
/// If no match is found, returns the original string unchanged.
pub(crate) fn sort_key_from_subject(subject: &str) -> String {
    let pattern = regex!(r"^\[(\d+)\/(\d+)\]");
    if let Some((_, [current, total])) = pattern.captures(subject).map(|caps| caps.extract::<2>()) {
        let width = total.len();
        let current = format!("{current:0>width$}");
        pattern.replace(subject, format!("[{current}/{total}]")).to_string()
    } else {
        subject.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;
    use rstest::rstest;

    #[rstest]
    #[case(
        "[011/116] - [AC-FFF] Highschool DxD BorN - 02 [BD][1080p-Hi10p] FLAC][Dual-Audio][442E5446].mkv yEnc (1/2401) 1720916370",
        "[AC-FFF] Highschool DxD BorN - 02 [BD][1080p-Hi10p] FLAC][Dual-Audio][442E5446].mkv",
        "[AC-FFF] Highschool DxD BorN - 02 [BD][1080p-Hi10p] FLAC][Dual-Audio][442E5446]",
        Some("mkv")
    )]
    #[case(
        "[010/108] - [SubsPlease] Ijiranaide, Nagatoro-san - 02 (1080p) [6E8E8065].mkv yEnc (1/2014) 1443366873",
        "[SubsPlease] Ijiranaide, Nagatoro-san - 02 (1080p) [6E8E8065].mkv",
        "[SubsPlease] Ijiranaide, Nagatoro-san - 02 (1080p) [6E8E8065]",
        Some("mkv")
    )]
    #[case(
            r#"[1/8] - "TenPuru - No One Can Live on Loneliness v05 {+ "Book of Earthly Desires" pamphlet} (2021) (Digital) (KG Manga).cbz" yEnc (1/230) 164676947"#,
            r#"TenPuru - No One Can Live on Loneliness v05 {+ "Book of Earthly Desires" pamphlet} (2021) (Digital) (KG Manga).cbz"#,
            r#"TenPuru - No One Can Live on Loneliness v05 {+ "Book of Earthly Desires" pamphlet} (2021) (Digital) (KG Manga)"#,
            Some("cbz"),
    )]
    #[case(
        r#"[1/10] - "ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG" yEnc (1/1277) 915318101"#,
        "ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG",
        "ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG",
        None
    )]
    #[case(
        r#"[1/10] - "ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG.mkv" yEnc (1/1277) 915318101"#,
        "ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG.mkv",
        "ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG",
        Some("mkv")
    )]
    #[case(r#"[27/141] - "index.bdmv" yEnc (1/1) 280"#, "index.bdmv", "index", Some("bdmv"))]
    fn test_name_stem_extension_extraction(
        #[case] subject: &str,
        #[case] filename: &str,
        #[case] stem: &str,
        #[case] extension: Option<&str>,
    ) {
        assert_eq!(extract_filename_from_subject(subject), Some(filename));
        assert_eq!(split_filename_at_extension(filename), (stem, extension));
    }

    #[test]
    fn test_sort_key_from_subject() {
        assert_eq!(
            sort_key_from_subject(r#"[10/141] - "00010.clpi" yEnc (1/1) 1000"#),
            r#"[010/141] - "00010.clpi" yEnc (1/1) 1000"#
        );

        assert_eq!(
            sort_key_from_subject(r#""00010.clpi" yEnc (1/1) 1000"#),
            r#""00010.clpi" yEnc (1/1) 1000"#
        );

        let control = vec![
            r#"[1/141] - "00001.clpi" yEnc (1/1) 24248"#,
            r#"[2/141] - "00002.clpi" yEnc (1/1) 860"#,
            r#"[3/141] - "00003.clpi" yEnc (1/1) 480"#,
            r#"[4/141] - "00004.clpi" yEnc (1/1) 1136"#,
            r#"[5/141] - "00005.clpi" yEnc (1/1) 480"#,
            r#"[6/141] - "00006.clpi" yEnc (1/1) 480"#,
            r#"[7/141] - "00007.clpi" yEnc (1/1) 712"#,
            r#"[8/141] - "00008.clpi" yEnc (1/1) 392"#,
            r#"[9/141] - "00009.clpi" yEnc (1/1) 1016"#,
            r#"[10/141] - "00010.clpi" yEnc (1/1) 1000"#,
            r#"[11/141] - "00011.clpi" yEnc (1/1) 296"#,
            r#"[12/141] - "00012.clpi" yEnc (1/1) 296"#,
            r#"[13/141] - "00013.clpi" yEnc (1/1) 332"#,
            r#"[14/141] - "00014.clpi" yEnc (1/1) 332"#,
            r#"[15/141] - "MovieObject.bdmv" yEnc (1/1) 39430"#,
            r#"[16/141] - "00001.mpls" yEnc (1/1) 708"#,
            r#"[17/141] - "00002.mpls" yEnc (1/1) 188"#,
            r#"[18/141] - "00003.mpls" yEnc (1/1) 188"#,
            r#"[19/141] - "00004.mpls" yEnc (1/1) 188"#,
            r#"[20/141] - "00005.mpls" yEnc (1/1) 188"#,
            r#"[21/141] - "00006.mpls" yEnc (1/1) 188"#,
            r#"[22/141] - "00007.mpls" yEnc (1/1) 188"#,
            r#"[23/141] - "00008.mpls" yEnc (1/1) 174"#,
            r#"[24/141] - "00009.mpls" yEnc (1/1) 13046"#,
            r#"[25/141] - "00010.mpls" yEnc (1/1) 158"#,
            r#"[26/141] - "00011.mpls" yEnc (1/1) 158"#,
            r#"[27/141] - "index.bdmv" yEnc (1/1) 280"#,
            r#"[28/141] - "00001.clpi" yEnc (1/1) 24248"#,
            r#"[29/141] - "00002.clpi" yEnc (1/1) 860"#,
            r#"[30/141] - "00003.clpi" yEnc (1/1) 480"#,
            r#"[31/141] - "00004.clpi" yEnc (1/1) 1136"#,
            r#"[32/141] - "00005.clpi" yEnc (1/1) 480"#,
            r#"[33/141] - "00006.clpi" yEnc (1/1) 480"#,
            r#"[34/141] - "00007.clpi" yEnc (1/1) 712"#,
            r#"[35/141] - "00008.clpi" yEnc (1/1) 392"#,
            r#"[36/141] - "00009.clpi" yEnc (1/1) 1016"#,
            r#"[37/141] - "00010.clpi" yEnc (1/1) 1000"#,
            r#"[38/141] - "00011.clpi" yEnc (1/1) 296"#,
            r#"[39/141] - "00012.clpi" yEnc (1/1) 296"#,
            r#"[40/141] - "00013.clpi" yEnc (1/1) 332"#,
            r#"[41/141] - "00014.clpi" yEnc (1/1) 332"#,
            r#"[42/141] - "bdmt_jpn.xml" yEnc (1/1) 446"#,
            r#"[43/141] - "fafner_BTL_L_xmb.jpg" yEnc (1/1) 267785"#,
            r#"[44/141] - "fafner_BTL_S_xmb.jpg" yEnc (1/1) 137371"#,
            r#"[45/141] - "MovieObject.bdmv" yEnc (1/1) 39430"#,
            r#"[46/141] - "00001.mpls" yEnc (1/1) 708"#,
            r#"[47/141] - "00002.mpls" yEnc (1/1) 188"#,
            r#"[48/141] - "00003.mpls" yEnc (1/1) 188"#,
            r#"[49/141] - "00004.mpls" yEnc (1/1) 188"#,
            r#"[50/141] - "00005.mpls" yEnc (1/1) 188"#,
            r#"[51/141] - "00006.mpls" yEnc (1/1) 188"#,
            r#"[52/141] - "00007.mpls" yEnc (1/1) 188"#,
            r#"[53/141] - "00008.mpls" yEnc (1/1) 174"#,
            r#"[54/141] - "00009.mpls" yEnc (1/1) 13046"#,
            r#"[55/141] - "00010.mpls" yEnc (1/1) 158"#,
            r#"[56/141] - "00011.mpls" yEnc (1/1) 158"#,
            r#"[57/141] - "00001.m2ts" yEnc (1/23594) 16911587328"#,
            r#"[58/141] - "00002.m2ts" yEnc (1/378) 270790656"#,
            r#"[59/141] - "00003.m2ts" yEnc (1/99) 70281216"#,
            r#"[60/141] - "00004.m2ts" yEnc (1/649) 464873472"#,
            r#"[61/141] - "00005.m2ts" yEnc (1/101) 71995392"#,
            r#"[62/141] - "00006.m2ts" yEnc (1/101) 71995392"#,
            r#"[63/141] - "00007.m2ts" yEnc (1/240) 171515904"#,
            r#"[64/141] - "00008.m2ts" yEnc (1/11) 7606272"#,
            r#"[65/141] - "00009.m2ts" yEnc (1/308) 220569600"#,
            r#"[66/141] - "00010.m2ts" yEnc (1/281) 201308160"#,
            r#"[67/141] - "00011.m2ts" yEnc (1/13) 8742912"#,
            r#"[68/141] - "00012.m2ts" yEnc (1/3) 2033664"#,
            r#"[69/141] - "00013.m2ts" yEnc (1/2) 1062912"#,
            r#"[70/141] - "00014.m2ts" yEnc (1/1) 110592"#,
            r#"[71/141] - "index.bdmv" yEnc (1/1) 280"#,
        ];

        let mut randomized = control.clone();

        let mut rng = rand::rng();
        randomized.shuffle(&mut rng);

        let mut sorted = randomized.clone();
        sorted.sort_by_key(|s| sort_key_from_subject(s));

        assert_ne!(randomized, control);
        assert_eq!(sorted, control);
    }
}
