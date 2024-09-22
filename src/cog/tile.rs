use super::compression::{Compression, Predictor};
use super::CloudTiffError;
use crate::endian::Endian;
use crate::raster::Raster;
use std::fmt::Display;
use std::io::{Read, Seek, SeekFrom};

pub struct Tile {
    pub width: u32,
    pub height: u32,
    pub bits_per_sample: Vec<u16>,
    pub photometric_interpretation: u16,
    pub endian: Endian,
    pub compression: Compression,
    pub predictor: Predictor,
    pub offset: u64,
    pub byte_count: usize,
}

impl Tile {
    pub fn extract<R: Read + Seek>(&self, stream: &mut R) -> Result<Raster, CloudTiffError> {
        // Get tile bytes
        let mut data = vec![0; self.byte_count];
        stream.seek(SeekFrom::Start(self.offset))?;
        stream.read_exact(&mut data)?;

        // Decompress buffer
        let mut buffer = Compression::from(self.compression)
            .decode(data.as_slice())
            .map_err(|e| CloudTiffError::DecompressError(e))?;
        // Todo, apply endian
        self.predict(buffer.as_mut_slice());

        // Metadata
        let dimensions = (self.width, self.height);
        let interpretation = self.photometric_interpretation.into();

        Ok(Raster {
            buffer,
            dimensions,
            interpretation,
            bits_per_sample: self.bits_per_sample.clone(),
            endian: self.endian,
        })
    }

    fn predict(&self, buffer: &mut [u8]) {
        let bit_depth = self.bits_per_sample[0] as u8;
        let samples = self.bits_per_sample.len();

        match self.predictor {
            Predictor::None => {}
            Predictor::Horizontal => {
                assert!(
                    bit_depth <= 8,
                    "Bit depth {bit_depth} not supported for Horizontal Predictor"
                );
                let row_bytes = self.width as usize * samples * bit_depth as usize / 8;
                for i in 0..buffer.len() {
                    if i % row_bytes < samples {
                        continue;
                    }
                    buffer[i] = buffer[i].wrapping_add(buffer[i - samples]);
                }
            }
            _ => panic!("Predictor {:?} not supported", self.predictor),
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tile({}Bytes @ 0x{:08X}, {:?} Compression, {:?} Predictor)",
            self.byte_count, self.offset, self.compression, self.predictor
        )
    }
}
