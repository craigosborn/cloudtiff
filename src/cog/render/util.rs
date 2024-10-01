use crate::cog::{CloudTiff, Level};
use crate::raster::Raster;
use std::collections::HashMap;

pub fn get_render_level(
    cog: &CloudTiff,
    region: (f64, f64, f64, f64),
    dimensions: (u32, u32),
) -> &Level {
    let (left, top, right, bottom) = region;
    let min_level_dims = (
        ((dimensions.0 as f64) / (right - left)).ceil() as u32,
        ((dimensions.1 as f64) / (top - bottom)).ceil() as u32,
    );
    cog.levels
        .iter()
        .rev()
        .find(|level| {
            level.dimensions.0 > min_level_dims.0 && level.dimensions.1 > min_level_dims.1
        })
        .unwrap_or(&cog.levels[0])
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

pub fn tile_info_from_indices(level: &Level, indices: Vec<usize>) -> Vec<(usize, (u64, u64))> {
    indices
        .into_iter()
        .filter_map(|index| match level.tile_byte_range(index) {
            Ok(range) => Some((index, range)),
            Err(e) => {
                println!("Failed to get tile byte range: {e:?}");
                None
            }
        })
        .collect()
}

pub fn resolution_from_mp_limit(max_dimensions: (u32, u32), max_megapixels: f64) -> (u32, u32) {
    let ar = max_dimensions.0 as f64 / max_dimensions.1 as f64;
    let max_pixels = max_dimensions.0 as f64 * max_dimensions.1 as f64;
    let height = ((max_megapixels * 1e6).min(max_pixels) / ar).sqrt();
    let width = ar * height;
    (width as u32, height as u32)
}
