use super::compression::DecompressError;
use crate::geotags::GeoTiffError;
use crate::projection::ProjectionError;
use crate::raster::RasterError;
use crate::tiff::TiffError;
use std::fmt;
use std::io;
use std::sync::PoisonError;

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
    NoLevels,
    RegionOutOfBounds(((f64, f64, f64, f64), (f64, f64, f64, f64))),
    ReadRangeError(String),
    MutexError(String),
    NotSupported(String),
    BadPath(String),
    TODO,
    #[cfg(feature = "async")]
    AsyncJoinError(tokio::task::JoinError),
}

impl fmt::Display for CloudTiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CloudTiffError {}

impl From<TiffError> for CloudTiffError {
    fn from(e: TiffError) -> Self {
        match e {
            TiffError::ReadError(io_error) => CloudTiffError::ReadError(io_error),
            tiff_error => CloudTiffError::BadTiff(tiff_error),
        }
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

impl<G> From<PoisonError<G>> for CloudTiffError {
    fn from(e: PoisonError<G>) -> Self {
        CloudTiffError::MutexError(format!("{e:?}"))
    }
}
