#![cfg(feature = "image")]

use super::{photometrics::PhotometricInterpretation as Style, ExtraSamples, RasterError, SampleFormat};
use crate::raster::Raster;
use crate::tiff::Endian;
use image::{DynamicImage, ImageBuffer, Rgba};

impl Raster {
    pub fn get_pixel_rgba(&self, x: u32, y: u32) -> Option<Rgba<u8>> {
        let p = self.get_pixel(x, y)?;
        Some(match self.bits_per_sample.as_slice() {
            [8] => Rgba([p[0], p[0], p[0], 255]),
            [8, 8] => Rgba([p[0], p[0], p[0], p[1]]),
            [8, 8, 8] => Rgba([p[0], p[1], p[2], 255]),
            [8, 8, 8, 8] => Rgba([p[0], p[1], p[2], p[3]]),
            [16] => {
                let v: i16 = self.endian.decode([p[0], p[1]]).ok()?;
                let v8 = (v / 10).clamp(0, 255) as u8;
                Rgba([v8, v8, v8, 255])
            }
            _ => return None,
        })
    }
}

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
    pub fn into_image(self) -> Result<DynamicImage, String> {
        self.try_into()
    }

    pub fn from_image(img: &DynamicImage) -> Result<Self, RasterError> {
        let dimensions = (img.width(), img.height());
        let buffer = img.as_bytes().to_vec();
        let endian = if cfg!(target_endian = "big") {
            Endian::Big
        } else {
            Endian::Little
        };

        let (interpretation, bits_per_sample, sample_format, extra_samples) = match img {
            DynamicImage::ImageLuma16(_) => (
                Style::BlackIsZero,
                vec![16],
                vec![SampleFormat::Unsigned],
                vec![],
            ),
            DynamicImage::ImageLuma8(_) => (
                Style::BlackIsZero,
                vec![8],
                vec![SampleFormat::Unsigned],
                vec![],
            ),
            DynamicImage::ImageLumaA8(_) => (
                Style::BlackIsZero,
                vec![8, 8],
                vec![SampleFormat::Unsigned; 2],
                vec![ExtraSamples::AssociatedAlpha],
            ),
            DynamicImage::ImageRgb8(_) => (
                Style::RGB,
                vec![8, 8, 8],
                vec![SampleFormat::Unsigned; 3],
                vec![],
            ),
            DynamicImage::ImageRgba8(_) => (
                Style::RGB,
                vec![8, 8, 8, 8],
                vec![SampleFormat::Unsigned; 4],
                vec![ExtraSamples::AssociatedAlpha],
            ),
            DynamicImage::ImageLumaA16(_) => (
                Style::BlackIsZero,
                vec![16, 16],
                vec![SampleFormat::Unsigned; 2],
                vec![ExtraSamples::AssociatedAlpha],
            ),
            DynamicImage::ImageRgb16(_) => (
                Style::RGB,
                vec![16, 16, 16],
                vec![SampleFormat::Unsigned; 3],
                vec![],
            ),
            DynamicImage::ImageRgba16(_) => (
                Style::RGB,
                vec![16, 16, 16, 16],
                vec![SampleFormat::Unsigned; 4],
                vec![ExtraSamples::AssociatedAlpha],
            ),
            DynamicImage::ImageRgb32F(_) => (
                Style::RGB,
                vec![32, 32, 32],
                vec![SampleFormat::Float; 3],
                vec![],
            ),
            DynamicImage::ImageRgba32F(_) => (
                Style::RGB,
                vec![32, 32, 32, 32],
                vec![SampleFormat::Float; 4],
                vec![ExtraSamples::AssociatedAlpha],
            ),
            _ => (
                Style::Unknown,
                vec![8],
                vec![SampleFormat::Unsigned],
                vec![],
            ),
        };

        Self::new(
            dimensions,
            buffer,
            bits_per_sample,
            interpretation,
            sample_format,
            extra_samples,
            endian,
        )
    }
}
