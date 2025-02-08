use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Represents errors that can occur during the parsing of an NZB document.
pub enum ParseNzbError {
    /// Inidcates an invalid or missing 'groups' element within the 'file' element.
    /// Each 'file' element must contain at least one valid 'groups' element.
    #[error("Invalid or missing 'groups' element within the 'file' element. Each 'file' element must contain at least one valid 'groups' element.")]
    GroupsElement,

    /// Indicates an invalid or missing 'segments' element within the 'file' element.
    /// Each 'file' element must contain at least one valid 'segments' element.
    #[error("Invalid or missing 'segments' element within the 'file' element. Each 'file' element must contain at least one valid 'segments' element.")]
    SegmentsElement,

    /// Indicates an invalid or missing 'file' element in the NZB document.
    /// The NZB document must contain at least one valid 'file' element, and each 'file' must have at least one valid 'groups' and 'segments' element.
    #[error("Invalid or missing 'file' element in the NZB document. The NZB document must contain at least one valid 'file' element, and each 'file' must have at least one valid 'groups' and 'segments' element.")]
    FileElement,

    /// Indicates an invalid or missing required attribute in a 'file' element.
    #[error("Invalid or missing required attribute '{attribute}' in a 'file' element.")]
    FileAttribute {
        /// The name of the attribute that was invalid or missing.
        attribute: String,
    },

    /// Indicates that the NZB document is not valid XML and could not be parsed.
    #[error("The NZB document is not valid XML and could not be parsed: {message}")]
    XmlSyntax {
        /// The error message provided by the underlying XML parsing library
        /// ([`roxmltree`](https://crates.io/crates/roxmltree) in this case).
        message: String,
    },
}

impl From<roxmltree::Error> for ParseNzbError {
    fn from(error: roxmltree::Error) -> Self {
        ParseNzbError::XmlSyntax {
            message: error.to_string(),
        }
    }
}

#[derive(Error, Debug)]
/// Represents errors that can occur when attempting to parse an NZB file from a file path.
pub enum ParseNzbFileError {
    /// Input/Output error encountered while trying to access or read the NZB file.
    #[error("I/O error while accessing file '{file}': {source}")]
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
