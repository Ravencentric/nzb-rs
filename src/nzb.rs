use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

use flate2::read::GzDecoder;

use crate::errors::{ParseNzbError, ParseNzbFileError};
use crate::file::File;
use crate::meta::Meta;
use crate::parser::parse_files;
use crate::xml;

/// Represents an NZB.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Nzb {
    meta: Meta,
    files: Vec<File>,
}

impl FromStr for Nzb {
    type Err = ParseNzbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nzb = xml::parse_document(s)?;
        let meta = Meta::parse(&nzb);
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
    /// use nzb_rs::{Nzb, ParseNzbFileError};
    ///
    /// fn main() -> Result<(), ParseNzbFileError> {
    ///     let nzb = Nzb::parse_file("tests/nzbs/big_buck_bunny.nzb")?;
    ///     println!("{:#?}", nzb);
    ///     assert_eq!(nzb.file().name(), Some("Big Buck Bunny - S01E01.mkv"));
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

    /// File objects representing the files included in the NZB.
    #[must_use]
    pub fn files(&self) -> &[File] {
        &self.files
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
    pub fn filenames(&self) -> impl Iterator<Item = &str> {
        self.files
            .iter()
            .filter_map(|f| f.name())
            .collect::<BTreeSet<_>>()
            .into_iter()
    }

    /// Vector of unique posters across all the files in the NZB.
    pub fn posters(&self) -> impl Iterator<Item = &str> {
        self.files
            .iter()
            .map(|f| f.poster())
            .collect::<BTreeSet<_>>()
            .into_iter()
    }

    /// Vector of unique groups across all the files in the NZB.
    pub fn groups(&self) -> impl Iterator<Item = &str> {
        self.files
            .iter()
            .flat_map(|f| f.groups().iter().map(String::as_str))
            .collect::<BTreeSet<_>>()
            .into_iter()
    }

    /// Vector of `.par2` files in the NZB.
    pub fn par2_files(&self) -> impl Iterator<Item = &File> {
        self.files.iter().filter(|f| f.is_par2())
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
