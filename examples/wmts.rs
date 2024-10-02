#[cfg(not(feature = "image"))]
compile_error!("This example requires the 'image' feature");

use cloudtiff::{CloudTiff, Point2D, Region};
use core::f64;
use image::DynamicImage;
use std::f64::consts::{PI, TAU};
use std::fs::File;
use std::time::Instant;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/wmts.jpg";
const TILE_SIZE: u32 = 512;

fn main() {
    println!("Example: cloudtiff wmts");

    // COG
    println!("Opening `{SAMPLE_COG}`");
    let mut file = File::open(SAMPLE_COG).unwrap();
    let cog = CloudTiff::open(&mut file).unwrap();

    // Bounds
    let tile_region = tile_bounds_lat_lon_deg(1188, 2608, 13).unwrap();
    let cog_bounds = cog.bounds_lat_lon_deg().unwrap();
    println!("Bounds:");
    println!("  Tile: {tile_region:.6?}");
    println!("  COG: {cog_bounds:.6?}");

    // Tile
    let t_tile = Instant::now();
    let (west, south, east, north) = tile_region.as_tuple();
    let tile = cog
        .renderer()
        .with_exact_resolution((TILE_SIZE, TILE_SIZE))
        .of_output_region_lat_lon_deg(west, south, east, north)
        .with_reader(file)
        .render()
        .unwrap();
    println!(
        "Rendered tile in {:.3}ms",
        t_tile.elapsed().as_secs_f64() * 1000.0
    );
    println!("Tile: {tile}");

    // Output
    let img: DynamicImage = tile.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
}

fn tile_bounds_lat_lon_deg(x: usize, y: usize, z: usize) -> Option<Region<f64>> {
    let nw = tile_index_to_lat_lon_deg(x as f64, y as f64, z as f64)?;
    let se = tile_index_to_lat_lon_deg((x + 1) as f64, (y + 1) as f64, z as f64)?;
    Some(Region::new(nw.x, se.y, se.x, nw.y))
}

fn tile_index_to_lat_lon_deg(x: f64, y: f64, z: f64) -> Option<Point2D<f64>> {
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
