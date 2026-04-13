#![doc = include_str!("../README.md")]

mod errors;
mod file;
mod files;
mod meta;
mod nzb;
mod parser;
mod segment;
mod subject;
mod xml;

pub use crate::errors::{FileAttributeKind, ParseNzbError, ParseNzbFileError};
pub use crate::file::File;
pub use crate::files::{Files, Parity};
pub use crate::meta::Meta;
pub use crate::nzb::Nzb;
pub use crate::segment::Segment;
