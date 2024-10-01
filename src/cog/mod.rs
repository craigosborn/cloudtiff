use crate::geotags::GeoTags;
use crate::tiff::Tiff;
use std::fmt::Display;
use std::io::{BufReader, Read, Seek};

mod compression;
mod error;
mod level;
mod projection;
pub mod render;

pub use error::{CloudTiffError, CloudTiffResult};
pub use level::Level;
pub use projection::Projection;

#[derive(Clone, Debug)]
pub struct CloudTiff {
    levels: Vec<Level>,
    projection: Projection,
}

#[cfg(feature = "async")]
use {
    std::io::{Cursor, ErrorKind},
    tokio::io::{AsyncRead, AsyncReadExt},
};

#[cfg(feature = "async")]
impl CloudTiff {
    pub async fn open_async<R: AsyncRead + Unpin>(source: &mut R) -> CloudTiffResult<Self> {
        let fetch_size = 4096;
        let mut result = Err(CloudTiffError::TODO);
        let mut buffer = Vec::with_capacity(fetch_size);
        for _i in 0..10 {
            println!("Fetch {_i}");
            let mut bytes = vec![0; fetch_size];
            let n = source.read(&mut bytes).await?;
            if n == 0 {
                break;
            }
            buffer.extend_from_slice(&bytes[..n]);

            let mut cursor = Cursor::new(&buffer);
            result = Self::open(&mut cursor);
            if let Err(CloudTiffError::ReadError(e)) = &result {
                if matches!(e.kind(), ErrorKind::UnexpectedEof) {
                    continue;
                }
            }
            break;
        }
        result
    }
}

impl CloudTiff {
    pub fn open<R: Read + Seek>(source: &mut R) -> CloudTiffResult<Self> {
        // TODO consider seeking source to start
        let stream = &mut BufReader::new(source);

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

    pub fn bounds_lat_lon_deg(&self) -> CloudTiffResult<(f64, f64, f64, f64)> {
        Ok(self.projection.bounds_lat_lon_deg()?)
    }

    pub fn full_dimensions(&self) -> (u32, u32) {
        self.levels[0].dimensions
    }

    pub fn full_megapixels(&self) -> f64 {
        self.levels[0].megapixels()
    }

    pub fn aspect_ratio(&self) -> f64 {
        let (w, h) = self.full_dimensions();
        w as f64 / h as f64
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
                self.levels.len() - 1,
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

    pub fn level_at_pixel_scale(&self, min_pixel_scale: f64) -> CloudTiffResult<&Level> {
        let level_scales = self.pixel_scales();
        let level_index = level_scales
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (level_scale_x, level_scale_y))| {
                level_scale_x.max(*level_scale_y) < min_pixel_scale
            })
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.get_level(level_index)
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
