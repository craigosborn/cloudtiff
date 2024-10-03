mod cog;
mod encode;
mod geotags;
mod io;
mod raster;
mod tiff;

pub use cog::{disect, CloudTiff, CloudTiffError};
pub use cog::{Point2D, Region};
pub use encode::{EncodeError, Encoder};

// IO exports
#[cfg(feature = "http")]
pub use io::http::HttpReader;
#[cfg(feature = "s3")]
pub use io::s3::S3Reader;
#[cfg(feature = "async")]
pub use io::AsyncReadRange;
pub use io::ReadRange;
