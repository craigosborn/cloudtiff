#![cfg(feature = "image")]

use super::{photometrics::PhotometricInterpretation as Style, RasterError, SampleFormat};
use crate::raster::Raster;
use crate::tiff::Endian;
use image::{DynamicImage, ImageBuffer};

impl TryInto<DynamicImage> for Raster {
    type Error = String;

    fn try_into(self) -> Result<DynamicImage, Self::Error> {
        let Raster {
            dimensions: (width, height),
            buffer,
            bits_per_sample,
            interpretation: _,
            endian,
            ..
        } = self;

        match bits_per_sample.as_slice() {
            [8] => {
                ImageBuffer::from_raw(width, height, buffer).map(|ib| DynamicImage::ImageLuma8(ib))
            }
            [8, 8] => {
                ImageBuffer::from_raw(width, height, buffer).map(|ib| DynamicImage::ImageLumaA8(ib))
            }
            [16] => endian.decode_all(&buffer).and_then(|buffer| {
                ImageBuffer::from_raw(width, height, buffer).map(|ib| DynamicImage::ImageLuma16(ib))
            }),
            [16, 16] => endian.decode_all(&buffer).and_then(|buffer| {
                ImageBuffer::from_raw(width, height, buffer)
                    .map(|ib| DynamicImage::ImageLumaA16(ib))
            }),
            [8, 8, 8] => {
                ImageBuffer::from_raw(width, height, buffer).map(|ib| DynamicImage::ImageRgb8(ib))
            }
            [8, 8, 8, 8] => {
                ImageBuffer::from_raw(width, height, buffer).map(|ib| DynamicImage::ImageRgba8(ib))
            }
            [16, 16, 16] => endian.decode_all(&buffer).and_then(|buffer| {
                ImageBuffer::from_raw(width, height, buffer).map(|ib| DynamicImage::ImageRgb16(ib))
            }),
            [16, 16, 16, 16] => endian.decode_all(&buffer).and_then(|buffer| {
                ImageBuffer::from_raw(width, height, buffer).map(|ib| DynamicImage::ImageRgba16(ib))
            }),
            [32, 32, 32] => endian.decode_all(&buffer).and_then(|buffer| {
                ImageBuffer::from_raw(width, height, buffer).map(|ib| DynamicImage::ImageRgb32F(ib))
            }),
            [32, 32, 32, 32] => endian.decode_all(&buffer).and_then(|buffer| {
                ImageBuffer::from_raw(width, height, buffer)
                    .map(|ib| DynamicImage::ImageRgba32F(ib))
            }),
            _ => None,
        }
        .ok_or("Not Supported".to_string())
    }
}

impl Raster {
    pub fn from_image(img: &DynamicImage) -> Result<Self, RasterError> {
        let dimensions = (img.width(), img.height());
        let buffer = img.as_bytes().to_vec();
        let endian = if cfg!(target_endian = "big") {
            Endian::Big
        } else {
            Endian::Little
        };

        let (interpretation, bits_per_sample, sample_format) = match img {
            DynamicImage::ImageLuma16(_) => (Style::Unknown, vec![16], vec![SampleFormat::Unsigned]),
            DynamicImage::ImageLuma8(_) => (Style::Unknown, vec![8], vec![SampleFormat::Unsigned]),
            DynamicImage::ImageLumaA8(_) => (Style::Unknown, vec![8, 8], vec![SampleFormat::Unsigned; 2]),
            DynamicImage::ImageRgb8(_) => (Style::Unknown, vec![8, 8, 8], vec![SampleFormat::Unsigned; 3]),
            DynamicImage::ImageRgba8(_) => (Style::Unknown, vec![8, 8, 8, 8], vec![SampleFormat::Unsigned; 4]),
            DynamicImage::ImageLumaA16(_) => (Style::Unknown, vec![16, 16], vec![SampleFormat::Unsigned; 2]),
            DynamicImage::ImageRgb16(_) => (Style::Unknown, vec![16, 16, 16], vec![SampleFormat::Unsigned; 3]),
            DynamicImage::ImageRgba16(_) => (Style::Unknown, vec![16, 16, 16, 16], vec![SampleFormat::Unsigned; 4]),
            DynamicImage::ImageRgb32F(_) => (Style::Unknown, vec![32, 32, 32], vec![SampleFormat::Float; 3]),
            DynamicImage::ImageRgba32F(_) => (Style::Unknown, vec![32, 32, 32, 32], vec![SampleFormat::Float; 4]),
            _ => (Style::Unknown, vec![8], vec![SampleFormat::Unsigned]),
        };

        Self::new(dimensions, buffer, bits_per_sample, interpretation, sample_format, endian)
    }
}
