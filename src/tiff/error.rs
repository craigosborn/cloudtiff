use std::io;

use super::TagId;

#[derive(Debug)]
pub enum TiffError {
    BadMagicBytes,
    NoIfd0,
    ReadError(io::Error),
    MissingTag(TagId),
    BadTag(TagId),
    TileOutOfRange((usize, usize)),
}

impl From<io::Error> for TiffError {
    fn from(e: io::Error) -> Self {
        TiffError::ReadError(e)
    }
}
