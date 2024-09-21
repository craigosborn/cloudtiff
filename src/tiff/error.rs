use std::io;

#[derive(PartialEq, Clone, Debug)]
pub enum TiffParseError {
    BadMagicBytes,
    ReadError,
}

impl From<io::Error> for TiffParseError {
    fn from(_: io::Error) -> Self {
        TiffParseError::ReadError
    }
}
