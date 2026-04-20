use std::fs;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

use flate2::read::GzDecoder;

use crate::errors::{ParseNzbError, ParseNzbFileError};
use crate::file::File;
use crate::files::Files;
use crate::meta::Meta;
use crate::parity::Parity;
use crate::parser::parse_files;
use crate::xml;

/// Represents an NZB.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Nzb {
    meta: Meta,
    files: Files,
    parity: Parity,
}

impl FromStr for Nzb {
    type Err = ParseNzbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nzb = xml::parse_document(s)?;
        let meta = Meta::parse(&nzb);
        let (files, parity) = parse_files(&nzb)?;
        Ok(Self { meta, files, parity })
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
    /// use nzb_rs::{Nzb, ParseNzbError};
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
    ///     assert_eq!(nzb.primary().name(), Some("Big Buck Bunny - S01E01.mkv"));
    ///     assert_eq!(nzb.files().len(), 1);
    ///     assert_eq!(nzb.files().size(), 1_478_616);
    ///     assert!(nzb.parity().is_empty());
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
    /// use nzb_rs::{Nzb, ParseNzbFileError};
    ///
    /// fn main() -> Result<(), ParseNzbFileError> {
    ///     let nzb = Nzb::parse_file("tests/nzbs/big_buck_bunny.nzb")?;
    ///     assert_eq!(nzb.primary().name(), Some("Big Buck Bunny - S01E01.mkv"));
    ///     assert_eq!(nzb.files().len(), 1);
    ///     assert_eq!(nzb.files().size(), 17_521_761);
    ///     assert_eq!(nzb.parity().len(), 4);
    ///     Ok(())
    /// }
    /// ```
    pub fn parse_file(nzb: impl AsRef<Path>) -> Result<Self, ParseNzbFileError> {
        let file = nzb.as_ref();

        let content = if file.extension().is_some_and(|f| f.eq_ignore_ascii_case("gz")) {
            let gzipped = fs::read(file).map_err(|source| ParseNzbFileError::from_gzip_err(source, file))?;
            let mut decoder = GzDecoder::new(&gzipped[..]);
            let mut content = String::with_capacity(gzipped.len());
            decoder
                .read_to_string(&mut content)
                .map_err(|source| ParseNzbFileError::from_gzip_err(source, file))?;
            content
        } else {
            fs::read_to_string(file).map_err(|source| ParseNzbFileError::from_io_err(source, file))?
        };

        Ok(Self::parse(content)?)
    }
    /// Optional creator-definable metadata for the contents of the NZB.
    #[must_use]
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Read-only collection of non-`.par2` files contained in the NZB.
    ///
    /// This collection is guaranteed to contain at least one file.
    #[must_use]
    pub fn files(&self) -> &Files {
        &self.files
    }

    /// The primary content file (episode, movie, etc) in the NZB.
    /// This is determined by finding the largest non-parity file in the NZB
    /// and may not always be accurate.
    #[must_use]
    pub fn primary(&self) -> &File {
        self.files.primary()
    }

    /// Read-only collection of `.par2` files contained in the NZB.
    ///
    /// This collection may be empty.
    #[must_use]
    pub fn parity(&self) -> &Parity {
        &self.parity
    }
}
