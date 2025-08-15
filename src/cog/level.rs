use super::compression::{Compression, Predictor};
use super::CloudTiffError;
use crate::raster::{ExtraSamples, PhotometricInterpretation, Raster, SampleFormat};
use crate::tiff::{Endian, Ifd, TagId, TiffError};
use crate::{Region, UnitFloat};
use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct Level {
    pub overview: Option<usize>,
    pub dimensions: (u32, u32),
    pub tile_width: u32,
    pub tile_height: u32,
    pub compression: Compression,
    pub predictor: Predictor,
    pub interpretation: PhotometricInterpretation,
    pub bits_per_sample: Vec<u16>,
    pub sample_format: Vec<SampleFormat>,
    pub extra_samples: Vec<ExtraSamples>,
    pub endian: Endian,
    pub offsets: Vec<u64>,
    pub byte_counts: Vec<usize>,
}

impl Level {
    pub fn from_ifd(ifd: &Ifd, endian: Endian) -> Result<Self, CloudTiffError> {
        // Required tags
        let width = ifd.get_tag_value(TagId::ImageWidth)?;
        let height = ifd.get_tag_value(TagId::ImageHeight)?;
        let tile_width = ifd.get_tag_value(TagId::TileWidth)?;
        let tile_height = ifd.get_tag_value(TagId::TileLength)?;
        let compression = ifd.get_tag_value::<u16>(TagId::Compression)?.into();
        let predictor = ifd
            .get_tag_value::<u16>(TagId::Predictor)
            .unwrap_or(1)
            .into();
        let bits_per_sample = ifd.get_tag_values(TagId::BitsPerSample)?;
        let sample_format = ifd
            .get_tag_values::<u16>(TagId::SampleFormat)
            .map(|v| {
                v.iter()
                    .map(|v| SampleFormat::from(*v))
                    .collect::<Vec<SampleFormat>>()
            })
            .unwrap_or_else(|_| vec![SampleFormat::Unsigned; bits_per_sample.len()]);
        let extra_samples = ifd
            .get_tag_values::<u16>(TagId::ExtraSamples)
            .map(|v| {
                v.iter()
                    .map(|v| ExtraSamples::from(*v))
                    .collect::<Vec<ExtraSamples>>()
            })
            .unwrap_or_else(|_| vec![]);
        let interpretation = ifd
            .get_tag_value::<u16>(TagId::PhotometricInterpretation)
            .unwrap_or(PhotometricInterpretation::Unknown.into())
            .into();
        let offsets = ifd.get_tag_values(TagId::TileOffsets)?;
        let byte_counts = ifd.get_tag_values(TagId::TileByteCounts)?;

        if offsets.len() != byte_counts.len() {
            return Err(CloudTiffError::BadTiff(TiffError::BadTag(
                TagId::TileOffsets,
            )));
        }

        Ok(Self {
            overview: None,
            dimensions: (width, height),
            tile_width,
            tile_height,
            compression,
            predictor,
            interpretation,
            bits_per_sample,
            sample_format,
            extra_samples,
            endian,
            offsets,
            byte_counts,
        })
    }

    pub fn megapixels(&self) -> f64 {
        (self.dimensions.0 as f64 * self.dimensions.1 as f64) / 1e6
    }

    pub fn width(&self) -> u32 {
        self.dimensions.0
    }

    pub fn height(&self) -> u32 {
        self.dimensions.1
    }

    pub fn tile_indices_within_image_crop(&self, crop: Region<UnitFloat>) -> Vec<usize> {
        let (left, top) = self.tile_coord_from_image_coord(crop.x.min.into(), crop.y.min.into());
        let (right, bottom) =
            self.tile_coord_from_image_coord(crop.x.max.into(), crop.y.max.into());

        let col_count = self.col_count();
        let row_count = self.row_count();

        let col_min = left.floor().max(0.0) as usize;
        let col_max = right.ceil().min(col_count as f64) as usize;
        let row_min = top.floor().max(0.0) as usize;
        let row_max = bottom.ceil().min(row_count as f64) as usize;

        let mut indices = vec![];
        for row in row_min..row_max {
            for col in col_min..col_max {
                indices.push(row * col_count + col);
            }
        }
        indices
    }

