#![doc = include_str!("../README.md")]

mod errors;
mod parser;
mod subparsers;

pub use crate::errors::{FileAttributeKind, ParseNzbError, ParseNzbFileError};
use crate::parser::{parse_files, parse_metadata, sabnzbd_is_obfuscated, sanitize_xml};
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use itertools::Itertools;
use lazy_regex::regex;
use roxmltree::Document;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents optional creator-definable metadata in an NZB.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Meta {
    pub title: Option<String>,
    pub passwords: Vec<String>,
    pub tags: Vec<String>,
    pub category: Option<String>,
}

impl Meta {
    /// Creates a new `Meta` instance.
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
}

/// Represents a single segment of a file in an NZB.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Segment {
    /// Size of the segment in bytes.
    pub size: u32,
    /// Number of the segment.
    pub number: u32,
    /// Message ID of the segment.
    pub message_id: String,
}

impl Segment {
    /// Creates a new `Segment` instance.
    pub fn new(size: impl Into<u32>, number: impl Into<u32>, message_id: impl Into<String>) -> Self {
        Self {
            size: size.into(),
            number: number.into(),
            message_id: message_id.into(),
        }
    }
}

/// Represents a complete file, consisting of segments that make up a file.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct File {
    /// The poster of the file.
    pub poster: String,
    /// The date and time when the file was posted, in UTC.
    pub posted_at: DateTime<Utc>,
    /// The subject of the file.
    pub subject: String,
    /// Groups that reference the file.
    pub groups: Vec<String>,
    /// Segments that make up the file.
    pub segments: Vec<Segment>,
}

impl File {
    /// Creates a new `File` instance.
    pub fn new(
        poster: impl Into<String>,
        posted_at: impl Into<DateTime<Utc>>,
        subject: impl Into<String>,
        groups: impl IntoIterator<Item = impl Into<String>>,
        segments: impl IntoIterator<Item = Segment>,
    ) -> Self {
        Self {
            poster: poster.into(),
            posted_at: posted_at.into(),
            subject: subject.into(),
            groups: groups.into_iter().map(Into::into).collect(),
            segments: segments.into_iter().collect(),
        }
    }

    /// Size of the file calculated from the sum of segment sizes.
    #[must_use]
    pub fn size(&self) -> u64 {
        self.segments.iter().map(|x| u64::from(x.size)).sum::<u64>()
    }

    /// Complete name of the file with it's extension extracted from the subject.
    /// May return [`None`] if it fails to extract the name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        subparsers::extract_filename_from_subject(&self.subject)
    }

    /// Base name of the file without it's extension extracted from the [`File::name`].
    /// May return [`None`] if it fails to extract the stem.
    #[must_use]
    pub fn stem(&self) -> Option<&str> {
        self.name().map(|name| {
            let (stem, _) = subparsers::split_filename_at_extension(name);
            stem
        })
    }

    ///  Extension of the file extracted from the [`File::name`].
    /// May return [`None`] if it fails to extract the extension.
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.name().and_then(|name| {
            let (_, ext) = subparsers::split_filename_at_extension(name);
            ext
        })
    }

    /// Return [`true`] if the file has the specified extension, [`false`] otherwise.
    ///
    /// This method ensures consistent extension comparison
    /// by normalizing the extension (removing any leading dot) and handling case-folding.
    pub fn has_extension(&self, ext: impl AsRef<str>) -> bool {
        let ext = ext.as_ref().strip_prefix('.').unwrap_or_else(|| ext.as_ref()).trim();
        self.extension()
            .is_some_and(|file_ext| file_ext.eq_ignore_ascii_case(ext))
    }

    /// Return [`true`] if the file is a `.par2` file, [`false`] otherwise.
    #[must_use]
    pub fn is_par2(&self) -> bool {
        let re = regex!(r"\.par2$"i);
        self.name().is_some_and(|name| re.is_match(name))
    }

    /// Return [`true`] if the file is a `.rar` file, [`false`] otherwise.
    #[must_use]
    pub fn is_rar(&self) -> bool {
        // https://github.com/sabnzbd/sabnzbd/blob/02b4a116dd4b46b2d2f33f7bbf249f2294458f2e/sabnzbd/nzbstuff.py#L107
        let re = regex!(r"(\.rar|\.r\d\d|\.s\d\d|\.t\d\d|\.u\d\d|\.v\d\d)$"i);
        self.name().is_some_and(|name| re.is_match(name))
    }

    /// Return [`true`] if the file is obfuscated, [`false`] otherwise.
    pub fn is_obfuscated(&self) -> bool {
        self.stem().is_none_or(sabnzbd_is_obfuscated)
    }
}

