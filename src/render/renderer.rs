use super::util;
use super::CloudTiffResult;
use crate::cog::Level;
use crate::raster::Raster;
use crate::{Region, UnitFloat};
use std::collections::HashMap;

pub fn render_image_crop_from_tile_cache(
    tile_cache: &HashMap<usize, Raster>,
    level: &Level,
    crop: &Region<UnitFloat>,
    dimensions: &(u32, u32),
) -> Raster {
    let mut render_raster = Raster::blank(
        *dimensions,
        level.bits_per_sample.clone(),
        level.interpretation,
        level.sample_format.clone(),
        level.extra_samples.clone(),
        level.endian,
    );
    let dxdi = crop.x.range().as_f64() / dimensions.0 as f64;
    let mut y = crop.y.min.as_f64();
    let dydj = crop.y.range().as_f64() / dimensions.1 as f64;
    for j in 0..dimensions.1 {
        let mut x = crop.x.min.as_f64();
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

pub fn render_pixel_map(
    pixel_map: &util::PixelMap,
    level: &Level,
    tile_cache: &HashMap<usize, Raster>,
    dimensions: &(u32, u32),
) -> CloudTiffResult<Raster> {
    let mut render_raster = Raster::blank(
        *dimensions,
        level.bits_per_sample.clone(),
        level.interpretation,
        level.sample_format.clone(),
        level.extra_samples.clone(),
        level.endian,
    );
    for (tile_index, tile_pixel_map) in pixel_map.iter() {
        if let Some(tile) = tile_cache.get(tile_index) {
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
