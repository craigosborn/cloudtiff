use crate::endian::Endian;
use std::fmt::Display;

mod photometrics;

pub use photometrics::PhotometricInterpretation;

// TODO
//  how to deal with odd bit endianness? Have seen it both ways.

#[derive(Debug)]
pub enum RasterError {
    BufferSize((usize, (u32, u32), Vec<u16>)),
}

#[derive(Clone, Debug)]
pub struct Raster {
    pub dimensions: (u32, u32),
    pub buffer: Vec<u8>,
    pub bits_per_sample: Vec<u16>,
    pub interpretation: PhotometricInterpretation,
    pub endian: Endian,
}

impl Raster {
    pub fn new(
        dimensions: (u32, u32),
        buffer: Vec<u8>,
        bits_per_sample: Vec<u16>,
        interpretation: PhotometricInterpretation,
        endian: Endian,
    ) -> Result<Self, RasterError> {
        let bits_per_pixel = bits_per_sample.iter().sum::<u16>() as usize;
        let required_bytes =
            dimensions.0 as usize * dimensions.1 as usize * bits_per_pixel as usize / 8;
        if buffer.len() != required_bytes {
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
                endian,
            })
        }
    }
    
    pub fn blank(
        dimensions: (u32, u32),
        bits_per_sample: Vec<u16>,
        interpretation: PhotometricInterpretation,
        endian: Endian,
    ) -> Self {
        let bits_per_pixel = bits_per_sample.iter().sum::<u16>() as usize;
        let required_bytes =
            dimensions.0 as usize * dimensions.1 as usize * bits_per_pixel as usize / 8;
        let buffer = vec![0; required_bytes];
        Self {
            dimensions,
            buffer,
            bits_per_sample,
            interpretation,
            endian,
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Vec<u32>> {
        if x >= self.dimensions.0 || y >= self.dimensions.1 {
            return None; // Out of bounds
        }
        
        // Calculate the start index of the pixel in the buffer
        let mut index = 0;
        let mut pixel = Vec::new();
        let bytes_per_row = self.row_size();
        
        let offset = ((y * bytes_per_row as u32) + (x * self.bits_per_pixel() as u32 / 8)) as usize;
        
        for &bits in &self.bits_per_sample {
            let bytes_to_read = (bits + 7) / 8; // Number of bytes needed to store 'bits' bits
            let mut value: u32 = 0;
            for i in 0..bytes_to_read {
                value |= (self.buffer[offset + index] as u32) << (i * 8);
                index += 1;
            }
            
            if bits < 8 * bytes_to_read {
                value &= (1 << bits) - 1; // Mask the unused bits
            }
            
            pixel.push(value);
        }
        
        Some(pixel)
    }

    /// Sets the pixel at the specified (x, y) coordinates.
    /// Expects a vector of u32 values, each representing a channel.
    pub fn put_pixel(&mut self, x: u32, y: u32, values: Vec<u32>) -> Result<(), RasterError> {
        if x >= self.dimensions.0 || y >= self.dimensions.1 {
            return Err(RasterError::BufferSize((self.buffer.len(), self.dimensions, self.bits_per_sample.clone()))); // Out of bounds
        }
        
        // Check if values length matches the number of channels
        if values.len() != self.bits_per_sample.len() {
            return Err(RasterError::BufferSize((self.buffer.len(), self.dimensions, self.bits_per_sample.clone())));
        }
        
        // Calculate the start index of the pixel in the buffer
        let mut index = 0;
        let bytes_per_row = self.row_size();
        
        let offset = ((y * bytes_per_row as u32) + (x * self.bits_per_pixel() as u32 / 8)) as usize;

        for (i, &bits) in self.bits_per_sample.iter().enumerate() {
            let bytes_to_write = (bits + 7) / 8;
            let value = values[i];

            for j in 0..bytes_to_write {
                let byte = (value >> (j * 8)) & 0xFF;
                self.buffer[offset + index] = byte as u8;
                index += 1;
            }
        }
        
        Ok(())
    }
    
    fn bits_per_pixel(&self) -> u16 {
        self.bits_per_sample.iter().sum()
    }

    fn row_size(&self) -> usize {
        let bits_per_pixel = self.bits_per_pixel();
        (self.dimensions.0 as usize * bits_per_pixel as usize + 7) / 8
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
