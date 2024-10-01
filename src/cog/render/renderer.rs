use super::CloudTiffResult;
use super::{tiles, util};
use super::{AsyncReader, RenderBuilder, SyncReader};
use crate::raster::Raster;
use std::collections::HashMap;

pub type TileCache = HashMap<usize, Raster>;

impl<'a> RenderBuilder<'a, SyncReader> {
    pub fn render(&self) -> CloudTiffResult<Raster> {
        let region = self.input_region.as_f64();

        // Tiles
        let level = util::get_render_level(self.cog, region, self.output_resolution);
        let indices = level.tile_indices_within_image_region(region);
        let tile_cache = tiles::get_tiles(&self.reader, level, indices);

        // Render
        Ok(util::render_image_region_from_tile_cache(
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

    impl<'a> RenderBuilder<'a, AsyncReader> {
        pub async fn render_async(&'a self) -> CloudTiffResult<Raster> {
            let region = self.input_region.as_f64();

            // Tiles
            let level = util::get_render_level(self.cog, region, self.output_resolution);
            let indices = level.tile_indices_within_image_region(region);
            let tile_cache: HashMap<usize, Raster> =
                tiles::get_tiles_async(&self.reader, level, indices).await;

            // Render
            Ok(util::render_image_region_from_tile_cache(
                &tile_cache,
                level,
                region,
                self.output_resolution,
            ))
        }
    }
}
