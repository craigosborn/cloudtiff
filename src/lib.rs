pub mod cog;
mod endian;
mod geotags;
mod integrations;
mod raster;
mod tiff;
mod io;

pub use cog::{CloudTiff, CloudTiffError};

#[cfg(feature = "fs")]
pub use io::fs::PathReader;

#[cfg(feature = "http")]
pub use io::http::HttpReader;

#[cfg(feature = "s3")]
pub use io::s3::S3Reader;