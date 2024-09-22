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
pub use value::GeoKeyValue;

#[derive(Clone, Debug)]
pub struct Geo {
    tiepoint: [f64; 6],
    pixel_scale: [f64; 3],
    directory: GeoKeyDirectory,
}

impl Display for Geo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GeoTiff:")?;
        writeln!(f, "  Tiepoint: {:?}", self.tiepoint)?;
        writeln!(f, "  Pixel Scale: {:?}", self.pixel_scale)?;
        writeln!(
            f,
            "  Directory: {{version: {}, revision: {}.{}}}",
            self.directory.version, self.directory.revision.0, self.directory.revision.1,
        )?;
        if self.directory.keys.len() > 0 {
            writeln!(f, "  Keys:")?;
            for key in self.directory.keys.iter() {
                writeln!(f, "    {key}")?;
            }
        }
        Ok(())
    }
}

impl Geo {
    pub fn parse(ifd: &Ifd) -> Result<Self, GeoTiffError> {
        let tiepoint = get_required_array_tag(ifd, TagId::ModelTiepoint)?;
        let pixel_scale = get_required_array_tag(ifd, TagId::ModelPixelScale)?;

        let directory = keys::GeoKeyDirectory::parse(ifd)?;

        Ok(Self {
            tiepoint,
            pixel_scale,
            directory,
        })
    }
}

fn get_required_array_tag<const N: usize>(ifd: &Ifd, id: TagId) -> Result<[f64; N], GeoTiffError> {
    get_required_tag(ifd, id)?
        .value()
        .into_array()
        .ok_or(GeoTiffError::BadTag(id))
}

fn get_required_tag(ifd: &Ifd, id: TagId) -> Result<&Tag, GeoTiffError> {
    ifd.get_tag(id.into()).ok_or(GeoTiffError::MissingTag(id))
}
