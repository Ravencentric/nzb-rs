/*!
nzb-rs
========

`nzb-rs` is a [spec](https://sabnzbd.org/wiki/extra/nzb-spec) compliant parser for [NZB](https://en.wikipedia.org/wiki/NZB) files.

This library is a partial port of the Python [`nzb`](https://pypi.org/project/nzb/) library.
While the parser API is similar to the original, it is not identical. Additionally, the meta editor API has not been implemented.

The heart of this library is the [`NZB::parse`] method, which handles parsing NZB files into structured data.

## Installation

To install `nzb-rs`, add the following to your `Cargo.toml`:

```toml
[dependencies]
nzb-rs = "0.1.1"
```

Optional features:

- `serde`: Enables serialization and deserialization via [serde](https://crates.io/crates/serde).

## Example

```rust
use nzb_rs::{InvalidNZBError, NZB};

fn main() -> Result<(), InvalidNZBError> {
    let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
        <nzb
            xmlns="http://www.newzbin.com/DTD/2003/nzb">
            <file poster="John &lt;nzb@nowhere.example&gt;" date="1706440708" subject="[1/1] - &quot;Big Buck Bunny - S01E01.mkv&quot; yEnc (1/2) 1478616">
                <groups>
                    <group>alt.binaries.boneless</group>
                </groups>
                <segments>
                    <segment bytes="739067" number="1">9cacde4c986547369becbf97003fb2c5-9483514693959@example</segment>
                    <segment bytes="739549" number="2">70a3a038ce324e618e2751e063d6a036-7285710986748@example</segment>
                </segments>
            </file>
        </nzb>
        "#;
    let nzb = NZB::parse(xml)?;
    println!("{:#?}", nzb);
    assert_eq!(nzb.file().name(), Some("Big Buck Bunny - S01E01.mkv"));
    Ok(())
}

```
*/

mod parser;

use crate::parser::{parse_files, parse_metadata, sabnzbd_is_obfuscated, sanitize_xml};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use lazy_regex::regex;
use roxmltree::Document;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An error type representing an invalid NZB.
#[derive(Error, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("{message}")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InvalidNZBError {
    /// Error message describing why the NZB is invalid.
    pub message: String,
}
impl InvalidNZBError {
    /// Creates a new `InvalidNZBError` with the given error message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

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
    pub datetime: DateTime<Utc>,
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
        datetime: impl Into<DateTime<Utc>>,
        subject: impl Into<String>,
        groups: impl IntoIterator<Item = impl Into<String>>,
        segments: impl IntoIterator<Item = Segment>,
    ) -> Self {
        Self {
            poster: poster.into(),
            datetime: datetime.into(),
            subject: subject.into(),
            groups: groups.into_iter().map(Into::into).collect(),
            segments: segments.into_iter().collect(),
        }
    }

    /// Size of the file calculated from the sum of segment sizes.
    pub fn size(&self) -> u64 {
        self.segments.iter().map(|x| u64::from(x.size)).sum::<u64>()
    }

    /// Complete name of the file with it's extension extracted from the subject.
    /// May return [`None`] if it fails to extract the name.
    pub fn name(&self) -> Option<&str> {
        let re_subject_filename_quotes = regex!(r#""([^"]*)""#);
        let re_subject_basic_filename =
            regex!(r"\b([\w\-+()' .,]+(?:\[[\w\-/+()' .,]*][\w\-+()' .,]*)*\.[A-Za-z0-9]{2,4})\b");

        if let Some(captured) = re_subject_filename_quotes.captures(&self.subject) {
            return captured.get(1).map(|m| m.as_str().trim());
        }

        if let Some(captured) = re_subject_basic_filename.captures(&self.subject) {
            return captured.get(1).map(|m| m.as_str().trim());
        }
        None
    }

    /// Base name of the file without it's extension extracted from the [`File::name`].
    /// May return [`None`] if it fails to extract the stem.
    pub fn stem(&self) -> Option<&str> {
        if let Some(name) = self.name() {
            return Path::new(name).file_stem().map(|f| f.to_str())?;
        }
        None
    }

    ///  Extension of the file extracted from the [`File::name`].
    /// May return [`None`] if it fails to extract the extension.
    pub fn extension(&self) -> Option<&str> {
        if let Some(name) = self.name() {
            return Path::new(name).extension().map(|f| f.to_str())?;
        }
        None
    }

    /// Return [`true`] if the file is a `.par2` file, [`false`] otherwise.
    pub fn is_par2(&self) -> bool {
        let re = regex!(r"\.par2$"i);
        self.name().is_some_and(|name| re.is_match(name))
    }

    /// Return [`true`] if the file is a `.rar` file, [`false`] otherwise.
    pub fn is_rar(&self) -> bool {
        let re = regex!(r"(\.rar|\.r\d\d|\.s\d\d|\.t\d\d|\.u\d\d|\.v\d\d)$"i);
        self.name().is_some_and(|name| re.is_match(name))
    }

    /// Return [`true`] if the file is obfuscated, [`false`] otherwise.
    pub fn is_obfuscated(&self) -> bool {
        if let Some(stem) = self.stem() {
            return sabnzbd_is_obfuscated(stem);
        }
        // Definitely obfuscated if we can't even extract the stem.
        true
    }
}

