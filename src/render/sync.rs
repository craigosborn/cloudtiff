use super::renderer;
use super::CloudTiffResult;
use super::{tiles, util};
use super::{RenderBuilder, RenderRegion};
use crate::raster::Raster;
use crate::ReadRange;

pub struct SyncRender<'c, 'r, R> {
    config: RenderBuilder<'c>,
    reader: &'r R,
}

impl<'c, 'r, R> SyncRender<'c, 'r, R>
where
    R: ReadRange,
{
    pub fn new(config: RenderBuilder<'c>, reader: &'r R) -> Self {
        Self { config, reader }
    }

    pub fn render(self) -> CloudTiffResult<Raster> {
        let Self { config, reader } = self;

        let dimensions = config.resolution;
        match config.region {
            RenderRegion::InputCrop(crop) => {
                let level = util::render_level_from_crop(config.cog, &crop, &dimensions);
                let indices = level.tile_indices_within_image_crop(crop);
                let tile_cache = tiles::get_tiles(self.reader, level, indices);
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
                let tile_cache = tiles::get_tiles(reader, &level, indices);
                renderer::render_pixel_map(&pixel_map, &level, &tile_cache, &dimensions)
            }
            RenderRegion::Tile((x, y, z)) => {
                let level = config.cog.get_level(z)?;
                let index = level.tile_index(y, x);
                tiles::get_tile(reader, level, index)
            }
        }
    }
}
