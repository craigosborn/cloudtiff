use super::CloudTiffResult;
use super::{tiles, util};
use super::{RenderBuilder, RenderRegion, SyncReader};
use crate::cog::Level;
use crate::raster::Raster;
use crate::{Region, UnitFloat};
use std::collections::HashMap;

impl<'a> RenderBuilder<'a, SyncReader> {
    pub fn render(&self) -> CloudTiffResult<Raster> {
        let dimensions = self.resolution;
        match self.region {
            RenderRegion::InputCrop(crop) => {
                let level = util::render_level_from_crop(self.cog, &crop, &dimensions);
                let indices = level.tile_indices_within_image_crop(crop);
                let tile_cache = tiles::get_tiles(&self.reader, level, indices);
                Ok(render_image_crop_from_tile_cache(
                    &tile_cache,
                    level,
                    &crop,
                    &dimensions,
                ))
            }
            RenderRegion::OutputRegion((epsg, region)) => {
                let level = util::render_level_from_region(self.cog, epsg, &region, &dimensions)?;
                let pixel_map = util::project_pixel_map(
                    level,
                    &self.input_projection,
                    epsg,
                    &region,
                    &dimensions,
                )?;
                let indices = pixel_map.iter().map(|(i, _)| *i).collect();
                let tile_cache = tiles::get_tiles(&self.reader, level, indices);
                render_pixel_map(&pixel_map, level, &tile_cache, &dimensions)
            }
        }
    }
}

#[cfg(feature = "async")]
mod not_sync {
    use super::super::AsyncReader;
    use super::*;

    impl<'a> RenderBuilder<'a, AsyncReader> {
        pub async fn render_async(&'a self) -> CloudTiffResult<Raster> {
            let dimensions = self.resolution;
            match self.region {
                RenderRegion::InputCrop(crop) => {
                    let level = util::render_level_from_crop(self.cog, &crop, &dimensions);
                    let indices = level.tile_indices_within_image_crop(crop);
                    let tile_cache: HashMap<usize, Raster> =
                        tiles::get_tiles_async(&self.reader, level, indices).await;
                    Ok(render_image_crop_from_tile_cache(
                        &tile_cache,
                        level,
                        &crop,
                        &dimensions,
                    ))
                }
                RenderRegion::OutputRegion((epsg, region)) => {
                    let level =
                        util::render_level_from_region(self.cog, epsg, &region, &dimensions)?;
                    let pixel_map = util::project_pixel_map(
                        level,
                        &self.input_projection,
                        epsg,
                        &region,
                        &dimensions,
                    )?;
                    let indices = pixel_map.iter().map(|(i, _)| *i).collect();
                    let tile_cache = tiles::get_tiles_async(&self.reader, level, indices).await;
                    render_pixel_map(&pixel_map, level, &tile_cache, &dimensions)
                }
            }
        }
    }
}

pub fn render_image_crop_from_tile_cache(
    tile_cache: &HashMap<usize, Raster>,
    level: &Level,
    crop: &Region<UnitFloat>,
    dimensions: &(u32, u32),
) -> Raster {
    let mut render_raster = Raster::blank(
        dimensions.clone(),
        level.bits_per_sample.clone(),
        level.interpretation,
        level.sample_format.clone(),
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

fn render_pixel_map(
    pixel_map: &util::PixelMap,
    level: &Level,
    tile_cache: &HashMap<usize, Raster>,
    dimensions: &(u32, u32),
) -> CloudTiffResult<Raster> {
    let mut render_raster = Raster::blank(
        dimensions.clone(),
        level.bits_per_sample.clone(),
        level.interpretation,
        level.sample_format.clone(),
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
