#![allow(dead_code)] // TODO

use std::fmt::Display;
use std::io::{Read, Seek};

mod error;
mod geo;
mod tiff;

pub use error::CloudTiffError;
pub use geo::Geo;
pub use tiff::{Ifd, TagId, Tiff, Tile};

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

    pub fn get_tile<R: Read + Seek>(
        &self,
        stream: &mut R,
        level: usize,
        row: usize,
        col: usize,
    ) -> Result<Tile, CloudTiffError> {
        let ifds = &self.tiff.ifds;
        let ifd = ifds
            .get(level)
            .ok_or(CloudTiffError::InvalidIndex((level, ifds.len())))?;
        let image_width: u16 = ifd.get_tag_value(TagId::ImageWidth)?;
        let tile_width: u16 = ifd.get_tag_value(TagId::TileWidth)?;
        let max_col = (image_width as f32 / tile_width as f32).ceil() as usize;
        let tile_index = col * max_col + row;
        println!("tile_index: {tile_index}");
        Ok(ifd.get_tile(stream, tile_index)?)
    }
}

impl Display for CloudTiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tiff)?;
        write!(f, "{}", self.geo)
    }
}
