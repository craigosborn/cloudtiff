use cloudtiff::{CloudTiff, PathReader};
use core::f64;
use image::DynamicImage;
use std::f64::consts::{PI, TAU};
use std::time::Instant;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/wmts.tif";
const TILE_SIZE: u32 = 512;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    println!("Example: WMTS Reader");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set the maximum log level
        .with_thread_ids(true)
        .init();

    // COG
    println!("Opening `{SAMPLE_COG}`");
    let reader = PathReader::new(SAMPLE_COG);
    let cog = CloudTiff::open_par(reader.clone()).await.unwrap();

    // Bounds
    let bounds = tile_bounds_lat_lon_deg(1188, 2608, 13).unwrap();//(18, 40, 7).unwrap();
    let cog_bounds = cog.bounds_lat_lon_deg().unwrap();
    println!("Bounds:");
    println!("  Tile: {bounds:.6?}");
    println!("  COG: {cog_bounds:.6?}");

    // Tile
    let t_tile = Instant::now();
    let tile = cog
        .render_region_lat_lon_deg_par(reader, bounds, (TILE_SIZE, TILE_SIZE))
        .await
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

fn tile_bounds_lat_lon_deg(x: usize, y: usize, z: usize) -> Option<(f64, f64, f64, f64)> {
    let (north, west) = tile_index_to_lat_lon_deg(x as f64, y as f64, z as f64)?;
    let (south, east) = tile_index_to_lat_lon_deg((x + 1) as f64, (y + 1) as f64, z as f64)?;
    Some((north, west, south, east))
}

fn tile_index_to_lat_lon_deg(x: f64, y: f64, z: f64) -> Option<(f64, f64)> {
    let n = 2.0_f64.powf(z);
    if x < 0.0 || x / n > 1.0 || y < 0.0 || y / n > 1.0 || z < 0.0 {
        return None;
    }
    let lon = x * TAU / n - PI;
    let var = PI * (1.0 - 2.0 * y / n);
    let lat = (0.5 * ((var).exp() - (-var).exp())).atan();
    Some((lat.to_degrees(), lon.to_degrees()))
}
