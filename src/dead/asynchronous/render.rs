use crate::cog::render::PixelMap;
use crate::cog::{CloudTiff, CloudTiffResult, Level};
use crate::raster::Raster;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncSeek};
use tokio::sync::Mutex;

impl CloudTiff {
    pub async fn render_image_with_mp_limit_async<R: AsyncRead + AsyncSeek + Unpin + Send + Sync + 'static>(
        &self,
        source: Arc<Mutex<R>>,
        max_megapixels: f64,
    ) -> CloudTiffResult<Raster> {
        let ar = self.aspect_ratio();
        let mp = max_megapixels.min(self.full_megapixels());
        let height = (mp * 1e6 / ar).sqrt();
        let width = ar * height;
        self.render_image_region_async(source, (0.0, 0.0, 1.0, 1.0), (width as u32, height as u32))
            .await
    }

    pub async fn render_region_lat_lon_deg_async<R: AsyncRead + AsyncSeek + Unpin>(
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
        self.render_region_async(stream, epsg, region, dimensions)
            .await
    }

    pub async fn render_region_async<R: AsyncRead + AsyncSeek + Unpin>(
        &self,
        stream: Arc<Mutex<R>>,
        epsg: u16,
        region: (f64, f64, f64, f64),
        dimensions: (u32, u32),
    ) -> CloudTiffResult<Raster> {
        let (level, deviation) = self.get_render_info(epsg, &region, &dimensions)?;
        println!("deviation: {deviation}");
        let pixel_map = self.get_pixel_map_correct(level, epsg, &region, &dimensions)?;
        self.render_pixel_map_async(stream, level, pixel_map, dimensions)
            .await
    }

    pub async fn render_image_region_async<R: AsyncRead + AsyncSeek + Unpin + Send + Sync + 'static>(
        &self,
        source: Arc<Mutex<R>>,
        region: (f64, f64, f64, f64),
        dimensions: (u32, u32),
    ) -> CloudTiffResult<Raster> {
        // Determine render layer
        let (left, top, right, bottom) = region;
        let min_level_dims = (
            ((dimensions.0 as f64) / (right - left)).ceil() as u32,
            ((dimensions.1 as f64) / (top - bottom)).ceil() as u32,
        );
        let level = self
            .levels
            .iter()
            .rev()
            .find(|level| {
                level.dimensions.0 > min_level_dims.0 && level.dimensions.1 > min_level_dims.1
            })
            .unwrap_or(&self.levels[0]);

        // Output Raster
        let mut render_raster = Raster::blank(
            dimensions.clone(),
            level.bits_per_sample.clone(),
            level.interpretation,
            level.endian,
        );

        // Tiles
        let mut tile_cache: HashMap<usize, Raster> = HashMap::new(); // TODO stream rather than cache
        let tile_results = level
            .stream_tiles_in_region(source.clone(), region.clone())
            .await;

        // while let Some(result) = join_set.join_next().await {
        for result in tile_results {
            match result {
                Ok(Ok((tile, index, _bounds))) => {
                    tile_cache.insert(index, tile);
                }
                Ok(Err(e)) => {
                    println!("Failed to get tile: {e:?}")
                }
                Err(e) => println!("Failed to join task: {e:?}"),
            }
        }

        // Render
        let dxdi = (bottom - top) / dimensions.1 as f64;
        let mut y = top;
        let dydj = (bottom - top) / dimensions.1 as f64;
        for j in 0..dimensions.1 {
            let mut x = top;
            for i in 0..dimensions.0 {
                let (tile_index, u, v) = level.index_from_image_coords(x, y)?; // TODO cache tiles
                if let Some(tile) = tile_cache.get(&tile_index) {
                    if let Some(pixel) = tile.get_pixel(u as u32, v as u32) {
                        let _ = render_raster.put_pixel(i, j, pixel);
                    }
                }
                x += dxdi;
            }
            y += dydj;
        }

        Ok(render_raster)
    }

    pub async fn get_tile_at_lat_lon_async<R: AsyncRead + AsyncSeek + Unpin>(
        &self,
        stream: Arc<Mutex<R>>,
        level: usize,
        lat: f64,
        lon: f64,
    ) -> CloudTiffResult<Raster> {
        let (x, y) = self.projection.transform_from_lat_lon_deg(lat, lon)?;
        let level = self.get_level(level)?;
        level.get_tile_at_image_coords_async(stream, x, y).await
    }

    async fn render_pixel_map_async<R: AsyncRead + AsyncSeek + Unpin>(
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
