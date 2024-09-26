use super::{CloudTiff, CloudTiffResult, Level, PixelMap};
use crate::raster::Raster;
use crate::CloudTiffError;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

impl CloudTiff {
    pub async fn render_region_lat_lon_deg_async<R: Read + Seek>(
        &self,
        stream: Arc<Mutex<R>>,
        nwse: (f64, f64, f64, f64),
        dimensions: (u32, u32),
    ) -> CloudTiffResult<Raster> {
        let epsg = 4326;
        let (north, west, south, east) = nwse;
        let region = (
            west.to_radians(),
            north.to_radians(),
            east.to_radians(),
            south.to_radians(),
        );
        self.render_region_async(stream, epsg, region, dimensions).await
    }

    pub async fn render_region_async<R: Read + Seek>(
        &self,
        stream: Arc<Mutex<R>>,
        epsg: u16,
        region: (f64, f64, f64, f64),
        dimensions: (u32, u32),
    ) -> CloudTiffResult<Raster> {
        let level = self.get_render_level(epsg, &region, &dimensions)?;
        let pixel_map =
        self.get_pixel_map_correct(level, epsg, &region, &dimensions)?;
        self.render_pixel_map_async(stream, level, pixel_map, dimensions).await
    }

    async fn render_pixel_map_async<R: Read + Seek>(
        &self,
        stream: Arc<Mutex<R>>,
        level: &Level,
        pixel_map: PixelMap,
        dimensions: (u32, u32),
    ) -> CloudTiffResult<Raster> {
        let mut render_raster = Raster::blank(
            dimensions.clone(),
            level.bits_per_sample.clone(),
            level.interpretation,
            level.endian,
        );
        for (tile_index, tile_pixel_map) in pixel_map.iter() {
            if let Ok(tile) = level
                .get_tile_by_index_async(stream.clone(), *tile_index)
                .await
            {
                for (from, to) in tile_pixel_map {
                    // TODO interpolation methods other than "floor"
                    if let Some(pixel) = tile.get_pixel(from.0 as u32, from.1 as u32) {
                        let _ = render_raster.put_pixel(to.0, to.1, pixel);
                    }
                }
            }
        }
        Ok(render_raster)
    }
}

impl Level {
    async fn get_tile_bytes_async<R: Read + Seek>(
        &self,
        stream: Arc<Mutex<R>>,
        index: usize,
    ) -> CloudTiffResult<Vec<u8>> {
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
        let mut bytes = vec![0; byte_count];
        {
            let mut locked_stream = stream.lock().unwrap(); 
            locked_stream.seek(SeekFrom::Start(offset))?;
            locked_stream.read_exact(&mut bytes)?;
        }; // Lock is released here

        Ok(bytes)
    }

    pub async fn get_tile_by_index_async<R: Read + Seek>(
        &self,
        stream: Arc<Mutex<R>>,
        index: usize,
    ) -> CloudTiffResult<Raster> {
        let mut bytes = self.get_tile_bytes_async(stream, index).await?;
        self.extract_tile_bytes(&mut bytes)
    }
}
