use super::{Raster, RasterError};
use crate::Region;

#[derive(Debug, Copy, Clone)]
pub enum ResizeFilter {
    Nearest,
    Maximum,
}

impl Raster {
    pub fn resize(
        &self,
        width: u32,
        height: u32,
        filter: ResizeFilter,
    ) -> Result<Self, RasterError> {
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
        let scale = (full_width / width as f32, full_height / height as f32);
        match filter {
            ResizeFilter::Nearest => {
                for j in 0..height {
                    let v = (j as f32 * scale.1) as u32;
                    for i in 0..width {
                        let u = (i as f32 * scale.0) as u32;
                        let src = (v * self.dimensions.0 + u) as usize * bytes_per_pixel;
                        let dst = (j * width + i) as usize * bytes_per_pixel;
                        buffer[dst..dst + bytes_per_pixel]
                            .copy_from_slice(&self.buffer[src..src + bytes_per_pixel]);
                    }
                }
            }
            ResizeFilter::Maximum => {
                let sample_size = self.sample_size()?;
                if sample_size != 8 {
                    return Err(RasterError::NotSupported(format!(
                        "Sample size {sample_size} not supported in ResizeFilter::Maximum"
                    )));
                }
                let samples = self.bits_per_sample.len();
                for j in 0..height {
                    let v_start = (j as f32 * scale.1) as u32;
                    let v_end = ((j + 1) as f32 * scale.1) as u32;
                    for i in 0..width {
                        let u_start = (i as f32 * scale.0) as u32;
                        let u_end = ((i + 1) as f32 * scale.0) as u32;
                        let dst = (j * width + i) as usize * bytes_per_pixel;
                        for s in 0..samples {
                            let mut value: u8 = 0;
                            for v in v_start..v_end {
                                for u in u_start..u_end {
                                    let src =
                                        (v * self.dimensions.0 + u) as usize * bytes_per_pixel;
                                    value = value.max(self.buffer[src + s]);
                                }
                            }
                            buffer[dst + s] = value;
                        }
                    }
                }
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
