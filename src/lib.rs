#![allow(dead_code)] // TODO

use std::fmt::Display;
use std::io::{Read, Seek};

mod error;
mod tiff;
mod geo;

pub use error::CloudTiffError;
pub use tiff::Tiff;
pub use geo::Geo;

#[derive(Clone, Debug)]
pub struct CloudTiff {
    tiff: Tiff,
    geo: Geo,
}

impl CloudTiff {
    pub fn open<R: Read + Seek>(stream: &mut R) -> Result<Self, CloudTiffError> {
        // TIFF
        let tiff = Tiff::open(stream)?;

        // GeoTIFF
        let ifd0 = tiff.ifd0()?;
        let geo = Geo::parse(ifd0)?;

        Ok(Self { tiff, geo })
    }
}

impl Display for CloudTiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tiff)?;
        write!(f, "{}", self.geo)
    }
}
