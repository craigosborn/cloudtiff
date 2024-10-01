use super::{tiles_async, util};
use crate::cog::{CloudTiff, CloudTiffResult};
use crate::raster::Raster;
use crate::AsyncReader;

pub async fn render_image_region_async<'a>(
    cog: &'a CloudTiff,
    reader: AsyncReader,
    region: (f64, f64, f64, f64),
    dimensions: (u32, u32),
) -> CloudTiffResult<Raster> {
    // Tiles
    let level = util::get_render_level(cog, region, dimensions);
    let indices = level.tile_indices_within_image_region(region);
    let tile_cache = tiles_async::get_tiles_async(reader, level, indices).await;

    // Render
    Ok(util::render_image_region_from_tile_cache(
        &tile_cache,
        level,
        region,
        dimensions,
    ))
}
