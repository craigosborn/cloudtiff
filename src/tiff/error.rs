use std::io;

#[derive(Debug)]
pub enum TiffError {
    BadMagicBytes,
    NoIfd0,
    ReadError(io::Error),
}

impl From<io::Error> for TiffError {
    fn from(e: io::Error) -> Self {
        TiffError::ReadError(e)
    }
}
