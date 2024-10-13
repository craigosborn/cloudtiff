use crate::tiff::Endian;
use std::fmt::Display;

mod image;
mod ops;
mod photometrics;

pub use ops::ResizeFilter;
pub use photometrics::{
    ExtraSamples, PhotometricInterpretation, PlanarConfiguration, SampleFormat,
};

// TODO
//  how to deal with odd bit endianness? Have seen it both ways.

#[derive(Debug)]
pub enum RasterError {
    BufferSize((usize, (u32, u32), Vec<u16>, u32)),
    NotSupported(String),
}

#[derive(Clone, Debug)]
pub struct Raster {
    pub dimensions: (u32, u32),
    pub buffer: Vec<u8>,
    pub bits_per_sample: Vec<u16>,
    pub interpretation: PhotometricInterpretation,
    pub sample_format: Vec<SampleFormat>,
    pub extra_samples: Vec<ExtraSamples>,
    pub endian: Endian,
    bits_per_pixel: u32, // cached sum of bits_per_sample
}

impl Raster {
    pub fn new(
        dimensions: (u32, u32),
        buffer: Vec<u8>,
        bits_per_sample: Vec<u16>,
        interpretation: PhotometricInterpretation,
        sample_format: Vec<SampleFormat>,
        extra_samples: Vec<ExtraSamples>,
        endian: Endian,
    ) -> Result<Self, RasterError> {
        let bits_per_pixel = bits_per_sample.iter().sum::<u16>() as u32;
        let bytes_per_pixel = bits_per_pixel / 8;
        let required_bytes = dimensions.0 * dimensions.1 * bytes_per_pixel;
        if buffer.len() != required_bytes as usize {
            Err(RasterError::BufferSize((
                buffer.len(),
                dimensions,
                bits_per_sample,
                bytes_per_pixel,
            )))
        } else {
            Ok(Self {
                dimensions,
                buffer,
                bits_per_sample,
                interpretation,
                sample_format,
                extra_samples,
                endian,
                bits_per_pixel,
            })
        }
    }

    pub fn blank(
        dimensions: (u32, u32),
        bits_per_sample: Vec<u16>,
        interpretation: PhotometricInterpretation,
        sample_format: Vec<SampleFormat>,
        extra_samples: Vec<ExtraSamples>,
        endian: Endian,
    ) -> Self {
        let bits_per_pixel = bits_per_sample.iter().sum::<u16>() as u32;
        let required_bytes = dimensions.0 * dimensions.1 * bits_per_pixel / 8;
        let buffer = vec![0; required_bytes as usize];
        Self {
            dimensions,
            buffer,
            bits_per_sample,
            interpretation,
            sample_format,
            extra_samples,
            endian,
            bits_per_pixel,
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Vec<u8>> {
        if x >= self.dimensions.0 || y >= self.dimensions.1 {
            return None;
        }

        let bytes_per_row = self.row_size();
        let row_offset: u32 = y * bytes_per_row;

        let start_col_offset_bits = x * self.bits_per_pixel;
        let start_col_offset_bytes = start_col_offset_bits / 8;
        let start = (row_offset + start_col_offset_bytes) as usize;

        let end_col_offset_bits = x * self.bits_per_pixel + self.bits_per_pixel;
        let end_col_offset_bytes = (end_col_offset_bits + 7) / 8;
        let end = (row_offset + end_col_offset_bytes) as usize;

        let mut pixel = self.buffer[start..end].to_vec();
        let n = end - start;

        let start_mask = 0xFF_u8 >> (start_col_offset_bits - start_col_offset_bytes * 8);
        pixel[0] &= start_mask;

        let end_mask =
            ((0xFF_u16 << (end_col_offset_bytes * 8 - end_col_offset_bits)) & 0xFF_u16) as u8;
        pixel[n - 1] &= end_mask;

        Some(pixel)
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, pixel: Vec<u8>) -> Result<(), String> {
        if x >= self.dimensions.0 || y >= self.dimensions.1 {
            return Err("Bad pixel index".into());
        }

        let bytes_per_row = self.row_size();
        let row_offset: u32 = y * bytes_per_row;

        let start_col_offset_bits = x * self.bits_per_pixel;
        let start_col_offset_bytes = start_col_offset_bits / 8;
        let start = (row_offset + start_col_offset_bytes) as usize;

        let end_col_offset_bits = x * self.bits_per_pixel + self.bits_per_pixel;
        let end_col_offset_bytes = (end_col_offset_bits + 7) / 8;
        let end = (row_offset + end_col_offset_bytes) as usize;

        let n = end - start;

        if pixel.len() != n {
            return Err("Bad pixel size".into());
        }

        let start_mask = 0xFF_u8 >> (start_col_offset_bits - start_col_offset_bytes * 8);
        self.buffer[start] = (self.buffer[start] & !start_mask) | (pixel[0] & start_mask);

        if n > 1 {
            let end_mask =
                ((0xFF_u16 << (end_col_offset_bytes * 8 - end_col_offset_bits)) & 0xFF_u16) as u8;
            self.buffer[end - 1] = (self.buffer[end - 1] & !end_mask) | (pixel[n - 1] & end_mask);
        }

        for i in 0..n {
            self.buffer[start + i] = pixel[i];
        }

        Ok(())
    }

    pub fn row_size(&self) -> u32 {
        (self.dimensions.0 * self.bits_per_pixel + 7) / 8
    }

    pub fn sample_size(&self) -> Result<u16, RasterError> {
        if self.bits_per_sample.len() == 0 {
            return Err(RasterError::NotSupported("Empty bits per sample".into()));
        }
        let first = self.bits_per_sample[0];
        if self.bits_per_sample.iter().all(|v| *v == first) {
            Ok(first)
        } else {
            Err(RasterError::NotSupported(
                "Resize Filter Maximum only available for 8 or 16 bits per sample".into(),
            ))
        }
    }
}

impl Display for Raster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Raster({}x{}, {:?}, {:?}, {}Bytes, {:?} Endian)",
            self.dimensions.0,
            self.dimensions.1,
            self.bits_per_sample,
            self.interpretation,
            self.buffer.len(),
            self.endian
        )
    }
}