    pub fn tile_index(&self, row: usize, col: usize) -> usize {
        row * self.col_count() + col
    }

    pub fn index_from_image_coords(
        &self,
        x: f64,
        y: f64,
    ) -> Result<(usize, f64, f64), CloudTiffError> {
        // TODO UnitFloat type that ensures valid range
        if x < 0.0 || x > 1.0 || y < 0.0 || y > 1.0 {
            return Err(CloudTiffError::ImageCoordOutOfRange((x, y)));
        }

        // Tile coord
        let (col, row) = self.tile_coord_from_image_coord(x, y);

        // Tile index and fraction
        let tile_index = row.floor() as usize * self.col_count() + col.floor() as usize;
        let tile_x = (col - col.floor()) * self.tile_width as f64;
        let tile_y = (row - row.floor()) * self.tile_height as f64;

        Ok((tile_index, tile_x, tile_y))
    }

    pub fn tile_coord_from_image_coord(&self, x: f64, y: f64) -> (f64, f64) {
        let col: f64 = x * self.width() as f64 / self.tile_width as f64;
        let row: f64 = y * self.height() as f64 / self.tile_height as f64;
        (col, row)
    }

    pub fn tile_byte_range(&self, index: usize) -> Result<(u64, u64), CloudTiffError> {
        // Validate index
        let max_valid_index = self.offsets.len().min(self.byte_counts.len()) - 1;
        if index > max_valid_index {
            return Err(CloudTiffError::TileIndexOutOfRange((
                index,
                max_valid_index,
            )));
        }

        // Lookup byte range
        let offset = self.offsets[index];
        let byte_count = self.byte_counts[index];

        Ok((offset, offset + byte_count as u64))
    }

    pub fn extract_tile_from_bytes(&self, bytes: &[u8]) -> Result<Raster, CloudTiffError> {
        // Decompression
        let mut buffer = self.compression.decode(bytes)?;

        // Todo, De-endian

        // Predictor
        let bit_depth = self.bits_per_sample[0] as usize; // TODO not all samples are necessarily the same bit depth
        self.predictor.predict(
            buffer.as_mut_slice(),
            self.tile_width as usize,
            bit_depth,
            self.bits_per_sample.len(),
        )?;

        // Rasterization
        Ok(Raster::new(
            (self.tile_width, self.tile_height),
            buffer,
            self.bits_per_sample.clone(),
            self.interpretation,
            self.sample_format.clone(),
            self.extra_samples.clone(),
            self.endian, // TODO shouldn't need this
        )?)
    }

    pub fn tile_bounds(&self, index: &usize) -> (f64, f64, f64, f64) {
        let col_count = self.col_count();
        let row = (index / col_count) as f64;
        let col = (index % col_count) as f64;
        let left = (col * self.tile_width as f64) / self.dimensions.0 as f64;
        let top = (row * self.tile_height as f64) / self.dimensions.1 as f64;
        let right = ((col + 1.0) * self.tile_width as f64) / self.dimensions.0 as f64;
        let bottom = ((row + 1.0) * self.tile_height as f64) / self.dimensions.1 as f64;
        (left, top, right, bottom)
    }

    pub fn col_count(&self) -> usize {
        (self.width() as f64 / self.tile_width as f64).ceil() as usize
    }

    pub fn row_count(&self) -> usize {
        (self.height() as f64 / self.tile_height as f64).ceil() as usize
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Level({}x{}, {} tiles, {:?} Compression, {:?} Predictor)",
            self.dimensions.0,
            self.dimensions.1,
            self.offsets.len(),
            self.compression,
            self.predictor
        )
    }
}
