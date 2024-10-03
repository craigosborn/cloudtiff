use crate::cog::ProjectionError;
use crate::raster::RasterError;
use std::fmt::Debug;
use std::io;

pub type EncodeResult<T> = Result<T, EncodeError>;

#[derive(Debug)]
pub enum EncodeError {
    WriteError(io::Error),
    RasterizationError(RasterError),
    ProjectionError(ProjectionError),
}

impl From<io::Error> for EncodeError {
    fn from(e: io::Error) -> Self {
        EncodeError::WriteError(e)
    }
}
impl From<RasterError> for EncodeError {
    fn from(e: RasterError) -> Self {
        EncodeError::RasterizationError(e)
    }
}

impl From<ProjectionError> for EncodeError {
    fn from(e: ProjectionError) -> Self {
        EncodeError::ProjectionError(e)
    }
}