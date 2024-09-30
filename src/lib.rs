#![allow(dead_code)] // TODO

pub mod cog;
mod endian;
mod geotags;
mod raster;
mod tiff;
mod io;

pub use cog::{CloudTiff, CloudTiffError};
pub use io::{ReadRange, AsyncReadRange};

#[cfg(feature = "fs")]
pub use io::path::PathReader;

#[cfg(feature = "http")]
pub use io::http::HttpReader;

#[cfg(feature = "s3")]
pub use io::s3::S3Reader;