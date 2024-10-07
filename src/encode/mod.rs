use crate::cog::Compression;
use crate::cog::{Predictor, Region};
use crate::geotags::{GeoKeyId, GeoKeyValue, GeoTags};
use crate::raster::{PlanarConfiguration, Raster};
use crate::tiff::{Endian, TagData, TagId, Tiff, TiffVariant};
use image::DynamicImage;
use std::io::{Seek, SeekFrom, Write};

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
            tile_dimensions: (512, 512),
        })
    }

    pub fn with_projection(mut self, epsg: u16, region: Region<f64>) -> Self {
        self.projection = Some((epsg, region));
        self
    }

    pub fn with_tile_size(mut self, size: u16) -> Self {
        self.tile_dimensions = (size, size);
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

    pub fn encode<W: Write + Seek>(&self, writer: &mut W) -> EncodeResult<()> {
        let endian = self.endian;
        let full_dims = self.raster.dimensions;
        let bps = self.raster.bits_per_sample.clone();
        let interpretation = self.raster.interpretation;
        let planar = PlanarConfiguration::Chunky;
        let predictor = Predictor::No;
        let sample_format: Vec<u16> = self
            .raster
            .sample_format
            .iter()
            .map(|v| (*v).into())
            .collect();

        // TODO is this necessary?
        let (epsg, tiepoint, pixel_scale) = match self.projection {
            Some((epsg, region)) => (
                epsg,
                [0.0, 0.0, 0.0, region.x.min, region.y.min, 0.0],
                [
                    region.x.range().abs() / (full_dims.0 as f64),
                    region.y.range().abs() / (full_dims.1 as f64),
                    0.0,
                ],
            ),
            None => (4326, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 0.0]),
        };

        let mut tiff = Tiff::new(endian, self.variant);

        // GeoTIFF Tags
        let ifd0 = tiff.ifds.first_mut().unwrap(); // Safe because Tiff::new creates ifd0.
        let mut geo = GeoTags::from_tiepoint_and_scale(tiepoint, pixel_scale);
        match epsg {
            // TODO support other projections
            4326 => {
                geo.set_key(GeoKeyId::GTModelTypeGeoKey, GeoKeyValue::Short(vec![2]));
                geo.set_key(GeoKeyId::GTRasterTypeGeoKey, GeoKeyValue::Short(vec![1]));
                geo.set_key(
                    GeoKeyId::GeographicTypeGeoKey,
                    GeoKeyValue::Short(vec![4326]),
                );
                geo.set_key(
                    GeoKeyId::GeographicTypeGeoKey,
                    GeoKeyValue::Ascii("WGS 84".into()),
                );
                geo.set_key(
                    GeoKeyId::GeogAngularUnitsGeoKey,
                    GeoKeyValue::Short(vec![9102]),
                );
                geo.set_key(
                    GeoKeyId::GeogSemiMajorAxisGeoKey,
                    GeoKeyValue::Double(vec![6378137.0]),
                );
                geo.set_key(
                    GeoKeyId::GeogInvFlatteningGeoKey,
                    GeoKeyValue::Double(vec![298.257223563]),
                );
            }
            32609 => {
                geo.set_key(GeoKeyId::GTModelTypeGeoKey, GeoKeyValue::Short(vec![1]));
                geo.set_key(GeoKeyId::GTRasterTypeGeoKey, GeoKeyValue::Short(vec![1]));
                geo.set_key(
                    GeoKeyId::GTCitationGeoKey,
                    GeoKeyValue::Ascii("WGS 84 / UTM zone 9N".into()),
                );
                geo.set_key(
                    GeoKeyId::GeogCitationGeoKey,
                    GeoKeyValue::Ascii("WGS 84".into()),
                );
                geo.set_key(
                    GeoKeyId::GeogAngularUnitsGeoKey,
                    GeoKeyValue::Short(vec![9102]),
                );
                geo.set_key(
                    GeoKeyId::ProjectedCSTypeGeoKey,
                    GeoKeyValue::Short(vec![32609]),
                );
                geo.set_key(
                    GeoKeyId::ProjLinearUnitsGeoKey,
                    GeoKeyValue::Short(vec![9001]),
                );
            }
            _ => {
                return Err(EncodeError::UnsupportedProjection(
                    epsg,
                    "Only EPSG 4326 supported at this time".into(),
                ))
            }
        }
        geo.add_to_ifd(ifd0, endian);

        // TODO add any general TIFF tags to idf0

        // TODO assumes each pyramid is half the previous size
        let overview_levels = ((full_dims.0 as f32 / self.tile_dimensions.0 as f32)
            .log2()
            .min((full_dims.1 as f32 / self.tile_dimensions.1 as f32).log2())
            .floor()) as usize;

        // Full and Overview IFD tags
        for i in 0..=overview_levels {
            let width = full_dims.0 / 2_u32.pow(i as u32);
            let height = full_dims.1 / 2_u32.pow(i as u32);
            let (tile_width, tile_height) = self.tile_dimensions;
            let tile_cols = (width as f32 / tile_width as f32).ceil() as usize;
            let tile_rows = (height as f32 / tile_height as f32).ceil() as usize;
            let number_of_tiles = tile_cols * tile_rows;
            let tile_offsets = match self.variant {
                TiffVariant::Normal => TagData::Long(vec![0; number_of_tiles]),
                TiffVariant::Big => TagData::Long8(vec![0; number_of_tiles]),
            };

            let ifd = if i == 0 {
                tiff.ifds.first_mut().unwrap()
            } else {
                let ifd = tiff.add_ifd();
                ifd.set_tag(TagId::SubfileType, TagData::from_long(1), endian);
                ifd
            };

            ifd.set_tag(
                TagId::ImageWidth,
                TagData::from_long(width), // TODO long or short?
                endian,
            );
            ifd.set_tag(TagId::ImageHeight, TagData::from_long(height), endian);
            ifd.set_tag(TagId::BitsPerSample, TagData::Short(bps.clone()), endian);
            ifd.set_tag(
                TagId::Compression,
                TagData::from_short(self.compression.into()),
                endian,
            );
            ifd.set_tag(
                TagId::PhotometricInterpretation,
                TagData::from_short(interpretation.into()),
                endian,
            );
            ifd.set_tag(
                TagId::SamplesPerPixel,
                TagData::from_short(bps.len() as u16),
                endian,
            );
            ifd.set_tag(
                TagId::PlanarConfiguration,
                TagData::from_short(planar as u16),
                endian,
            );
            ifd.set_tag(
                TagId::Predictor,
                TagData::from_short(predictor as u16),
                endian,
            );
            ifd.set_tag(
                TagId::TileWidth,
                TagData::from_short(tile_width),
                endian,
            );
            ifd.set_tag(
                TagId::TileLength,
                TagData::from_short(tile_height),
                endian,
            );
            ifd.set_tag(TagId::TileOffsets, tile_offsets, endian);
            ifd.set_tag(
                TagId::TileByteCounts,
                TagData::Long(vec![0; number_of_tiles]),
                endian,
            );
            ifd.set_tag(
                TagId::SampleFormat,
                TagData::Short(sample_format.clone()),
                endian,
            );

            if i==0 {
                ifd.set_tag(TagId::GDALMetadata, TagData::Ascii(r#"<GDALMetadata>\n  <Item name="OVR_RESAMPLING_ALG">AVERAGE</Item>\n</GDALMetadata>\n"#.into()), endian);
            }
            ifd.set_tag(TagId::GDALNoData, TagData::Ascii("0".into()), endian);

            ifd.0.sort_by(|a, b| a.code.cmp(&b.code)); // TIFF Tags should be sorted
        }

        // Encode TIFF
        let offsets = tiff.encode(writer)?;

        // Encode tiles
        let mut ifd_tile_offsets = vec![vec![]; overview_levels + 1];
        let mut ifd_tile_bytes = vec![vec![]; overview_levels + 1];
        let (tile_width, tile_height) = self.tile_dimensions;
        for i in (0..=overview_levels).rev() {
            let mut tile_offsets = vec![];
            let mut tile_byte_counts = vec![];
            let width = full_dims.0 / 2_u32.pow(i as u32);
            let height = full_dims.1 / 2_u32.pow(i as u32);
            let tile_cols = (width as f32 / tile_width as f32).ceil() as u32;
            let tile_rows = (height as f32 / tile_height as f32).ceil() as u32;
            let img = if i > 0 {
                &self.raster.resize(width, height)?
            } else {
                &self.raster
            };
            for row in 0..tile_rows {
                for col in 0..tile_cols {
                    tile_offsets.push(writer.stream_position()?);
                    let region = Region::new(
                        col * tile_width as u32,
                        row * tile_height as u32,
                        (col + 1) * tile_width as u32,
                        (row + 1) * tile_height as u32,
                    );
                    let tile_raster = img.get_region(region)?;
                    let tile_bytes = &tile_raster.buffer; // TODO compression and endian
                    writer.write(tile_bytes)?;
                    tile_byte_counts.push(tile_bytes.len() as u32);
                }
            }
            ifd_tile_offsets[i] = tile_offsets;
            ifd_tile_bytes[i] = tile_byte_counts;
        }

        // Go back and set tile offsets and byte count values
        for i in 0..=overview_levels {
            if let Some(offset) = offsets[i].get(&TagId::TileOffsets.into()) {
                writer.seek(SeekFrom::Start(*offset))?;
                match self.variant {
                    TiffVariant::Normal => writer.write(
                        &endian.encode_all(
                            &ifd_tile_offsets[i]
                                .iter()
                                .map(|v| *v as u32)
                                .collect::<Vec<u32>>(),
                        ),
                    ),
                    TiffVariant::Big => writer.write(&endian.encode_all(&ifd_tile_offsets[i])),
                }?;
            }

            if let Some(offset) = offsets[i].get(&TagId::TileByteCounts.into()) {
                writer.seek(SeekFrom::Start(*offset))?;

                writer.write(&endian.encode_all(&ifd_tile_bytes[i]))?;
            }
        }

        Ok(())
    }
}
