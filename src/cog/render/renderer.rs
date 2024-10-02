use super::CloudTiffResult;
use super::{tiles, util};
use super::{RenderBuilder, SyncReader};
use crate::cog::Level;
use crate::raster::Raster;
use std::collections::HashMap;

impl<'a> RenderBuilder<'a, SyncReader> {
    pub fn render(&self) -> CloudTiffResult<Raster> {
        let region = self.input_region.as_f64();

        // Tiles
        let level = util::get_render_level(self.cog, region, self.output_resolution);
        let indices = level.tile_indices_within_image_region(region);
        let tile_cache = tiles::get_tiles(&self.reader, level, indices);

        // Render
        Ok(render_image_region_from_tile_cache(
            &tile_cache,
            level,
            region,
            self.output_resolution,
        ))
    }
}

#[cfg(feature = "async")]
mod not_sync {
    use super::*;
    use super::super::AsyncReader;

    impl<'a> RenderBuilder<'a, AsyncReader> {
        pub async fn render_async(&'a self) -> CloudTiffResult<Raster> {
            let region = self.input_region.as_f64();

            // Tiles
            let level = util::get_render_level(self.cog, region, self.output_resolution);
            let indices = level.tile_indices_within_image_region(region);
            let tile_cache: HashMap<usize, Raster> =
                tiles::get_tiles_async(&self.reader, level, indices).await;

            // Render
            Ok(render_image_region_from_tile_cache(
                &tile_cache,
                level,
                region,
                self.output_resolution,
            ))
        }
    }
}

pub fn render_image_region_from_tile_cache(
    tile_cache: &HashMap<usize, Raster>,
    level: &Level,
    region: (f64, f64, f64, f64),
    dimensions: (u32, u32),
) -> Raster {
    let mut render_raster = Raster::blank(
        dimensions.clone(),
        level.bits_per_sample.clone(),
        level.interpretation,
        level.endian,
    );
    let (left, top, right, bottom) = region;
    let dxdi = (right - left) / dimensions.0 as f64;
    let mut y = top;
    let dydj = (bottom - top) / dimensions.1 as f64;
    for j in 0..dimensions.1 {
        let mut x = top;
        for i in 0..dimensions.0 {
            if let Ok((tile_index, u, v)) = level.index_from_image_coords(x, y) {
                if let Some(tile) = tile_cache.get(&tile_index) {
                    if let Some(pixel) = tile.get_pixel(u as u32, v as u32) {
                        let _ = render_raster.put_pixel(i, j, pixel);
                    }
                }
            }
            x += dxdi;
        }
        y += dydj;
    }
    render_raster
}
