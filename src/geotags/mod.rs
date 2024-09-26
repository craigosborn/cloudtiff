// https://docs.ogc.org/is/19-008r4/19-008r4.html#_geotiff_tags_for_coordinate_transformations

use std::fmt::Display;

use crate::tiff::{Ifd, Tag, TagId};

mod error;
mod id;
mod keys;
mod value;

pub use error::GeoTiffError;
pub use id::GeoKeyId;
pub use keys::GeoKeyDirectory;
use num_traits::NumCast;
pub use value::GeoKeyValue;

#[derive(Clone, Debug)]
pub struct GeoTags {
    pub tiepoint: [f64; 6],
    pub pixel_scale: [f64; 3],
    pub directory: GeoKeyDirectory,
}

impl Display for GeoTags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GeoTIFF Tags:")?;
        writeln!(f, "  Tiepoint: {:?}", self.tiepoint)?;
        writeln!(f, "  Pixel Scale: {:?}", self.pixel_scale)?;
        write!(
            f,
            "  Directory: {{version: {}, revision: {}.{}}}",
            self.directory.version, self.directory.revision.0, self.directory.revision.1,
        )?;
        if self.directory.keys.len() > 0 {
            write!(f, "\n  Keys:")?;
            for key in self.directory.keys.iter() {
                write!(f, "\n    {key}")?;
            }
        }
        Ok(())
    }
}

impl GeoTags {
    pub fn parse(ifd: &Ifd) -> Result<Self, GeoTiffError> {
        let tiepoint = get_tag_as_array(ifd, TagId::ModelTiepoint)?;
        let pixel_scale = get_tag_as_array(ifd, TagId::ModelPixelScale)?;

        let directory = keys::GeoKeyDirectory::parse(ifd)?;

        Ok(Self {
            tiepoint,
            pixel_scale,
            directory,
        })
    }
}

// Methods for accessing tiff tags with geotiff errors
fn get_geo_tag(ifd: &Ifd, id: TagId) -> Result<&Tag, GeoTiffError> {
    ifd.get_tag(id).ok().ok_or(GeoTiffError::MissingTag(id))
}

fn get_geo_tag_values<T: NumCast>(ifd: &Ifd, id: TagId) -> Result<Vec<T>, GeoTiffError> {
    get_geo_tag(ifd, id)?
        .values()
        .ok_or(GeoTiffError::BadTag(id))
}

fn get_tag_as_array<const N: usize, T: NumCast>(
    ifd: &Ifd,
    id: TagId,
) -> Result<[T; N], GeoTiffError> {
    get_geo_tag_values::<T>(ifd, id)?
        .try_into()
        .ok()
        .ok_or(GeoTiffError::BadTag(id))
}
