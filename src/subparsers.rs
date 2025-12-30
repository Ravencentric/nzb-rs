use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

/// Splits a string once on `delimiter`, trimming whitespace from both results.
///
/// Returns `None` if the delimiter is not present.
fn split_once_trimmed<'a>(s: &'a str, delimiter: &str) -> Option<(&'a str, &'a str)> {
    let (left, right) = s.split_once(delimiter)?;
    Some((left.trim(), right.trim()))
}

/// Returns `true` if the string represents a valid unsigned integer.
///
/// Leading and trailing whitespace is ignored.
fn is_number(s: &str) -> bool {
    s.trim().parse::<u64>().is_ok()
}

/// Returns `true` if the string matches a multipart counter format.
///
/// Accepted forms are `[x/y]` or `(x/y)`, where both values must be numeric.
fn is_multipart_counter(s: &str) -> bool {
    let mut chars = s.trim().chars();

    let close = match chars.next() {
        Some('[') => ']',
        Some('(') => ')',
        _ => return false,
    };

    if chars.next_back() != Some(close) {
        return false;
    }

    let interior = chars.as_str();

    if let Some((left, right)) = interior.split_once('/') {
        is_number(left) && is_number(right)
    } else {
        false
    }
}

/// Attempts to extract a filename (including extension) from the subject.
///
/// Returns `None` if no filename can be identified.
///
/// This function is based on SABnzbd’s [`subject_name_extractor`],
/// but is not an exact port and intentionally diverges in some cases.
///
/// Notably, it returns `None` when no valid filename is found,
/// whereas SABnzbd’s [`subject_name_extractor`] returns the original subject string.
///
/// [`subject_name_extractor`]: https://github.com/sabnzbd/sabnzbd/blob/b5dda7c52d9055a3557e7f5fc6e76fe86c4c4365/sabnzbd/misc.py#L1642-L1655
pub(crate) fn extract_filename_from_subject(subject: &str) -> Option<&str> {
    // The extraction logic is intentionally ordered from most specific to most
    // general to avoid false positives.

    // ---------------------------------------------------------------------
    // Case 1: Filename enclosed in quotes
    // ---------------------------------------------------------------------
    //
    // Based on SABnzbd’s [`RE_SUBJECT_FILENAME_QUOTES`], but
    // slightly more relaxed, as used in [`subject_name_extractor`].
    //
    // [`RE_SUBJECT_FILENAME_QUOTES`]: https://github.com/sabnzbd/sabnzbd/blob/02b4a116dd4b46b2d2f33f7bbf249f2294458f2e/sabnzbd/nzbstuff.py#L104
    // [`subject_name_extractor`]: https://github.com/sabnzbd/sabnzbd/blob/02b4a116dd4b46b2d2f33f7bbf249f2294458f2e/sabnzbd/nzbstuff.py#L2170-L2172
    if let Some(start) = subject.find('"')
        && let Some(end) = subject.rfind('"')
    {
        let start = start + 1;
        if start < end {
            let s = subject[start..end].trim_matches(|c: char| c.is_whitespace() || c == '"');
            if !s.is_empty() {
                return Some(s);
            }
        }
    }

    // ---------------------------------------------------------------------
    // Case 2: Structured multipart yEnc subject
    // ---------------------------------------------------------------------
    //
    // This matches a common Usenet pattern of:
    //   [part/total] - <filename> yEnc (chunk/total) <digits>
    //
    // Example:
    //   [011/116] - [Foobar] Violet Evergarden - 01.mkv yEnc (1/2401) 1720916370
    //
    // Parsed as:
    // - part: "[011/116]"
    // - filename: "[Foobar] Violet Evergarden - 01.mkv"
    // - chunk: "(1/2401)"
    // - trailing digits: "1720916370"
    if let Some((part, rest)) = split_once_trimmed(subject, "-")
        && is_multipart_counter(part)
        && let Some((filename, rest)) = split_once_trimmed(rest, "yEnc")
        && let Some((chunk, digits)) = split_once_trimmed(rest, " ")
        && is_multipart_counter(chunk)
        && is_number(digits)
    {
        let trimmed = filename.trim();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    // ---------------------------------------------------------------------
    // Case 3: Best-effort filename extraction
    // ---------------------------------------------------------------------
    //
    // The regex used here is a direct port of SABnzbd’s
    // [`RE_SUBJECT_BASIC_FILENAME`], as used in
    // [`subject_name_extractor`].
    //
    // [`RE_SUBJECT_BASIC_FILENAME`]: https://github.com/sabnzbd/sabnzbd/blob/b5dda7c52d9055a3557e7f5fc6e76fe86c4c4365/sabnzbd/misc.py#L90
    // [`subject_name_extractor`]: https://github.com/sabnzbd/sabnzbd/blob/b5dda7c52d9055a3557e7f5fc6e76fe86c4c4365/sabnzbd/misc.py#L1650-L1652
    static SABNZBD_SUBJECT_BASIC_FILENAME: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\b([\w\-+()' .,]+(?:\[[\w\-/+()' .,]*][\w\-+()' .,]*)*\.[A-Za-z0-9]{2,4})\b").unwrap()
    });

    for matched in SABNZBD_SUBJECT_BASIC_FILENAME.find_iter(subject) {
        let trimmed = matched.as_str().trim();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    None
}

/// Returns the file extension, if any.
///
/// This is a small wrapper around [`Path::extension`] that *attempts* to filter
/// out common false positives found in P2P and obfuscated file or folder names
/// (where dots are often used without indicating an actual extension).
///
/// The returned extension does not include the leading dot.
pub(crate) fn file_extension(name: &str) -> Option<&str> {
    let ext = Path::new(name).extension()?.to_str()?;

    const COMMON_EXTENSION_MAX_LEN: usize = 8;

    if ext.len() > COMMON_EXTENSION_MAX_LEN {
        return None;
    }

    if !ext.as_bytes().iter().all(u8::is_ascii_alphanumeric) {
        return None;
    }

    Some(ext)
}

/// Returns the file stem (the name without its extension).
///
/// If no extension is detected, the entire `name` is returned as the stem.
pub(crate) fn file_stem(name: &str) -> &str {
    file_extension(name).map_or(name, |ext| {
        // SAFETY:
        // [`file_extension`] relies on [`Path::extension`], which guarantees `ext`
        // comes after a final `.` that is not the first character. Therefore,
        // slicing off `.<ext>` always produces a valid subslice.
        &name[..name.len() - ext.len() - 1]
    })
}

/// Extracts a numeric prefix from subjects formatted like "[N/...]".
///
/// This is used to avoid lexicographic sorting errors when numbers are not
/// zero-padded (e.g. "[1/...]", "[11/...]", "[2/...]").
///
/// Not all subjects include the "[N/...]" pattern, so the original subject is
/// always returned unchanged and the numeric key may be absent.
///
/// # Example
/// Input: "[27/141] - "index.bdmv" yEnc (1/1) 280"
/// Output: (Some(27), "[27/141] - "index.bdmv" yEnc (1/1) 280")
pub(crate) fn sort_key_from_subject(subject: &str) -> (Option<u32>, &str) {
    let num = subject
        .strip_prefix('[')
        .and_then(|s| s.split_once('/'))
        .and_then(|(digits, _)| digits.parse().ok());

    (num, subject)
}

#[cfg(test)]
mod tests {
    use rand::seq::SliceRandom;
    use rstest::rstest;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct NameParts<'a> {
        filename: Option<&'a str>,
        stem: Option<&'a str>,
        extension: Option<&'a str>,
    }

    #[rstest]
    #[case(
        r#"Great stuff (001/143) - "Filename.txt" yEnc (1/1)"#,
        NameParts {
            filename: Some("Filename.txt"),
            stem: Some("Filename"),
            extension: Some("txt"),
        }
    )]
    #[case(
        r#""910a284f98ebf57f6a531cd96da48838.vol01-03.par2" yEnc (1/3)"#,
        NameParts {
            filename: Some("910a284f98ebf57f6a531cd96da48838.vol01-03.par2"),
            stem: Some("910a284f98ebf57f6a531cd96da48838.vol01-03"),
            extension: Some("par2"),
        }
    )]
    #[case(
        r#"Subject-KrzpfTest [02/30] - ""KrzpfTest.part.nzb"" yEnc"#,
        NameParts {
            filename: Some("KrzpfTest.part.nzb"),
            stem: Some("KrzpfTest.part"),
            extension: Some("nzb"),
        }
    )]
    #[case(
        r#"[PRiVATE]-[WtFnZb]-[Supertje-_S03E11-12_-blabla_+_blabla_WEBDL-480p.mkv]-[4/12] - "" yEnc 9786 (1/1366)"#,
        NameParts {
            filename: Some("Supertje-_S03E11-12_-blabla_+_blabla_WEBDL-480p.mkv"),
            stem: Some("Supertje-_S03E11-12_-blabla_+_blabla_WEBDL-480p"),
            extension: Some("mkv"),
        }
    )]
    #[case(
        r#"[N3wZ] MAlXD245333\\::[PRiVATE]-[WtFnZb]-[Show.S04E04.720p.AMZN.WEBRip.x264-GalaxyTV.mkv]-[1/2] - "" yEnc  293197257 (1/573)"#,
        NameParts {
            filename: Some("Show.S04E04.720p.AMZN.WEBRip.x264-GalaxyTV.mkv"),
            stem: Some("Show.S04E04.720p.AMZN.WEBRip.x264-GalaxyTV"),
            extension: Some("mkv"),
        }
    )]
    #[case(
        r#"reftestnzb bf1664007a71 [1/6] - "20b9152c-57eb-4d02-9586-66e30b8e3ac2" yEnc (1/22) 15728640"#,
        NameParts {
            filename: Some("20b9152c-57eb-4d02-9586-66e30b8e3ac2"),
            stem: Some("20b9152c-57eb-4d02-9586-66e30b8e3ac2"),
            extension: None,
        }
    )]
    #[case(
        r#"Re: REQ Author Child's The Book-Thanks much - Child, Lee - Author - The Book.epub (1/1)"#,
        NameParts {
            filename: Some("REQ Author Child's The Book-Thanks much - Child, Lee - Author - The Book.epub"),
            stem: Some("REQ Author Child's The Book-Thanks much - Child, Lee - Author - The Book"),
            extension: Some("epub"),
        }
    )]
    #[case(
        r#"63258-0[001/101] - "63258-2.0" yEnc (1/250) (1/250)"#,
        NameParts {
            filename: Some("63258-2.0"),
            stem: Some("63258-2"),
            extension: Some("0"),
        }
    )]
    #[case(
        r#"63258-0[001/101] - "63258-2.0toolong" yEnc (1/250) (1/250)"#,
        NameParts {
            filename: Some("63258-2.0toolong"),
            stem: Some("63258-2"),
            extension: Some("0toolong"),
        }
    )]
    #[case(
        r#"Singer - A Album (2005) - [04/25] - 02 Sweetest Somebody (I Know).flac"#,
        NameParts {
            filename: Some("Singer - A Album (2005) - [04/25] - 02 Sweetest Somebody (I Know).flac"),
            stem: Some("Singer - A Album (2005) - [04/25] - 02 Sweetest Somebody (I Know)"),
            extension: Some("flac"),
        }
    )]
    #[case(
        "<>random!>",
        NameParts {
            filename: None,
            stem: None,
            extension: None,
        }
    )]
    #[case(
        "nZb]-[Supertje-_S03E11-12_",
        // We intentionally diverge from SABnzbd's behavior here, as it would
        // return the subject when it fails to extract a valid filename.
        // Since idiomatic Rust favors Option types for such cases, we return None.
        // This makes it clearer to the caller that no valid filename was found.
        // Test Case: https://github.com/sabnzbd/sabnzbd/blob/a637d218c40af29279468a17a1e3ee2dbc976557/tests/test_misc.py#L904C15-L904C41
        // Function Definition: https://github.com/sabnzbd/sabnzbd/blob/a637d218c40af29279468a17a1e3ee2dbc976557/sabnzbd/misc.py#L1642-L1655
        NameParts {
            filename: None, // Sabnzbd: "nZb]-[Supertje-_S03E11-12_"
            stem: None,
            extension: None,
        }
    )]
    #[case(
        r#"Bla [Now it's done.exe]"#,
        NameParts {
            filename: Some("Now it's done.exe"),
            stem: Some("Now it's done"),
            extension: Some("exe"),
        }
    )]
    #[case(
        r#"Bla [Now it's done.123nonsense]"#,
        NameParts {
            filename: None,
            stem: None,
            extension: None,
        }
    )]
    #[case(
        r#"[PRiVATE]-[WtFnZb]-[00000.clpi]-[1/46] - "" yEnc  788 (1/1)"#,
        NameParts {
            filename: Some("00000.clpi"),
            stem: Some("00000"),
            extension: Some("clpi"),
        }
    )]
    #[case(
        r#"[PRiVATE]-[WtFnZb]-[Video_(2001)_AC5.1_-RELEASE_[TAoE].mkv]-[1/23] - "" yEnc 1234567890 (1/23456)"#,
        NameParts {
            filename: Some("Video_(2001)_AC5.1_-RELEASE_[TAoE].mkv"),
            stem: Some("Video_(2001)_AC5.1_-RELEASE_[TAoE]"),
            extension: Some("mkv"),
        }
    )]
    #[case(
        r#"[PRiVATE]-[WtFnZb]-[219]-[1/series.name.s01e01.1080p.web.h264-group.mkv] -  yEnc (1/[PRiVATE] \\c2b510b594\\::686ea969999193.155368eba4965e56a8cd263382e012.f2712fdc::/97bd201cf931/) 1 (1/0)"#,
        NameParts {
            filename: Some("series.name.s01e01.1080p.web.h264-group.mkv"),
            stem: Some("series.name.s01e01.1080p.web.h264-group"),
            extension: Some("mkv"),
        }
    )]
    #[case(
        r#"[PRiVATE]-[WtFnZb]-[/More.Bla.S02E01.1080p.WEB.h264-EDITH[eztv.re].mkv-WtF[nZb]/More.Bla.S02E01.1080p.WEB.h264-EDITH.mkv]-[1/2] - "" yEnc  2990558544 (1/4173)"#,
        NameParts {
            filename: Some("More.Bla.S02E01.1080p.WEB.h264-EDITH[eztv.re].mkv"),
            stem: Some("More.Bla.S02E01.1080p.WEB.h264-EDITH[eztv.re]"),
            extension: Some("mkv"),
        }
    )]
    #[case(
        "[011/116] - [AC-FFF] Highschool DxD BorN - 02 [BD][1080p-Hi10p] FLAC][Dual-Audio][442E5446].mkv yEnc (1/2401) 1720916370",
        NameParts {
            filename: Some("[AC-FFF] Highschool DxD BorN - 02 [BD][1080p-Hi10p] FLAC][Dual-Audio][442E5446].mkv"),
            stem: Some("[AC-FFF] Highschool DxD BorN - 02 [BD][1080p-Hi10p] FLAC][Dual-Audio][442E5446]"),
            extension: Some("mkv"),
        }
    )]
    #[case(
        "[010/108] - [SubsPlease] Ijiranaide, Nagatoro-san - 02 (1080p) [6E8E8065].mkv yEnc (1/2014) 1443366873",
        NameParts {
            filename: Some("[SubsPlease] Ijiranaide, Nagatoro-san - 02 (1080p) [6E8E8065].mkv"),
            stem: Some("[SubsPlease] Ijiranaide, Nagatoro-san - 02 (1080p) [6E8E8065]"),
            extension: Some("mkv"),
        }
    )]
    #[case(
        r#"[1/8] - "TenPuru - No One Can Live on Loneliness v05 {+ "Book of Earthly Desires" pamphlet} (2021) (Digital) (KG Manga).cbz" yEnc (1/230) 164676947"#,
        NameParts {
            filename: Some(r#"TenPuru - No One Can Live on Loneliness v05 {+ "Book of Earthly Desires" pamphlet} (2021) (Digital) (KG Manga).cbz"#),
            stem: Some(r#"TenPuru - No One Can Live on Loneliness v05 {+ "Book of Earthly Desires" pamphlet} (2021) (Digital) (KG Manga)"#),
            extension: Some("cbz"),
        }
    )]
    #[case(
        r#"[1/10] - "ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG" yEnc (1/1277) 915318101"#,
        NameParts {
            filename: Some("ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG"),
            stem: Some("ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG"),
            extension: None,
        }
    )]
    #[case(
        r#"[1/10] - "ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG.mkv" yEnc (1/1277) 915318101"#,
        NameParts {
            filename: Some("ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG.mkv"),
            stem: Some("ONE.PIECE.S01E1109.1080p.NF.WEB-DL.AAC2.0.H.264-VARYG"),
            extension: Some("mkv"),
        }
    )]
    #[case(
        r#"[27/141] - "index.bdmv" yEnc (1/1) 280"#,
        NameParts {
            filename: Some("index.bdmv"),
            stem: Some("index"),
            extension: Some("bdmv"),
        }
    )]
    fn test_name_stem_extension_extraction(#[case] subject: &str, #[case] expected: NameParts) {
        let filename = extract_filename_from_subject(subject);
        let stem = filename.map(file_stem);
        let extension = filename.and_then(file_extension);

        let actual = NameParts {
            filename,
            stem,
            extension,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_sort_key_from_subject() {
        assert_eq!(
            sort_key_from_subject(r#"[10/141] - "00010.clpi" yEnc (1/1) 1000"#),
            (Some(10), r#"[10/141] - "00010.clpi" yEnc (1/1) 1000"#)
        );

        assert_eq!(
            sort_key_from_subject(r#""00010.clpi" yEnc (1/1) 1000"#),
            (None, r#""00010.clpi" yEnc (1/1) 1000"#)
        );

        assert_eq!(
            sort_key_from_subject("Here's your file!  abc-mr2a.r01 (1/2)"),
            (None, "Here's your file!  abc-mr2a.r01 (1/2)")
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

        let mut sorted_by = randomized.clone();
        let mut sorted_by_key = randomized.clone();
        sorted_by.sort_by(|a, b| sort_key_from_subject(a).cmp(&sort_key_from_subject(b)));
        sorted_by_key.sort_by_key(|s| sort_key_from_subject(s));

        assert_ne!(randomized, control);
        assert_eq!(sorted_by, control);
        assert_eq!(sorted_by_key, control);
        assert_eq!(sorted_by, sorted_by_key);
    }
}
