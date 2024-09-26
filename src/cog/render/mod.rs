use super::{CloudTiff, CloudTiffError, CloudTiffResult, Level};
use crate::raster::Raster;
use std::collections::HashMap;
use std::io::{Read, Seek};

#[cfg(feature = "async")]
mod concurrent;

type PixelMap = HashMap<usize, Vec<((f64, f64), (u32, u32))>>;

impl CloudTiff {
    pub fn render_region_lat_lon_deg<R: Read + Seek>(
        &self,
        stream: &mut R,
        nwse: (f64, f64, f64, f64),
        dimensions: (u32, u32),
    ) -> CloudTiffResult<Raster> {
        let epsg = 4326;
        let (north, west, south, east) = nwse;
        let region = (
            west.to_radians(),
            north.to_radians(),
            east.to_radians(),
            south.to_radians(),
        );
        self.render_region(stream, epsg, region, dimensions)
    }

    pub fn render_region<R: Read + Seek>(
        &self,
        stream: &mut R,
        epsg: u16,
        region: (f64, f64, f64, f64),
        dimensions: (u32, u32),
    ) -> CloudTiffResult<Raster> {
        let level = self.get_render_level(epsg, &region, &dimensions)?;
        let pixel_map = self.get_pixel_map_correct(level, epsg, &region, &dimensions)?;
        self.render_pixel_map(stream, level, pixel_map, dimensions)
    }

    fn get_render_level(
        &self,
        epsg: u16,
        region: &(f64, f64, f64, f64),
        dimensions: &(u32, u32),
    ) -> CloudTiffResult<&Level> {
        let (left, top, ..) = self
            .projection
            .transform_from(region.0, region.1, 0.0, epsg)?;
        let (right, bottom, ..) = self
            .projection
            .transform_from(region.2, region.3, 0.0, epsg)?;

        // Determine render level
        //   TODO this method not accurate if projections are not aligned
        let pixel_scale_x = (right - left).abs() / dimensions.0 as f64;
        let pixel_scale_y = (top - bottom).abs() / dimensions.1 as f64;
        let min_pixel_scale = pixel_scale_x.min(pixel_scale_y);
        let level_scales = self.pixel_scales();
        let level_index = level_scales
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (level_scale_x, level_scale_y))| {
                level_scale_x.max(*level_scale_y) < min_pixel_scale
            })
            .map(|(i, _)| i)
            .unwrap_or(0);

        self.get_level(level_index)
    }

    fn render_pixel_map<R: Read + Seek>(
        &self,
        stream: &mut R,
        level: &Level,
        pixel_map: PixelMap,
        dimensions: (u32, u32),
    ) -> CloudTiffResult<Raster> {
        let mut render_raster = Raster::blank(
            dimensions.clone(),
            level.bits_per_sample.clone(),
            level.interpretation,
            level.endian,
        );
        for (tile_index, tile_pixel_map) in pixel_map.iter() {
            if let Ok(tile) = level.get_tile_by_index(stream, *tile_index) {
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

    fn get_pixel_map_correct(
        &self,
        level: &Level,
        epsg: u16,
        region: &(f64, f64, f64, f64),
        dimensions: &(u32, u32),
    ) -> CloudTiffResult<PixelMap> {
        let mut pixel_map = HashMap::new();
        let dxdi = (region.2 - region.0) / dimensions.0 as f64;
        let dydj = (region.3 - region.1) / dimensions.1 as f64;
        for j in 0..dimensions.1 {
            for i in 0..dimensions.0 {
                let x = region.0 + dxdi * i as f64;
                let y = region.1 + dydj * j as f64;
                match self.projection.transform_from(x, y, 0.0, epsg) {
                    Ok((u, v, ..)) => {
                        // println!("transform: ({x:.6}, {y:.6}) -> ({u:.1}, {v:.1})");
                        if let Ok((tile_index, tile_x, tile_y)) =
                            level.index_from_image_coords(u, v)
                        {
                            let tile_pixel_map = pixel_map.entry(tile_index).or_insert(vec![]);
                            tile_pixel_map.push(((tile_x, tile_y), (i, j)));
                        }
                    }
                    Err(e) => println!("pixel transform: {e:?}"),
                }
            }
        }
        if pixel_map.len() == 0 {
            return Err(CloudTiffError::RegionOutOfBounds((
                region.clone(),
                self.projection.bounds(epsg)?,
            )));
        } else {
            Ok(pixel_map)
        }
    }
}