/// Represents an NZB.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Nzb {
    /// Optional creator-definable metadata for the contents of the NZB.
    pub meta: Meta,
    /// File objects representing the files included in the NZB.
    pub files: Vec<File>,
}

impl FromStr for Nzb {
    type Err = ParseNzbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let xml = sanitize_xml(s);
        let nzb = Document::parse(xml)?;
        let meta = parse_metadata(&nzb);
        let files = parse_files(&nzb)?;
        Ok(Self { meta, files })
    }
}

impl Nzb {
    /// Parses a string into an [`Nzb`] instance.
    ///
    /// # Errors
    ///
    /// This function returns an [`ParseNzbError`] in the following cases:
    /// - If the XML is malformed and cannot be parsed.
    /// - If the NZB structure is invalid or missing required components.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nzb_rs::{ParseNzbError, Nzb};
    ///
    /// fn main() -> Result<(), ParseNzbError> {
    ///     let xml = r#"
    ///         <?xml version="1.0" encoding="UTF-8"?>
    ///         <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
    ///         <nzb
    ///             xmlns="http://www.newzbin.com/DTD/2003/nzb">
    ///             <file poster="John &lt;nzb@nowhere.example&gt;" date="1706440708" subject="[1/1] - &quot;Big Buck Bunny - S01E01.mkv&quot; yEnc (1/2) 1478616">
    ///                 <groups>
    ///                     <group>alt.binaries.boneless</group>
    ///                 </groups>
    ///                 <segments>
    ///                     <segment bytes="739067" number="1">9cacde4c986547369becbf97003fb2c5-9483514693959@example</segment>
    ///                     <segment bytes="739549" number="2">70a3a038ce324e618e2751e063d6a036-7285710986748@example</segment>
    ///                 </segments>
    ///             </file>
    ///         </nzb>
    ///         "#;
    ///     let nzb = Nzb::parse(xml)?;
    ///     println!("{:#?}", nzb);
    ///     assert_eq!(nzb.file().name(), Some("Big Buck Bunny - S01E01.mkv"));
    ///     Ok(())
    /// }
    /// ```
    pub fn parse(nzb: impl AsRef<str>) -> Result<Self, ParseNzbError> {
        nzb.as_ref().parse()
    }

    /// Parse a file into an [`Nzb`] instance.
    /// Handles both regular and gzipped NZB files.
    ///
    /// # Errors
    ///
    /// This function returns an [`ParseNzbFileError`] in the following cases:
    /// - If the file cannot be read.
    /// - If the contents of the file are malformed and cannot be parsed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nzb_rs::{ParseNzbFileError, Nzb};
    ///
    /// fn main() -> Result<(), ParseNzbFileError> {
    ///     let nzb = Nzb::parse_file("tests/nzbs/big_buck_bunny.nzb")?;
    ///     println!("{:#?}", nzb);
    ///     assert_eq!(nzb.file().name(), Some("Big Buck Bunny - S01E01.mkv"));
    ///     Ok(())
    /// }
    /// ```
    pub fn parse_file(nzb: impl AsRef<Path>) -> Result<Self, ParseNzbFileError> {
        let file =
            dunce::canonicalize(nzb.as_ref()).map_err(|source| ParseNzbFileError::from_io_err(source, nzb.as_ref()))?;

        let content = if file.extension().is_some_and(|f| f.eq_ignore_ascii_case("gz")) {
            let gzipped = fs::read(&file).map_err(|source| ParseNzbFileError::from_gzip_err(source, file.clone()))?;
            let mut decoder = GzDecoder::new(&gzipped[..]);
            let mut content = String::new();
            decoder
                .read_to_string(&mut content)
                .map_err(|source| ParseNzbFileError::from_gzip_err(source, file))?;
            content
        } else {
            fs::read_to_string(&file).map_err(|source| ParseNzbFileError::from_io_err(source, file.clone()))?
        };

        Ok(Self::parse(content)?)
    }

