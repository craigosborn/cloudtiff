pub mod cog;
mod geotags;
mod io;
mod raster;
mod tiff;

pub use cog::{CloudTiff, CloudTiffError};
pub use cog::{Point2D, Region};

// IO exports
pub use io::ReadRange;
#[cfg(feature = "async")]
pub use io::AsyncReadRange;
#[cfg(feature = "http")]
pub use io::http::HttpReader;
#[cfg(feature = "s3")]
pub use io::s3::S3Reader;
