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

/// Returns `true` if the string matches a counter format.
///
/// Accepted forms are `[x/y]` or `(x/y)`, where both values must be numeric.
fn is_counter(s: &str) -> bool {
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

/// Returns `true` if the string contains at least 30 consecutive hexdigits
///
/// The run must be uninterrupted, any non-hexdigit resets the count.
/// Equivalent to the regex `[a-fA-F0-9]{30}`.
fn has_30_consecutive_hexdigits(s: &str) -> bool {
    let mut run = 0;

    for char in s.as_bytes() {
        if char.is_ascii_hexdigit() {
            run += 1;
            if run >= 30 {
                return true;
            }
        } else {
            run = 0;
        }
    }

    false
}

/// Returns `true` if the string contains at least two bracketed words
///
/// Each word matches `\[\w+\]` (non-empty alphanumeric or underscore content
/// enclosed in square brackets).
fn has_two_bracketed_words(s: &str) -> bool {
    s.split('[')
        .skip(1)
        .filter_map(|part| part.split_once(']'))
        .filter(|(inside, _)| !inside.is_empty() && inside.chars().all(|c| c.is_alphanumeric() || c == '_'))
        .take(2)
        .count()
        == 2
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
pub(crate) fn file_name(subject: &str) -> Option<&str> {
    // The extraction logic is intentionally ordered from most specific to most
    // general to avoid false positives.

    // ---------------------------------------------------------------------
    // Case 1: Filename enclosed in quotes
    // ---------------------------------------------------------------------
    //
    // Based on SABnzbd’s [`RE_SUBJECT_FILENAME_QUOTES`][0], but
    // slightly more relaxed, as used in [`subject_name_extractor`][1].
    //
    // [0]: https://github.com/sabnzbd/sabnzbd/blob/448c034f79eb0c02c34d0da5e546926d7bec0d61/sabnzbd/misc.py#L90
    // [1]: https://github.com/sabnzbd/sabnzbd/blob/448c034f79eb0c02c34d0da5e546926d7bec0d61/sabnzbd/misc.py#L1623-L1628
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
        && is_counter(part)
        && let Some((filename, rest)) = split_once_trimmed(rest, "yEnc")
        && let Some((chunk, digits)) = split_once_trimmed(rest, " ")
        && is_counter(chunk)
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
    // [`RE_SUBJECT_BASIC_FILENAME`][0], as used in
    // [`subject_name_extractor`][1].
    //
    // [0]: https://github.com/sabnzbd/sabnzbd/blob/448c034f79eb0c02c34d0da5e546926d7bec0d61/sabnzbd/misc.py#L91
    // [1]: https://github.com/sabnzbd/sabnzbd/blob/448c034f79eb0c02c34d0da5e546926d7bec0d61/sabnzbd/misc.py#L1630-L1633
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

/// Return `true` if the file stem appears to be obfuscated, `false` otherwise.
///
/// Based on SABnzbd’s [`is_probably_obfuscated`][0].
///
/// Differences from the original:
/// - Expects the file *stem* (basename without extension); no path or extension
///   handling is performed. Passing anything else yields incorrect results.
/// - Implemented without regular expressions.
/// - Intended to produce identical results for equivalent inputs; any deviation
///   is considered a bug.
///
/// [0]: <https://github.com/sabnzbd/sabnzbd/blob/d21a1119932896c1f7fea1b804e99c70f05dbd19/sabnzbd/deobfuscate_filenames.py#L103>
pub(crate) fn is_obfuscated(filestem: &str) -> bool {
    // In a lot of cases, we do not care about anything other than ASCII characters.
    // So, we can work on the byte level for minor performance gains.
    let filestem_bytes = filestem.as_bytes();
    let length = filestem_bytes.len();

    // First, the patterns that are certainly obfuscated:

    // 32-character hex strings, e.g.
    // ...blabla.H.264/b082fa0beaa644d3aa01045d5b8d0b36.mkv is certainly obfuscated
    if length == 32 && filestem_bytes.iter().all(u8::is_ascii_hexdigit) {
        return true;
    }

    // 40+ chars consisting only of hex digits and dots, e.g.
    // 0675e29e9abfd2.f7d069dab0b853283cc1b069a25f82.6547
    if length >= 40 && filestem_bytes.iter().all(|b| b.is_ascii_hexdigit() || *b == b'.') {
        return true;
    }

    // "[BlaBla] something [More] something 5937bc5e32146e.bef89a622e4a23f07b0d3757ad5e8a.a02b264e [Brrr]"
    // So: square brackets plus 30+ hex digit
    if has_30_consecutive_hexdigits(filestem) && has_two_bracketed_words(filestem) {
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

/// Extracts the file number from a subject, if present.
///
/// This is used to avoid lexicographic sorting errors when numbers are not
/// zero-padded (e.g. `[1/10]`, `[2/10]`, `[10/10]`).
///
/// Not all subjects include such a number. If no valid `[current/total]` prefix is
/// found, `None` is returned.
///
/// # Example
/// Input: "[27/141] - "index.bdmv" yEnc (1/1) 280"
/// Output: Some((27, 141))
pub(crate) fn file_number(subject: &str) -> Option<(u32, u32)> {
    subject
        .strip_prefix('[')
        .and_then(|s| s.split_once(']'))
        .and_then(|(counter, _)| counter.split_once('/'))
        .and_then(|(current, total)| {
            let current = current.parse::<u32>().ok()?;
            let total = total.parse::<u32>().ok()?;
            Some((current, total))
        })
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
    // These test cases were sourced from:
    // https://github.com/sabnzbd/sabnzbd/blob/33aa4f1199371b1cf262028a0aae53d0766e82b6/tests/test_misc.py#L871-L926
    // with a few additional cases added at the end.
    //
    // ---- SABnzbd-derived test cases START ----
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
        // No valid filename can be extracted from this subject.
        // SABnzbd falls back to returning the original subject string,
        // but our implementation treats that as a failed extraction.
        //
        // Corresponding SABnzbd test:
        // https://github.com/sabnzbd/sabnzbd/blob/33aa4f1199371b1cf262028a0aae53d0766e82b6/tests/test_misc.py#L903
        NameParts {
            filename: None, // Sabnzbd: "<>random!>"
            stem: None,
            extension: None,
        }
    )]
    #[case(
        "nZb]-[Supertje-_S03E11-12_",
        // No valid filename can be extracted from this subject.
        // SABnzbd falls back to returning the original subject string,
        // but our implementation treats that as a failed extraction.
        //
        // Corresponding SABnzbd test:
        // https://github.com/sabnzbd/sabnzbd/blob/33aa4f1199371b1cf262028a0aae53d0766e82b6/tests/test_misc.py#L904
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
    // ---- SABnzbd-derived test cases END ----
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
        let filename = file_name(subject);
        let stem = filename.map(file_stem);
        let extension = filename.and_then(file_extension);

        let actual = NameParts {
            filename,
            stem,
            extension,
        };

        assert_eq!(actual, expected);
    }

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
        assert!(is_obfuscated(filestem));
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
        assert!(!is_obfuscated(filestem));
    }

    #[test]
    fn test_sort_key_from_subject() {
        assert_eq!(
            file_number(r#"[10/141] - "00010.clpi" yEnc (1/1) 1000"#),
            Some((10, 141))
        );
        assert_eq!(file_number(r#""00010.clpi" yEnc (1/1) 1000"#), None);
        assert_eq!(file_number("Here's your file!  abc-mr2a.r01 (1/2)"), None);

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
        sorted_by.sort_by(|a, b| file_number(a).cmp(&file_number(b)).then_with(|| a.cmp(b)));
        sorted_by_key.sort_by_key(|s| file_number(s));

        assert_ne!(randomized, control);
        assert_eq!(sorted_by, control);
        assert_eq!(sorted_by_key, control);
        assert_eq!(sorted_by, sorted_by_key);
    }
}
