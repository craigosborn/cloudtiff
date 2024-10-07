use std::collections::HashMap;

use crate::cog::projection::ProjectionError;
use crate::cog::{CloudTiff, CloudTiffResult, Level, Projection};
use crate::cog::{Region, UnitFloat};
use crate::CloudTiffError;
use proj4rs::Proj;
use tracing::*;

pub type PixelMap = HashMap<usize, Vec<((f64, f64), (u32, u32))>>;

const MAX_PIXEL_DEVIATION: f64 = 1.0;

pub fn render_level_from_crop<'a>(
    cog: &'a CloudTiff,
    crop: &Region<UnitFloat>,
    dimensions: &(u32, u32),
) -> &'a Level {
    let (left, top, right, bottom) = crop.to_f64();
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

pub fn render_level_from_region<'a>(
    cog: &'a CloudTiff,
    epsg: u16,
    region: &Region<f64>,
    dimensions: &(u32, u32),
) -> CloudTiffResult<&'a Level> {
    let (left, top, ..) = cog
        .projection
        .transform_from(region.x.min, region.y.min, 0.0, epsg)?;
    let (right, bottom, ..) =
        cog.projection
            .transform_from(region.x.max, region.y.max, 0.0, epsg)?;

    // Determine render level
    //   TODO this method not accurate if projections are not aligned
    let pixel_scale_x = (right - left).abs() / dimensions.0 as f64;
    let pixel_scale_y = (top - bottom).abs() / dimensions.1 as f64;
    let min_pixel_scale = pixel_scale_x.min(pixel_scale_y);
    let level_scales = cog.pixel_scales();
    let level_index = level_scales
        .iter()
        .enumerate()
        .rev()
        .find(|(_, (level_scale_x, level_scale_y))| {
            level_scale_x.max(*level_scale_y) < min_pixel_scale
        })
        .map(|(i, _)| i)
        .unwrap_or(0);

    cog.get_level(level_index)
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

pub fn project_pixel_map(
    level: &Level,
    projection: &Projection,
    epsg: u16,
    region: &Region<f64>,
    dimensions: &(u32, u32),
) -> CloudTiffResult<PixelMap> {
    let mut pixel_map = HashMap::new();
    let output_proj = Proj::from_epsg_code(epsg).map_err(|e| ProjectionError::from(e))?;
    let dxdi = region.x.range() / dimensions.0 as f64;
    let dydj = region.y.range() / dimensions.1 as f64;
    for j in 0..dimensions.1 {
        for i in 0..dimensions.0 {
            let x = region.x.min + dxdi * i as f64;
            let y = region.y.max - dydj * j as f64;
            match projection.transform_from_proj(&output_proj, x, y, 0.0) {
                Ok((u, v, ..)) => {
                    if let Ok((tile_index, tile_x, tile_y)) = level.index_from_image_coords(u, v) {
                        let tile_pixel_map = pixel_map.entry(tile_index).or_insert(vec![]);
                        tile_pixel_map.push(((tile_x, tile_y), (i, j)));
                    }
                }
                Err(e) => warn!("pixel transform: {e:?}"),
            }
        }
    }
    if pixel_map.len() == 0 {
        return Err(CloudTiffError::RegionOutOfBounds((
            region.as_tuple(),
            projection.bounds_in_proj(&output_proj)?.as_tuple(),
        )));
    } else {
        Ok(pixel_map)
    }
}

pub fn is_simililarity_valid(
    projection: &Projection,
    epsg: u16,
    region: &Region<f64>,
    dimensions: &(u32, u32),
) -> Option<f64> {
    // println!("region: {region:?}");
    let output_proj = Proj::from_epsg_code(epsg)
        .map_err(|e| ProjectionError::from(e))
        .ok()?;
    let origin = projection
        .transform_from_proj(&output_proj, region.x.min, region.y.min, 0.0)
        .ok()?;
    let right = projection
        .transform_from_proj(&output_proj, region.x.max, region.y.min, 0.0)
        .ok()?;
    let down = projection
        .transform_from_proj(&output_proj, region.x.min, region.y.max, 0.0)
        .ok()?;
    let across = projection
        .transform_from_proj(&output_proj, region.x.max, region.y.max, 0.0)
        .ok()?;

    let dudx = (right.0 - origin.0) / region.x.range();
    let dvdx = (right.1 - origin.1) / region.x.range();
    let dudy = (down.0 - origin.0) / region.y.range();
    let dvdy = (down.1 - origin.1) / region.y.range();

    let projected_across = (
        origin.0 + dudx * region.x.range() + dudy * region.y.range(),
        origin.1 + dvdx * region.x.range() + dvdy * region.y.range(),
    );
    let deviation =
        ((projected_across.0 - across.0).powi(2) + (projected_across.1 - across.1).powi(2)).sqrt();
    let pixel_size = ((across.0 - origin.0).powi(2) + (across.1 - origin.1).powi(2)).sqrt()
        / ((dimensions.0 as f64).powi(2) + (dimensions.1 as f64).powi(2)).sqrt();
    let relative_deviation = deviation / pixel_size;

    if relative_deviation <= MAX_PIXEL_DEVIATION {
        Some(relative_deviation)
    } else {
        None
    }
}

pub fn project_pixel_map_simililarity(
    level: &Level,
    projection: &Projection,
    epsg: u16,
    region: &Region<f64>,
    dimensions: &(u32, u32),
) -> CloudTiffResult<PixelMap> {
    let mut pixel_map = HashMap::new();

    let output_proj = Proj::from_epsg_code(epsg).map_err(|e| ProjectionError::from(e))?;
    let origin = projection.transform_from_proj(&output_proj, region.x.min, region.y.max, 0.0)?;
    let right = projection.transform_from_proj(&output_proj, region.x.max, region.y.max, 0.0)?;
    let down = projection.transform_from_proj(&output_proj, region.x.min, region.y.min, 0.0)?;
    
    let dudx = (right.0 - origin.0) / region.x.range();
    let dvdx = (right.1 - origin.1) / region.x.range();
    let dudy = (down.0 - origin.0) / region.y.range();
    let dvdy = (down.1 - origin.1) / region.y.range();

    let dxdi = region.x.range() / dimensions.0 as f64;
    let dydj = region.y.range() / dimensions.1 as f64;
    for j in 0..dimensions.1 {
        let dy = dydj * j as f64;
        for i in 0..dimensions.0 {
            let dx = dxdi * i as f64;
            let u = origin.0 + dudx * dx + dudy * dy;
            let v = origin.1 + dvdx * dx + dvdy * dy;
            if let Ok((tile_index, tile_x, tile_y)) = level.index_from_image_coords(u, v) {
                let tile_pixel_map = pixel_map.entry(tile_index).or_insert(vec![]);
                tile_pixel_map.push(((tile_x, tile_y), (i, j)));
            }
        }
    }
    Ok(pixel_map)
}
