use crate::Region;

use super::{Raster, RasterError};

impl Raster {
    pub fn resize(&self, width: u32, height: u32) -> Result<Self, RasterError> {
        if self.bits_per_pixel % 8 != 0 {
            return Err(RasterError::NotSupported(format!(
                "Pixel is not byte aligned: {} bits",
                self.bits_per_pixel
            )));
        }
        let bytes_per_pixel = (self.bits_per_pixel / 8) as usize;
        let mut buffer = vec![0; ((width * height) as usize) * bytes_per_pixel];

        let full_width = self.dimensions.0 as f32;
        let full_height = self.dimensions.1 as f32;
        let scale = (width as f32 / full_width, height as f32 / full_height);
        for j in 0..height {
            let v = (full_height * scale.1) as u32;
            for i in 0..width {
                let u = (full_width * scale.0) as u32;
                let src = (v * self.dimensions.0 + u) as usize * bytes_per_pixel;
                let dst = (j * width + i) as usize * bytes_per_pixel;
                buffer[dst..dst + bytes_per_pixel]
                    .copy_from_slice(&self.buffer[src..src + bytes_per_pixel]);
            }
        }
        Self::new(
            (width, height),
            buffer,
            self.bits_per_sample.clone(),
            self.interpretation,
            self.sample_format.clone(),
            self.extra_samples.clone(),
            self.endian,
        )
    }

    pub fn get_region(&self, region: Region<u32>) -> Result<Self, RasterError> {
        if self.bits_per_pixel % 8 != 0 {
            return Err(RasterError::NotSupported(format!(
                "Pixel is not byte aligned: {} bits",
                self.bits_per_pixel
            )));
        }
        let bytes_per_pixel = (self.bits_per_pixel / 8) as usize;
        let width = region.x.range();
        let height = region.y.range();
        let mut buffer = vec![0; ((width * height) as usize) * bytes_per_pixel];

        for j in region.y.min..region.y.max.min(self.dimensions.1 - 1) {
            for i in region.x.min..region.x.max.min(self.dimensions.0 - 1) {
                let src = (j * self.dimensions.0 + i) as usize * bytes_per_pixel;
                let dst =
                    ((j - region.y.min) * width + i - region.x.min) as usize * bytes_per_pixel;
                let pixel = &self.buffer[src..src + bytes_per_pixel];
                buffer[dst..dst + bytes_per_pixel].copy_from_slice(pixel);
            }
        }
        Self::new(
            (width, height),
            buffer,
            self.bits_per_sample.clone(),
            self.interpretation,
            self.sample_format.clone(),
            self.extra_samples.clone(),
            self.endian,
        )
    }
}
