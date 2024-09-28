use super::{CloudTiff, CloudTiffError, CloudTiffResult, Level};
use crate::raster::Raster;
use std::collections::HashMap;
use std::io::{Read, Seek};

pub type PixelMap = HashMap<usize, Vec<((f64, f64), (u32, u32))>>;

impl CloudTiff {
    pub fn render_image_with_mp_limit<R: Read + Seek>(
        &self,
        stream: &mut R,
        max_megapixels: f64,
    ) -> CloudTiffResult<Raster> {
        let ar = self.aspect_ratio();
        let mp = max_megapixels.min(self.full_megapixels());
        let height = (mp * 1e6 / ar).sqrt();
        let width = ar * height;
        self.render_image_region(stream, (0.0, 0.0, 1.0, 1.0), (width as u32, height as u32))
    }

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
        let (level, deviation) = self.get_render_info(epsg, &region, &dimensions)?;
        println!("deviation: {deviation}");
        let pixel_map = self.get_pixel_map_correct(level, epsg, &region, &dimensions)?;
        self.render_pixel_map(stream, level, pixel_map, dimensions)
    }

    pub fn render_image_region<R: Read + Seek>(
        &self,
        stream: &mut R,
        region: (f64, f64, f64, f64),
        dimensions: (u32, u32),
    ) -> CloudTiffResult<Raster> {
        // Determine render layer
        let (left, top, right, bottom) = region;
        let min_level_dims = (
            ((dimensions.0 as f64) / (right - left)).ceil() as u32,
            ((dimensions.1 as f64) / (top - bottom)).ceil() as u32,
        );
        let level = self
            .levels
            .iter()
            .rev()
            .find(|level| {
                level.dimensions.0 > min_level_dims.0 && level.dimensions.1 > min_level_dims.1
            })
            .unwrap_or(&self.levels[0]);
        println!("Level: {level}");

        // Output Raster
        let mut render_raster = Raster::blank(
            dimensions.clone(),
            level.bits_per_sample.clone(),
            level.interpretation,
            level.endian,
        );

        // Tiles
        let mut tile_cache: HashMap<usize, Raster> = HashMap::new();
        for index in level.tile_indices_within_image_region(region).iter() {
            match level.get_tile_by_index(stream, *index) {
                Ok(tile) => {tile_cache.insert(*index, tile);},
                Err(e) => println!("Failed to get tile {index}: {e:?}"),
            }
        }

        // Render
        let dxdi = (bottom - top) / dimensions.1 as f64;
        let mut y = top;
        let dydj = (bottom - top) / dimensions.1 as f64;
        for j in 0..dimensions.1 {
            let mut x = top;
            for i in 0..dimensions.0 {
                let (tile_index, u, v) = level.index_from_image_coords(x, y)?; // TODO cache tiles
                if let Some(tile) = tile_cache.get(&tile_index) {
                    if let Some(pixel) = tile.get_pixel(u as u32, v as u32) {
                        let _ = render_raster.put_pixel(i, j, pixel);
                    }
                }
                x += dxdi;
            }
            y += dydj;
        }

        Ok(render_raster)
    }

    pub fn get_tile_at_lat_lon<R: Read + Seek>(
        &self,
        stream: &mut R,
        level: usize,
        lat: f64,
        lon: f64,
    ) -> CloudTiffResult<Raster> {
        let (x, y) = self.projection.transform_from_lat_lon_deg(lat, lon)?;
        let level = self.get_level(level)?;
        level.get_tile_at_image_coords(stream, x, y)
    }

    pub fn get_render_info(
        &self,
        epsg: u16,
        region: &(f64, f64, f64, f64),
        dimensions: &(u32, u32),
    ) -> CloudTiffResult<(&Level, f64)> {
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
        let level = self.level_at_pixel_scale(min_pixel_scale)?;

        // Determine deviation of similarity approximation
        let (test_left, test_bottom, _) = self
            .projection
            .transform_from(region.0, region.3, 0.0, epsg)?;
        let deviation = (((test_left - left) / pixel_scale_x).powi(2)
            + ((test_bottom - bottom) / pixel_scale_y).powi(2))
        .sqrt();

        Ok((level, deviation))
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

    pub fn get_pixel_map_correct(
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
