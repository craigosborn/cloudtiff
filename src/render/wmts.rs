use crate::{Point2D, Region};
use std::f64::consts::{PI, TAU};

pub const MAX_LAT_DEG: f64 = 85.06;
pub const MIN_LAT_DEG: f64 = -85.06;

pub fn tile_tree_indices(
    bounds_lat_lon_deg: Region<f64>,
    dimensions: (u32, u32),
    tile_dim: (u32, u32),
) -> Vec<(u32, u32, u32)> {
    let mut tree = vec![];
    let (bounds, (min_z, max_z)) = bounds_wmts(bounds_lat_lon_deg, dimensions, tile_dim);

    for z in min_z..=max_z {
        let tile_bounds = bounds * 2_f64.powi(z as i32);
        for y in tile_bounds.y.min.floor() as u32..tile_bounds.y.max.ceil() as u32 {
            for x in tile_bounds.x.min.floor() as u32..tile_bounds.x.max.ceil() as u32 {
                tree.push((x, y, z));
            }
        }
    }
    tree
}

pub fn bounds_wmts(
    bounds_lat_lon_deg: Region<f64>,
    dimensions: (u32, u32),
    tile_dim: (u32, u32),
) -> (Region<f64>, (u32, u32)) {
    let bounds = bounds_lat_lon_deg;

    // Lateral bounds at zoom 0
    let max_lat = bounds.y.max.clamp(MIN_LAT_DEG, MAX_LAT_DEG);
    let min_lat = bounds.y.min.clamp(MIN_LAT_DEG, MAX_LAT_DEG);
    let north_west = Point2D {
        x: bounds.x.min,
        y: bounds.y.max,
    };
    let south_east = Point2D {
        x: bounds.x.max,
        y: bounds.y.min,
    };
    let (min_x, min_y, _) = lat_lon_deg_to_tile_index(north_west, 0.0);
    let (max_x, max_y, _) = lat_lon_deg_to_tile_index(south_east, 0.0);
    let z0_bounds = Region::new(min_x, min_y, max_x, max_y);

    // Minimum zoom, where bounds fit in one tile.
    let mut min_z = (360.0 / bounds.x.range())
        .min((MAX_LAT_DEG - MIN_LAT_DEG) / (max_lat - min_lat))
        .log2()
        .floor() as u32;
    let z_min_bounds = z0_bounds * 2_f64.powi(min_z as i32);
    if (z_min_bounds.x.min.floor() != z_min_bounds.x.max.floor())
        || (z_min_bounds.y.min.floor() != z_min_bounds.y.max.floor())
    {
        min_z -= 1;
    }

    // Maximum zoom, where tile resolution >= original resolution
    //   TODO, this assumes input projection is aligned to WGS84
    let x_resolution = bounds.x.range() / dimensions.0 as f64;
    let y_resolution = bounds.y.range() / dimensions.1 as f64;
    let z0_x_resolution = 360.0 / tile_dim.0 as f64;
    let z0_y_resolution = (MAX_LAT_DEG - MIN_LAT_DEG) / tile_dim.1 as f64;
    let max_z = (z0_x_resolution / x_resolution)
        .max(z0_y_resolution / y_resolution)
        .log2()
        .ceil() as u32;

    (z0_bounds, (min_z, max_z))
}

pub fn tile_bounds_lat_lon_deg(x: u32, y: u32, z: u32) -> Option<Region<f64>> {
    let nw = tile_index_to_lat_lon_deg(x as f64, y as f64, z as f64)?;
    let se = tile_index_to_lat_lon_deg((x + 1) as f64, (y + 1) as f64, z as f64)?;
    Some(Region::new(nw.x, se.y, se.x, nw.y))
}

pub fn tile_index_to_lat_lon_deg(x: f64, y: f64, z: f64) -> Option<Point2D<f64>> {
    let n = 2.0_f64.powf(z);
    if x < 0.0 || x / n > 1.0 || y < 0.0 || y / n > 1.0 || z < 0.0 {
        return None;
    }
    let lon = x * TAU / n - PI;
    let var = PI * (1.0 - 2.0 * y / n);
    let lat = (0.5 * ((var).exp() - (-var).exp())).atan();
    Some(Point2D {
        x: lon.to_degrees(),
        y: lat.to_degrees(),
    })
}

pub fn lat_lon_deg_to_tile_index(point: Point2D<f64>, z: f64) -> (f64, f64, f64) {
    let n = 2.0_f64.powf(z);
    let lon = point.x.to_radians();
    let lat = point.y.to_radians();
    let x = n * (lon + PI) / TAU;
    let y = n * (1.0 - ((lat.tan() + 1.0 / lat.cos()).ln() / PI)) / 2.0;
    (x, y, z)
}
