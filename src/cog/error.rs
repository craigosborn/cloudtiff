use super::compression::DecompressError;
use super::projection::ProjectionError;
use crate::geotags::GeoTiffError;
use crate::raster::RasterError;
use crate::tiff::TiffError;
use std::fmt::Debug;
use std::io;
use std::sync::PoisonError;

pub type CloudTiffResult<T> = Result<T, CloudTiffError>;

#[derive(Debug)]
pub enum CloudTiffError {
    BadTiff(TiffError),
    BadGeoTiff(GeoTiffError),
    TileLevelOutOfRange((usize, usize)),
    TileIndexOutOfRange((usize, usize)),
    BadWmtsTileIndex((u32,u32,u32)),
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

impl From<TiffError> for CloudTiffError {
    fn from(e: TiffError) -> Self {
        match e {
            TiffError::ReadError(io_error) => CloudTiffError::ReadError(io_error),
            tiff_error => CloudTiffError::BadTiff(tiff_error)
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
