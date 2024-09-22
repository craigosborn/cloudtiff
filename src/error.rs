use crate::tiff::TiffError;
use crate::geo::GeoTiffError;

#[derive(Debug)]
pub enum CloudTiffError {
    BadTiff(TiffError),
    BadGeoTiff(GeoTiffError),
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