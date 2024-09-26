use crate::geotags::GeoTags;
use crate::raster::Raster;
use crate::tiff::Tiff;
use std::fmt::Display;
use std::io::{Read, Seek};

mod compression;
mod error;
mod level;
mod projection;
mod render;

pub use error::{CloudTiffError,CloudTiffResult};
pub use level::Level;
pub use projection::Projection;

#[derive(Clone, Debug)]
pub struct CloudTiff {
    levels: Vec<Level>,
    projection: Projection,
}

impl CloudTiff {
    pub fn open<R: Read + Seek>(stream: &mut R) -> CloudTiffResult<Self> {
        // TIFF indexing
        let tiff = Tiff::open(stream)?;

        // Parse GeoTIFF tags
        let ifd0 = tiff.ifd0()?;
        let geo_tags = GeoTags::parse(ifd0)?;

        // Map IFDs into COG Levels
        //   Note this skips over any ifds which aren't valid COG levels
        //   TODO check that all levels have the same shape
        let mut levels: Vec<Level> = tiff
            .ifds
            .iter()
            .filter_map(|ifd| Level::from_ifd(ifd, tiff.endian).ok())
            .collect();

        // Validate levels
        //   COGs should already have levels sorted big to small
        levels.sort_by(|a, b| (b.megapixels()).total_cmp(&a.megapixels()));
        if levels.len() == 0 {
            return Err(CloudTiffError::NoLevels);
        }

        // Projection georeferences any level
        let projection = Projection::from_geo_tags(&geo_tags, levels[0].dimensions)?;

        Ok(Self { levels, projection })
    }

    pub fn get_tile_at_lat_lon<R: Read + Seek>(
        &self,
        stream: &mut R,
        level: usize,
        lat: f64,
        lon: f64,
    ) -> CloudTiffResult<Raster> {
        let (x, y) = self.projection.transform_from_lat_lon_deg(lat, lon)?;
        let level = self.get_level(level)?;
        level.get_tile_at_image_coords(stream, x, y)
    }

    pub fn bounds_lat_lon_deg(&self) -> CloudTiffResult<(f64, f64, f64, f64)> {
        Ok(self.projection.bounds_lat_lon_deg()?)
    }

    pub fn full_dimensions(&self) -> (u32, u32) {
        self.levels[0].dimensions
    }

    pub fn max_level(&self) -> usize {
        let n = self.levels.len();
        assert!(n > 0, "CloudTIFF has no levels"); // Checked at initialization
        n - 1
    }

    pub fn get_level(&self, level: usize) -> CloudTiffResult<&Level> {
        self.levels
            .get(level)
            .ok_or(CloudTiffError::TileLevelOutOfRange((
                level,
                self.levels.len(),
            )))
    }

    pub fn pixel_scales(&self) -> Vec<(f64, f64)> {
        let scale = self.projection.scale;
        self.levels
            .iter()
            .map(|level| {
                (
                    scale.0 / level.dimensions.0 as f64,
                    scale.1 / level.dimensions.0 as f64,
                )
            })
            .collect()
    }
}

impl Display for CloudTiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CloudTiff({} Levels)", self.levels.len())?;
        for level in self.levels.iter() {
            write!(f, "\n  {level}")?;
        }
        Ok(())
    }
}

pub fn disect<R: Read + Seek>(stream: &mut R) -> Result<(), CloudTiffError> {
    let tiff = Tiff::open(stream)?;
    println!("{tiff}");

    let geo = GeoTags::parse(tiff.ifd0()?)?;
    println!("{geo}");

    Ok(())
}
