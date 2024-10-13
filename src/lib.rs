pub mod cog;
pub mod encode;
pub mod geotags;
pub mod io;
pub mod projection;
pub mod raster;
pub mod render;
pub mod tiff;

pub use cog::{disect, CloudTiff, CloudTiffError};
pub use encode::{EncodeError, Encoder, SupportedCompression};
pub use proj4rs::Proj;
pub use projection::primatives::{Point2D, Region, UnitFloat};
pub use projection::Projection;
pub use raster::{Raster, ResizeFilter};
pub use render::tiles;

// IO exports
#[cfg(feature = "http")]
pub use io::http::HttpReader;
#[cfg(feature = "s3")]
pub use io::s3::S3Reader;
#[cfg(feature = "async")]
pub use io::AsyncReadRange;
pub use io::ReadRange;