/// Represents an NZB.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NZB {
    /// Optional creator-definable metadata for the contents of the NZB.
    pub meta: Meta,
    /// File objects representing the files included in the NZB.
    pub files: Vec<File>,
}

impl FromStr for NZB {
    type Err = InvalidNZBError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let xml = sanitize_xml(s);
        let nzb = match Document::parse(xml) {
            Ok(doc) => doc,
            Err(err) => return Err(InvalidNZBError::new(err.to_string())),
        };
        let meta = parse_metadata(&nzb);
        let files = match parse_files(&nzb) {
            Ok(files) => files,
            Err(err) => return Err(InvalidNZBError::new(err.to_string())),
        };
        Ok(NZB { meta, files })
    }
}

impl NZB {
    /// Parses a string into an [`NZB`] instance.
    ///
    /// # Errors
    ///
    /// This function returns an [`InvalidNZBError`] in the following cases:
    /// - If the XML is malformed and cannot be parsed.
    /// - If the NZB structure is invalid or missing required components.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nzb_rs::{InvalidNZBError, NZB};
    ///
    /// fn main() -> Result<(), InvalidNZBError> {
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
    ///     let nzb = NZB::parse(xml)?;
    ///     println!("{:#?}", nzb);
    ///     assert_eq!(nzb.file().name(), Some("Big Buck Bunny - S01E01.mkv"));
    ///     Ok(())
    /// }
    /// ```
    pub fn parse(xml: impl AsRef<str>) -> Result<Self, InvalidNZBError> {
        Self::from_str(xml.as_ref())
    }

    /// The main content file (episode, movie, etc) in the NZB.
    /// This is determined by finding the largest file in the NZB
    /// and may not always be accurate.
    pub fn file(&self) -> &File {
        // self.files is guranteed to have atleast one file, so we can safely unwrap().
        self.files.iter().max_by_key(|file| file.size()).unwrap()
    }

    /// Total size of all the files in the NZB.
    pub fn size(&self) -> u64 {
        self.files.iter().map(|file| file.size()).sum()
    }

    /// Vector of unique file names across all the files in the NZB.
    pub fn filenames(&self) -> Vec<&str> {
        self.files.iter().filter_map(|f| f.name()).unique().sorted().collect()
    }

    /// Vector of unique posters across all the files in the NZB.
    pub fn posters(&self) -> Vec<&str> {
        self.files.iter().map(|f| f.poster.as_str()).unique().sorted().collect()
    }

    /// Vector of unique groups across all the files in the NZB.
    pub fn groups(&self) -> Vec<&str> {
        self.files
            .iter()
            .flat_map(|f| f.groups.iter().map(|f| f.as_str()))
            .unique()
            .sorted()
            .collect()
    }

    /// Total size of all the `.par2` files.
    pub fn par2_size(&self) -> u64 {
        self.files.iter().filter(|f| f.is_par2()).map(|file| file.size()).sum()
    }

    /// Percentage of the size of all the `.par2` files relative to the total size.
    pub fn par2_percentage(&self) -> f64 {
        (self.par2_size() as f64 / self.size() as f64) * 100.0
    }

    /// Return [`true`] if there's at least one `.par2` file in the NZB, [`false`] otherwise.
    pub fn has_par2(&self) -> bool {
        self.files.iter().any(|file| file.is_par2())
    }

    /// Return [`true`] if any file in the NZB is a `.rar` file, [`false`] otherwise.
    pub fn has_rar(&self) -> bool {
        self.files.iter().any(|file| file.is_rar())
    }

    /// Return [`true`] if every file in the NZB is a `.rar` file, [`false`] otherwise.
    pub fn is_rar(&self) -> bool {
        self.files.iter().all(|file| file.is_rar())
    }

    /// Return [`true`] if any file in the NZB is obfuscated, [`false`] otherwise.
    pub fn is_obfuscated(&self) -> bool {
        self.files.iter().any(|file| file.is_obfuscated())
    }
}
