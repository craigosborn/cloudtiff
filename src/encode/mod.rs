use crate::cog::{Predictor, Region};
use crate::raster::{Raster, PlanarConfiguration};
use crate::tiff::{Endian, TagData, TagId, Tiff, TiffVariant};
use image::DynamicImage;
use std::io::Write;
use crate::cog::Compression;

pub mod error;

pub use error::{EncodeError, EncodeResult};

#[derive(Debug)]
pub struct Encoder {
    raster: Raster,
    projection: Option<(u16, Region<f64>)>,
    endian: Endian,
    variant: TiffVariant,
    compression: Compression,
    tile_dimensions: (u16, u16),
    // TODO tiff tags
}

impl Encoder {
    #[cfg(feature = "image")]
    pub fn from_image(img: &DynamicImage) -> EncodeResult<Self> {
        Ok(Self {
            raster: Raster::from_image(img)?,
            projection: None,
            endian: Endian::Little,
            variant: TiffVariant::Big,
            compression: Compression::Uncompressed,
            tile_dimensions: (1024, 1024),
        })
    }

    pub fn with_projection(mut self, epsg: u16, region: Region<f64>) -> Self {
        self.projection = Some((epsg, region));
        self
    }

    pub fn with_tile_size(mut self, pixels: u16) -> Self {
        self.tile_dimensions = (pixels, pixels);
        self
    }

    pub fn with_big_endian(mut self, big: bool) -> Self {
        self.endian = if big { Endian::Big } else { Endian::Little };
        self
    }

    pub fn with_big_tiff(mut self, big: bool) -> Self {
        self.variant = if big {
            TiffVariant::Big
        } else {
            TiffVariant::Normal
        };
        self
    }

    pub fn encode<W: Write>(&self, writer: &mut W) -> EncodeResult<()> {
        let endian = self.endian;
        let full_dims = self.raster.dimensions;
        let bps = self.raster.bits_per_sample.clone();
        let interpretation = self.raster.interpretation;
        let planar = PlanarConfiguration::Chunky;
        let predictor = Predictor::No;

        let mut tiff = Tiff::new(endian, self.variant);

        let overview_levels = 5;
        for i in 0..=overview_levels {
            let ifd = if i==0 {tiff.ifds.first_mut().unwrap()} else {tiff.add_ifd()};

            let number_of_tiles = 0;
            let tile_offsets = match self.variant {
                TiffVariant::Normal => TagData::Long(vec![0; number_of_tiles]),
                TiffVariant::Big => TagData::Long8(vec![0; number_of_tiles]),
            };

            let scale_factor = 2_u32.pow(i as u32);
            ifd.set_tag(TagId::ImageWidth, TagData::from_long(full_dims.0 / scale_factor), endian);
            ifd.set_tag(TagId::ImageHeight, TagData::from_long(full_dims.1 / scale_factor), endian);
            ifd.set_tag(TagId::BitsPerSample, TagData::Short(bps.clone()), endian);
            ifd.set_tag(TagId::Compression, TagData::from_short(self.compression.into()), endian);
            ifd.set_tag(TagId::PhotometricInterpretation, TagData::from_short(interpretation.into()), endian);
            ifd.set_tag(TagId::SamplesPerPixel, TagData::from_short(bps.len() as u16), endian);
            ifd.set_tag(TagId::PlanarConfiguration, TagData::from_short(planar as u16), endian);
            ifd.set_tag(TagId::Predictor, TagData::from_short(predictor as u16), endian);
            ifd.set_tag(TagId::TileWidth, TagData::from_short(self.tile_dimensions.0), endian);
            ifd.set_tag(TagId::TileLength, TagData::from_short(self.tile_dimensions.1), endian);
            ifd.set_tag(TagId::TileOffsets, tile_offsets, endian);
            ifd.set_tag(TagId::TileByteCounts, TagData::Long(vec![0; number_of_tiles]), endian);
        }

        // generate levels
        //   calc number of tiles per level
        //   add placeholder tile_byte_count and tile_offsets

        // calculate tiff header + directory size
        // set tile bytes offsets

        // write all

        todo!()
    }
}