    /// The main content file (episode, movie, etc) in the NZB.
    /// This is determined by finding the largest non `par2` file in the NZB
    /// and may not always be accurate.
    #[must_use]
    pub fn file(&self) -> &File {
        // self.files is guaranteed to have at least one file.
        self.files
            .iter()
            .filter(|file| !file.is_par2())
            .max_by_key(|file| file.size())
            .expect("NZB should have at least one non-`.par2` file")
    }

    /// Total size of all the files in the NZB.
    #[must_use]
    pub fn size(&self) -> u64 {
        self.files.iter().map(File::size).sum()
    }

    /// Vector of unique file names across all the files in the NZB.
    #[must_use]
    pub fn filenames(&self) -> Vec<&str> {
        self.files.iter().filter_map(|f| f.name()).unique().sorted().collect()
    }

    /// Vector of unique posters across all the files in the NZB.
    #[must_use]
    pub fn posters(&self) -> Vec<&str> {
        self.files.iter().map(|f| f.poster.as_str()).unique().sorted().collect()
    }

    /// Vector of unique groups across all the files in the NZB.
    #[must_use]
    pub fn groups(&self) -> Vec<&str> {
        self.files
            .iter()
            .flat_map(|f| f.groups.iter().map(String::as_str))
            .unique()
            .sorted()
            .collect()
    }

    /// Vector of `.par2` files in the NZB.
    #[must_use]
    pub fn par2_files(&self) -> Vec<&File> {
        self.files.iter().filter(|f| f.is_par2()).collect()
    }

    /// Total size of all the `.par2` files.
    #[must_use]
    pub fn par2_size(&self) -> u64 {
        self.files
            .iter()
            .filter_map(|f| if f.is_par2() { Some(f.size()) } else { None })
            .sum()
    }

    /// Percentage of the size of all the `.par2` files relative to the total size.
    #[must_use]
    pub fn par2_percentage(&self) -> f64 {
        (self.par2_size() as f64 / self.size() as f64) * 100.0
    }

    /// Return [`true`] if any file in the NZB has the specified extension, [`false`] otherwise.
    ///
    /// This method ensures consistent extension comparison
    /// by normalizing the extension (removing any leading dot) and handling case-folding.
    pub fn has_extension(&self, ext: impl AsRef<str>) -> bool {
        self.files.iter().any(|f| f.has_extension(ext.as_ref()))
    }

    /// Return [`true`] if there's at least one `.par2` file in the NZB, [`false`] otherwise.
    #[must_use]
    pub fn has_par2(&self) -> bool {
        self.files.iter().any(File::is_par2)
    }

    /// Return [`true`] if any file in the NZB is a `.rar` file, [`false`] otherwise.
    #[must_use]
    pub fn has_rar(&self) -> bool {
        self.files.iter().any(File::is_rar)
    }

    /// Return [`true`] if every file in the NZB is a `.rar` file, [`false`] otherwise.
    #[must_use]
    pub fn is_rar(&self) -> bool {
        self.files.iter().all(File::is_rar)
    }

    /// Return [`true`] if any file in the NZB is obfuscated, [`false`] otherwise.
    #[must_use]
    pub fn is_obfuscated(&self) -> bool {
        self.files.iter().any(File::is_obfuscated)
    }
}
