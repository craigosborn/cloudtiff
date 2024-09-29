use super::compression::DecompressError;
use super::projection::ProjectionError;
use crate::geotags::GeoTiffError;
use crate::raster::RasterError;
use crate::tiff::TiffError;
use std::fmt::Debug;
use std::io;

pub type CloudTiffResult<T> = Result<T, CloudTiffError>;

#[derive(Debug)]
pub enum CloudTiffError {
    BadTiff(TiffError),
    BadGeoTiff(GeoTiffError),
    TileLevelOutOfRange((usize, usize)),
    TileIndexOutOfRange((usize, usize)),
    ImageCoordOutOfRange((f64, f64)),
    ReadError(io::Error),
    DecompresionError(DecompressError),
    RasterizationError(RasterError),
    ProjectionError(ProjectionError),
    RegionOutOfBounds(((f64, f64, f64, f64), (f64, f64, f64, f64))),
    ReadRangeError(String),
    NoLevels,
    JoinError,
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

impl From<DecompressError> for CloudTiffError {
    fn from(e: DecompressError) -> Self {
        CloudTiffError::DecompresionError(e)
    }
}

impl From<RasterError> for CloudTiffError {
    fn from(e: RasterError) -> Self {
        CloudTiffError::RasterizationError(e)
    }
}

impl From<ProjectionError> for CloudTiffError {
    fn from(e: ProjectionError) -> Self {
        CloudTiffError::ProjectionError(e)
    }
}
