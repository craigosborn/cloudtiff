// https://docs.ogc.org/is/19-008r4/19-008r4.html#_geotiff_tags_for_coordinate_transformations

use crate::tiff::{Endian, Ifd, Tag, TagData, TagId};
use keys::GeoKey;
use num_traits::NumCast;
use std::fmt::Display;

mod error;
mod id;
mod keys;
mod value;

pub use error::GeoTiffError;
pub use id::GeoKeyId;
pub use keys::GeoKeyDirectory;
pub use value::GeoKeyValue;

#[derive(Clone, Debug)]
pub struct GeoTags {
    pub directory: GeoKeyDirectory,
    pub model: GeoModel,
}

#[derive(Clone, Debug)]
pub enum GeoModel {
    Transformed(GeoModelTransformed),
    Scaled(GeoModelScaled),
}

#[derive(Clone, Debug)]
pub struct GeoModelTransformed {
    pub transformation: [f64; 16],
    pub tiepoint: Option<[f64; 6]>,
}

#[derive(Clone, Debug)]
pub struct GeoModelScaled {
    pub pixel_scale: [f64; 3],
    pub tiepoint: [f64; 6],
}

impl Display for GeoTags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GeoTIFF Tags:")?;
        match &self.model {
            GeoModel::Transformed(model) => {
                writeln!(f, "  Tiepoint: {:?}", model.tiepoint)?;
                writeln!(f, "  Transformation: {:?}", model.transformation)?;
            }
            GeoModel::Scaled(model) => {
                writeln!(f, "  Tiepoint: {:?}", model.tiepoint)?;
                writeln!(f, "  Pixel Scale: {:?}", model.pixel_scale)?;
            }
        }
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
    pub fn from_tiepoint_and_scale(tiepoint: [f64; 6], pixel_scale: [f64; 3]) -> Self {
        Self {
            model: GeoModel::Scaled(GeoModelScaled {
                tiepoint,
                pixel_scale,
            }),
            directory: GeoKeyDirectory::new(),
        }
    }

    pub fn from_tiepoint_and_transformation(tiepoint: [f64; 6], transformation: [f64; 16]) -> Self {
        Self {
            model: GeoModel::Transformed(GeoModelTransformed {
                tiepoint: Some(tiepoint),
                transformation,
            }),
            directory: GeoKeyDirectory::new(),
        }
    }

    pub fn parse(ifd: &Ifd) -> Result<Self, GeoTiffError> {
        let tiepoint = get_tag_as_array(ifd, TagId::ModelTiepoint).ok();
        let pixel_scale = get_tag_as_array(ifd, TagId::ModelPixelScale).ok();
        let transformation = get_tag_as_array(ifd, TagId::ModelTransformation).ok();
        let model = match (tiepoint, pixel_scale, transformation) {
            (Some(tiepoint), Some(pixel_scale), _) => GeoModel::Scaled(GeoModelScaled {
                tiepoint,
                pixel_scale,
            }),
            (tiepoint, _, Some(transformation)) => GeoModel::Transformed(GeoModelTransformed {
                tiepoint,
                transformation,
            }),
            _ => return Err(GeoTiffError::MissingTag(TagId::ModelPixelScale)),
        };

        let directory = GeoKeyDirectory::parse(ifd)?;

        Ok(Self { model, directory })
    }

    pub fn add_to_ifd(&self, ifd: &mut Ifd, endian: Endian) {
        match &self.model {
            GeoModel::Transformed(model) => {
                ifd.set_tag(
                    TagId::ModelTransformation,
                    TagData::Double(model.transformation.to_vec()),
                    endian,
                );
                if let Some(tiepoint) = model.tiepoint {
                    ifd.set_tag(
                        TagId::ModelTiepoint,
                        TagData::Double(tiepoint.to_vec()),
                        endian,
                    );
                }
            }
            GeoModel::Scaled(model) => {
                ifd.set_tag(
                    TagId::ModelTiepoint,
                    TagData::Double(model.tiepoint.to_vec()),
                    endian,
                );
                ifd.set_tag(
                    TagId::ModelPixelScale,
                    TagData::Double(model.pixel_scale.to_vec()),
                    endian,
                );
            }
        }
        self.directory.add_to_ifd(ifd, endian);
    }

    pub fn set_key<I: Into<u16>>(&mut self, id: I, value: GeoKeyValue) {
        let code: u16 = id.into();
        let key = GeoKey { code, value };
        let keys = &mut self.directory.keys;
        if let Some(index) = keys.iter().position(|key| key.code == code) {
            keys[index] = key;
        } else {
            keys.push(key);
        }
    }
}

// Methods for accessing tiff tags with geotiff errors
//   TODO remove this
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
