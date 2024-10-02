use crate::cog::{CloudTiff, Level};
use tracing::*;

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

pub fn tile_info_from_indices(level: &Level, indices: Vec<usize>) -> Vec<(usize, (u64, u64))> {
    indices
        .into_iter()
        .filter_map(|index| match level.tile_byte_range(index) {
            Ok(range) => Some((index, range)),
            Err(e) => {
                warn!("Failed to get tile byte range: {e:?}");
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
