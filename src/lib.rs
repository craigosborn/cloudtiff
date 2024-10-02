pub mod cog;
mod endian;
mod geotags;
mod io;
mod raster;
mod tiff;

pub use cog::{CloudTiff, CloudTiffError};
pub use cog::{Point2D, Region};

// IO exports
#[cfg(feature = "http")]
pub use io::http::HttpReader;
#[cfg(feature = "s3")]
pub use io::s3::S3Reader;
