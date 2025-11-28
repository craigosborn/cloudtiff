#![cfg(feature = "async")]

use super::renderer;
use super::CloudTiffResult;
use super::{tiles, util};
use super::{RenderBuilder, RenderRegion};
use crate::raster::Raster;
use crate::AsyncReadRange;
use std::collections::HashMap;
use std::sync::Arc;

pub struct AsyncRender<'c, R> {
    config: RenderBuilder<'c>,
    reader: Arc<R>,
}

impl<'c, R> AsyncRender<'c, R>
where
    R: AsyncReadRange + 'static,
{
    pub fn new(config: RenderBuilder<'c>, reader: R) -> Self {
        Self {
            config,
            reader: Arc::new(reader),
        }
    }

    pub async fn render(self) -> CloudTiffResult<Raster> {
        let Self { config, reader } = self;

        let dimensions = config.resolution;
        match config.region {
            RenderRegion::InputCrop(crop) => {
                let level = util::render_level_from_crop(config.cog, &crop, &dimensions);
                let indices = level.tile_indices_within_image_crop(crop);
                let tile_cache: HashMap<usize, Raster> =
                    tiles::get_tiles_async(reader.clone(), level, indices).await;
                Ok(renderer::render_image_crop_from_tile_cache(
                    &tile_cache,
                    level,
                    &crop,
                    &dimensions,
                ))
            }
            RenderRegion::OutputRegion((epsg, region)) => {
                let level = util::render_level_from_region(config.cog, epsg, &region, &dimensions)?;
                let pixel_map = util::project_pixel_map(
                    &level,
                    &config.input_projection,
                    epsg,
                    &region,
                    &dimensions,
                )?;
                let indices = pixel_map.keys().copied().collect();
                let tile_cache = tiles::get_tiles_async(reader.clone(), &level, indices).await;
                renderer::render_pixel_map(&pixel_map, &level, &tile_cache, &dimensions)
            }
            RenderRegion::Tile((x, y, z)) => {
                let level = config.cog.get_level(z)?;
                let index = level.tile_index(y, x);
                tiles::get_tile_async(reader.clone(), level, index).await
            }
        }
    }
}
