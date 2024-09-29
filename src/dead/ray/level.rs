use crate::cog::{CloudTiffResult, Level};
use crate::raster::Raster;
use crate::reader::ReadRange;
use crate::CloudTiffError;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;
use std::time::Instant;
use tracing::*;

impl Level {
    pub fn stream_tiles_in_region_ray<R: ReadRange + Clone>(
        &self,
        source: &mut R,
        region: (f64, f64, f64, f64),
    ) -> Vec<CloudTiffResult<(Raster, usize, (f64, f64, f64, f64))>> {
        let indices = self.tile_indices_within_image_region(region);
        self.stream_tiles_ray(source, indices)
    }

    pub fn stream_tiles_ray<R: ReadRange + Clone>(
        &self,
        source: &mut R,
        indices: Vec<usize>, // (left, top, right, bottom)
    ) -> Vec<CloudTiffResult<(Raster, usize, (f64, f64, f64, f64))>> {
        let self_arc = Arc::new(self.clone()); // TODO is this the best way?

        debug!("Start Stream Bytes Ray");
        let t0 = Instant::now();
        let byte_results: Vec<_> = indices
            .into_iter()
            .map(|index| (self_arc.clone(), source.clone(), index))
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(self_clone, mut reader_clone, index)| {
                (
                    self_clone.get_tile_bytes_ray(&mut reader_clone, index),
                    self_clone,
                    index,
                )
            })
            .collect();
        debug!(
            "End Stream Bytes Par in {}ms",
            t0.elapsed().as_secs_f32() * 1e3
        );

        debug!("Start stream extract par");
        let t_extract = Instant::now();
        let input2: Vec<_> = byte_results
            .into_iter()
            .map(|(byte_result, self_clone, index)| match byte_result {
                Ok(bytes) => Ok((self_clone, bytes, index)),
                Err(e) => Err(e),
            })
            .collect();

        let result_vec: Vec<_> = input2
            .into_par_iter()
            .map(|result| match result {
                Ok((self_clone, mut bytes, index)) => {
                    debug!("Start Extract Bytes Par {index}");
                    let t0 = Instant::now();
                    let r = self_clone
                        .extract_tile_bytes_ray(&mut bytes)
                        .map(|r| (r, index, self_clone.tile_bounds(&index)));

                    debug!(
                        "End Extract Bytes Par in {}ms",
                        t0.elapsed().as_secs_f32() * 1e3
                    );
                    r
                }
                Err(e) => Err(e),
            })
            .collect();

        debug!(
            "End stream extract Par in {}ms",
            t_extract.elapsed().as_secs_f32() * 1e3
        );
        result_vec
    }

    pub fn extract_tile_bytes_ray(&self, bytes: &[u8]) -> Result<Raster, CloudTiffError> {
        // Decompression
        let t0 = Instant::now();
        let mut buffer = self.compression.decode(bytes)?;
        debug!(
            "Decompression Par took {:.3}ms",
            t0.elapsed().as_secs_f32() * 1e3
        );

        // Todo, De-endian

        // Predictor
        let bit_depth = self.bits_per_sample[0] as usize; // TODO not all samples are necessarily the same bit depth
        self.predictor.predict(
            buffer.as_mut_slice(),
            self.tile_width as usize,
            bit_depth,
            self.bits_per_sample.len(),
        )?;

        // Rasterization
        Ok(Raster::new(
            (self.tile_width, self.tile_height),
            buffer,
            self.bits_per_sample.clone(),
            self.interpretation,
            self.endian, // TODO shouldn't need this
        )?)
    }

    pub fn get_tile_at_image_coords_ray<R: ReadRange>(
        &self,
        reader: &mut R,
        x: f64,
        y: f64,
    ) -> CloudTiffResult<Raster> {
        let (index, _tile_x, _tile_y) = self.index_from_image_coords(x, y)?;
        self.get_tile_by_index_ray(reader, index)
    }

    pub fn get_tile_by_index_ray<R: ReadRange>(
        &self,
        reader: &mut R,
        index: usize,
    ) -> CloudTiffResult<Raster> {
        let mut bytes = self.get_tile_bytes_ray(reader, index)?;
        self.extract_tile_bytes(&mut bytes)
    }

    pub fn get_tile_bytes_ray<R: ReadRange>(
        &self,
        reader: &mut R,
        index: usize,
    ) -> CloudTiffResult<Vec<u8>> {
        // debug!("Start Bytes Par: {index}");
        // let t0 = Instant::now();

        // Validate index
        let max_valid_index = self.offsets.len().min(self.byte_counts.len()) - 1;
        if index > max_valid_index {
            return Err(CloudTiffError::TileIndexOutOfRange((
                index,
                max_valid_index,
            )));
        }

        // Lookup byte range
        let offset = self.offsets[index];
        let byte_count = self.byte_counts[index];

        // Read bytes
        let bytes = reader.read_range(offset, offset + byte_count as u64)?;

        Ok(bytes)
    }
}
