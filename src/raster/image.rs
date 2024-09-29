#![cfg(feature = "image")]

use image::{DynamicImage, ImageBuffer};
use crate::raster::Raster;

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