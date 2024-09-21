#![allow(dead_code)] // TODO

use std::fmt::Display;
use std::io::{Read, Seek};

mod tiff;

use tiff::{Tiff, TiffParseError};

#[derive(Clone, Debug)]
pub struct CloudTiff {
    tiff: Tiff,
}

impl CloudTiff {
    pub fn open<R: Read + Seek>(stream: &mut R) -> Result<Self, TiffParseError> {
        let tiff = Tiff::open(stream)?;

        // TODO geo tags

        Ok(Self { tiff })
    }
}

impl Display for CloudTiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tiff)
    }
}
