use crate::tiff::Endian;
use std::fmt::Display;

mod image;
mod ops;
mod photometrics;

pub use photometrics::{PhotometricInterpretation, PlanarConfiguration, SampleFormat};

// TODO
//  how to deal with odd bit endianness? Have seen it both ways.

#[derive(Debug)]
pub enum RasterError {
    BufferSize((usize, (u32, u32), Vec<u16>)),
    NotSupported(String),
}

#[derive(Clone, Debug)]
pub struct Raster {
    pub dimensions: (u32, u32),
    pub buffer: Vec<u8>,
    pub bits_per_sample: Vec<u16>,
    pub interpretation: PhotometricInterpretation,
    pub sample_format: Vec<SampleFormat>,
    pub endian: Endian,
    bits_per_pixel: u32, // calculated from bits_per_sample and cached
}

impl Raster {
    pub fn new(
        dimensions: (u32, u32),
        buffer: Vec<u8>,
        bits_per_sample: Vec<u16>,
        interpretation: PhotometricInterpretation,
        sample_format: Vec<SampleFormat>,
        endian: Endian,
    ) -> Result<Self, RasterError> {
        let bits_per_pixel = bits_per_sample.iter().sum::<u16>() as u32;
        let required_bytes = dimensions.0 * dimensions.1 * bits_per_pixel / 8;
        if buffer.len() != required_bytes as usize {
            Err(RasterError::BufferSize((
                buffer.len(),
                dimensions,
                bits_per_sample,
            )))
        } else {
            Ok(Self {
                dimensions,
                buffer,
                bits_per_sample,
                interpretation,
                sample_format,
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

    // fn bits_per_pixel(&self) -> u16 {
    //     self.bits_per_sample.iter().sum()
    // }

    fn row_size(&self) -> u32 {
        (self.dimensions.0 * self.bits_per_pixel + 7) / 8
    }

    // fn normalize_rgba(&self) -> Self {
    //     let rgba = Self::blank(
    //         self.dimensions,
    //         vec![8, 8, 8, 8],
    //         self.interpretation,
    //         self.endian,
    //     );

    //     match self.bits_per_sample.as_slice() {
    //         [8, 8, 8] => self.clone(),
    //         [16] => {

    //             for i in 0..self.dimensions.0 {
    //                 for j in 0..self.dimensions.1 {
    //                     if let Some(p) = self.get_pixel(i, j) {
    //                         rgba.put
    //                     }
    //                 }
    //             }
    //             for i in 0..self.dimensions.0 {
    //                 for j in 0..self.dimensions.1 {
    //                     if let Some(p) = self.get_pixel(i, j) {
    //                         rgba.put
    //                     }
    //                 }
    //             }
    //             rgba
    //         }
    //     }
    // }
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
