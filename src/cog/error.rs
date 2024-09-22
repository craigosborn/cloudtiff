use super::compression::DecompressError;
use crate::geo::GeoTiffError;
use crate::tiff::TiffError;
use std::io;

#[derive(Debug)]
pub enum CloudTiffError {
    BadTiff(TiffError),
    BadGeoTiff(GeoTiffError),
    TileLevelOutOfRange((usize, usize)),
    TileIndexOutOfRange((usize, usize)),
    ReadError(io::Error),
    DecompressError(DecompressError),
}

impl From<TiffError> for CloudTiffError {
    fn from(e: TiffError) -> Self {
        CloudTiffError::BadTiff(e)
    }
}

impl From<GeoTiffError> for CloudTiffError {
    fn from(e: GeoTiffError) -> Self {
        CloudTiffError::BadGeoTiff(e)
    }
}

impl From<io::Error> for CloudTiffError {
    fn from(e: io::Error) -> Self {
        CloudTiffError::ReadError(e)
    }
}
