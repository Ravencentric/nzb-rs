use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Represents the attributes that can be present in a 'file' element of an NZB document.
pub enum FileAttributeKind {
    Poster,
    Date,
    Subject,
}

impl std::fmt::Display for FileAttributeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Poster => write!(f, "poster"),
            Self::Date => write!(f, "date"),
            Self::Subject => write!(f, "subject"),
        }
    }
}

#[derive(Error, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Represents errors that can occur during the parsing of an NZB document.
pub enum ParseNzbError {
    /// Indicates an invalid or missing 'groups' element within a 'file' element.
    /// Each 'file' element must contain at least one valid 'groups' element.
    #[error(
        "Invalid or missing 'groups' element within a 'file' element. \
        Each 'file' element must contain at least one valid 'groups' element."
    )]
    GroupsElement,

    /// Indicates an invalid or missing 'segments' element within a 'file' element.
    /// Each 'file' element must contain at least one valid 'segments' element.
    #[error(
        "Invalid or missing 'segments' element within a 'file' element. \
        Each 'file' element must contain at least one valid 'segments' element."
    )]
    SegmentsElement,

    /// Indicates an invalid or missing 'file' element in the NZB document.
    /// The NZB document must contain at least one valid 'file' element.
    #[error(
        "Invalid or missing 'file' element in the NZB document. \
        The NZB document must contain at least one valid 'file' element."
    )]
    FileElement,

    /// Indicates that the NZB document contains only `.par2` files.
    /// The NZB document must include at least one non-`.par2` file.
    #[error(
        "The NZB document contains only `.par2` files. \
        It must include at least one non-`.par2` file."
    )]
    OnlyPar2Files,

    /// Indicates an invalid or missing required attribute in a 'file' element.
    #[error("Invalid or missing required attribute '{0}' in a 'file' element.")]
    FileAttribute(FileAttributeKind),

    /// Indicates that the NZB document is not valid XML and could not be parsed.
    ///
    /// The contained string is the error message provided by the underlying
    /// XML parsing library ([`roxmltree`](https://crates.io/crates/roxmltree) in this case).
    #[error("The NZB document is not valid XML and could not be parsed: {0}")]
    XmlSyntax(String),
}

impl From<roxmltree::Error> for ParseNzbError {
    fn from(error: roxmltree::Error) -> Self {
        ParseNzbError::XmlSyntax(error.to_string())
    }
}

#[derive(Error, Debug)]
/// Represents errors that can occur when attempting to parse an NZB file from a file path.
pub enum ParseNzbFileError {
    /// Input/Output error encountered while trying to read the NZB file.
    #[error("I/O error while reading file '{file}': {source}")]
    Io {
        /// The underlying I/O error that occurred.
        source: io::Error,
        /// The path to the file that was being accessed when the error occurred.
        file: PathBuf,
    },

    /// Error during Gzip decompression of the NZB file.
    #[error("Gzip decompression error for file '{file}': {source}")]
    Gzip {
        /// The underlying I/O error reported by the Gzip decompression process.
        source: io::Error,
        /// The path to the file that was being decompressed when the error occurred.
        file: PathBuf,
    },

    ///  Error encountered during the core NZB parsing logic.
    #[error("NZB parsing error: {source}")]
    Parse {
        /// The specific NZB parsing error.
        source: ParseNzbError,
    },
}

impl ParseNzbFileError {
    pub(crate) fn from_io_err(source: io::Error, file: impl Into<PathBuf>) -> Self {
        ParseNzbFileError::Io {
            source,
            file: file.into(),
        }
    }

    pub(crate) fn from_gzip_err(source: io::Error, file: impl Into<PathBuf>) -> Self {
        ParseNzbFileError::Gzip {
            source,
            file: file.into(),
        }
    }
}

impl From<ParseNzbError> for ParseNzbFileError {
    fn from(source: ParseNzbError) -> Self {
        ParseNzbFileError::Parse { source }
    }
}
