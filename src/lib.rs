#![doc = include_str!("../README.md")]

mod errors;
mod file;
mod files;
mod meta;
mod nzb;
mod parity;
mod parser;
mod segment;
mod subject;
mod xml;

pub use crate::errors::{FileAttributeKind, ParseNzbError, ParseNzbFileError};
pub use crate::file::File;
pub use crate::files::Files;
pub use crate::meta::Meta;
pub use crate::nzb::Nzb;
pub use crate::parity::Parity;
pub use crate::segment::Segment;
