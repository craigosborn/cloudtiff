use super::tiles::TileCache;
use super::CloudTiffResult;
use super::{tiles, util, wmts};
use super::{RenderBuilder, RenderRegion, SyncReader};
use crate::cog::Level;
use crate::cog::{Region, UnitFloat};
use crate::raster::Raster;
use image::{DynamicImage, Rgba, RgbaImage};
use std::collections::HashMap;
use std::fs;
use std::time::Instant;
use tracing::{debug, info, warn};

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

    #[cfg(feature = "image")]
    pub fn render_wmts_tile_tree(
        &self,
        tile_dimensions: (u32, u32),
        path_template: &str,
    ) -> CloudTiffResult<usize> {
        let epsg = 4326;
        let cog_bounds = self.cog.bounds_lat_lon_deg()?;
        let tree_indices =
            wmts::tile_tree_indices(cog_bounds, self.cog.full_dimensions(), tile_dimensions);
        let mut tile_cache = TileCache::new();
        let mut previous_level = None;
        let n = tree_indices.len();
        for (i, (x, y, z)) in tree_indices.into_iter().enumerate() {
            let t_tile = Instant::now();
            let Some(region_degrees) = wmts::tile_bounds_lat_lon_deg(x, y, z) else {
                warn!("bad wmts tree index: {:?}", (x, y, z));
                continue;
            };
            let region = region_degrees * 1_f64.to_radians();
            let level = util::render_level_from_region(self.cog, epsg, &region, &tile_dimensions)?;
            let pixel_map = match util::project_pixel_map(
                level,
                &self.input_projection,
                epsg,
                &region,
                &tile_dimensions,
            ) {
                Ok(pm) => pm,
                Err(e) => {
                    warn!("Bad Pixel Mapping on tile {:?}: {e:?}", (x, y, z));
                    continue;
                }
            };
            if level.overview != previous_level {
                tile_cache.clear();
                previous_level = level.overview;
            }
            let indices = pixel_map
                .iter()
                .map(|(i, _)| *i)
                .filter(|i| !tile_cache.contains_key(&i))
                .collect();
            let new_tiles = tiles::get_tiles(&self.reader, level, indices);
            tile_cache.extend(new_tiles);
            let raster = match render_pixel_map(&pixel_map, level, &tile_cache, &tile_dimensions) {
                Ok(r) => r,
                Err(e) => {
                    warn!("bad pixel map: {e:?}");
                    continue;
                }
            };
            let tile_name = path_template
                .replace("{x}", &x.to_string())
                .replace("{y}", &y.to_string())
                .replace("{z}", &z.to_string());
            let img: DynamicImage = match raster.try_into() {
                Ok(i) => i,
                Err(e) => {
                    warn!("Failed to convert Raster to DynamicImage: {e:?}");
                    continue;
                }
            };

            match img.save(tile_name) {
                Ok(_) => {
                    let dt = t_tile.elapsed().as_secs_f32();
                    let fps = 1.0 / dt;
                    info!(
                        "{i}/{n} rendered tile {:?} in {}ms ({:.1}fps)",
                        (x, y, z),
                        dt * 1e3,
                        fps
                    );
                }
                Err(e) => warn!("Failed to save tile: {e}"),
            }
        }
        Ok(n)
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

        #[cfg(feature = "image")]
        pub async fn render_wmts_tile_tree_async(
            &self,
            tile_dimensions: (u32, u32),
            path_template: &str,
        ) -> CloudTiffResult<usize> {
            use std::path::PathBuf;

            let epsg = 4326;
            let cog_bounds = self.cog.bounds_lat_lon_deg()?;
            let cog_dimensions = self.cog.full_dimensions();
            let tree_indices = wmts::tile_tree_indices(cog_bounds, cog_dimensions, tile_dimensions);
            let mut tile_cache = TileCache::new();
            let mut previous_level = None;
            let n = tree_indices.len();
            let path_template_path = PathBuf::from(path_template);
            let Some(folder) = path_template_path.parent() else {
                return Err(crate::CloudTiffError::BadPath(path_template.to_string()));
            };
            println!("folder: {folder:?}");
            let _ = fs::create_dir_all(folder);
            let similarity_valid = util::is_simililarity_valid(
                &self.input_projection,
                epsg,
                &(cog_bounds * 1_f64.to_radians()),
                &cog_dimensions,
            );

            for (i, (x, y, z)) in tree_indices.into_iter().enumerate() {
                let xt = Instant::now();
                let t_tile = Instant::now();
                let Some(region_degrees) = wmts::tile_bounds_lat_lon_deg(x, y, z) else {
                    warn!("bad wmts tree index: {:?}", (x, y, z));
                    continue;
                };
                debug!("wmts bounds: {}us", xt.elapsed().as_micros());
                let xt = Instant::now();

                let region = region_degrees * 1_f64.to_radians();
                let level =
                    util::render_level_from_region(self.cog, epsg, &region, &tile_dimensions)?;

                debug!("render level: {}us", xt.elapsed().as_micros());
                let xt = Instant::now();

                let pixel_map_result = match &similarity_valid {
                    Some(_) => util::project_pixel_map_simililarity(
                        level,
                        &self.input_projection,
                        epsg,
                        &region,
                        &tile_dimensions,
                    ),
                    None => util::project_pixel_map(
                        level,
                        &self.input_projection,
                        epsg,
                        &region,
                        &tile_dimensions,
                    ),
                };

                let pixel_map = match pixel_map_result {
                    Ok(pm) => pm,
                    Err(e) => {
                        warn!("Bad Pixel Mapping on tile {:?}: {e:?}", (x, y, z));
                        continue;
                    }
                };

                debug!("project pixelmap: {}us", xt.elapsed().as_micros());
                let xt = Instant::now();

                if level.overview != previous_level {
                    tile_cache.clear();
                    previous_level = level.overview;
                }
                let indices = pixel_map
                    .iter()
                    .map(|(i, _)| *i)
                    .filter(|i| !tile_cache.contains_key(&i))
                    .collect();
                let new_tiles = tiles::get_tiles_async(&self.reader, level, indices).await;
                tile_cache.extend(new_tiles);

                debug!("tile cache: {}us", xt.elapsed().as_micros());
                let xt = Instant::now();

                let img = match render_pixel_map_rgba(&pixel_map, &tile_cache, &tile_dimensions) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("bad pixel map: {e:?}");
                        continue;
                    }
                };

                debug!("render pixelmap: {}us", xt.elapsed().as_micros());
                let xt = Instant::now();

                let tile_name = path_template
                    .replace("{x}", &x.to_string())
                    .replace("{y}", &y.to_string())
                    .replace("{z}", &z.to_string());

                match img.save(tile_name) {
                    Ok(_) => {
                        let dt = t_tile.elapsed().as_secs_f32();
                        let fps = 1.0 / dt;
                        info!(
                            "{i}/{n} rendered tile {:?} in {}ms ({:.1}fps)",
                            (x, y, z),
                            dt * 1e3,
                            fps
                        );
                    }
                    Err(e) => warn!("Failed to save tile: {e}"),
                }

                debug!("save img: {}us", xt.elapsed().as_micros());
            }
            Ok(n)
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

fn render_pixel_map_rgba(
    pixel_map: &util::PixelMap,
    tile_cache: &HashMap<usize, Raster>,
    dimensions: &(u32, u32),
) -> CloudTiffResult<RgbaImage> {
    let mut render_rgba = RgbaImage::from_pixel(dimensions.0, dimensions.1, Rgba([0, 0, 0, 0]));
    for (tile_index, tile_pixel_map) in pixel_map.iter() {
        if let Some(tile) = tile_cache.get(tile_index) {
            for (from, to) in tile_pixel_map {
                // TODO interpolation methods other than "floor"
                if let Some(pixel) = tile.get_pixel_rgba(from.0 as u32, from.1 as u32) {
                    render_rgba.put_pixel(to.0, to.1, pixel);
                }
            }
        }
    }
    Ok(render_rgba)
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
