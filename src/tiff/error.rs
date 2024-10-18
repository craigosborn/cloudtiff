use std::io;
use std::fmt;

use super::TagId;

#[derive(Debug)]
pub enum TiffError {
    BadMagicBytes,
    NoIfd0,
    ReadError(io::Error),
    MissingTag(TagId),
    BadTag(TagId),
}

impl From<io::Error> for TiffError {
    fn from(e: io::Error) -> Self {
        TiffError::ReadError(e)
    }
}

impl fmt::Display for TiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TiffError {}