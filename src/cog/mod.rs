use crate::geo::Geo;
use crate::tiff::{TagId, Tiff};
use std::fmt::Display;
use std::io::{Read, Seek};

mod compression;
mod error;
mod tile;

pub use error::CloudTiffError;
pub use tile::Tile;

#[derive(Clone, Debug)]
pub struct CloudTiff {
    tiff: Tiff,
    geo: Geo,
}

impl CloudTiff {
    pub fn open<R: Read + Seek>(stream: &mut R) -> Result<Self, CloudTiffError> {
        // TIFF
        let tiff = Tiff::open(stream)?;

        // GeoTIFF
        let ifd0 = tiff.ifd0()?;
        let geo = Geo::parse(ifd0)?;

        Ok(Self { tiff, geo })
    }

    pub fn max_level(&self) -> usize {
        let n_ifds = self.tiff.ifds.len();
        assert!(n_ifds > 0, "CloudTiff is missing IFDs.");
        n_ifds - 1
    }

    pub fn get_tile(&self, level: usize, row: usize, col: usize) -> Result<Tile, CloudTiffError> {
        let ifds = &self.tiff.ifds;
        let ifd = ifds
            .get(level)
            .ok_or(CloudTiffError::TileLevelOutOfRange((level, ifds.len())))?;

        // Required tags
        let compression = ifd.get_tag_value::<u16>(TagId::Compression)?.into();
        let predictor = ifd
            .get_tag_value::<u16>(TagId::Predictor)
            .unwrap_or(1)
            .into();
        let bits_per_sample = ifd.get_tag_values(TagId::BitsPerSample)?;
        let photometric_interpretation = ifd.get_tag_value(TagId::PhotometricInterpretation)?;
        let tile_width = ifd.get_tag_value(TagId::TileWidth)?;
        let tile_length = ifd.get_tag_value(TagId::TileLength)?;
        let tile_offsets = ifd.get_tag_values(TagId::TileOffsets)?;
        let byte_counts = ifd.get_tag_values(TagId::TileByteCounts)?;

        // Coordinate to index
        let image_width: u16 = ifd.get_tag_value(TagId::ImageWidth)?;
        let max_col = (image_width as f32 / tile_width as f32).ceil() as usize;
        let tile_index = col * max_col + row;

        // Validate tile index
        let max_valid_tile_index = tile_offsets.len().min(byte_counts.len()) - 1;
        if tile_index > max_valid_tile_index {
            return Err(CloudTiffError::TileIndexOutOfRange((
                tile_index,
                max_valid_tile_index,
            )));
        }

        // Indexed tile
        let offset = tile_offsets[tile_index];
        let byte_count = byte_counts[tile_index];

        Ok(Tile {
            width: tile_width,
            height: tile_length,
            predictor,
            compression,
            bits_per_sample,
            photometric_interpretation,
            offset,
            byte_count,
            endian: self.tiff.endian,
        })
    }
}

impl Display for CloudTiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tiff)?;
        write!(f, "{}", self.geo)
    }
}
