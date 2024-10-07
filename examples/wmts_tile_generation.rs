#[cfg(not(feature = "image"))]
compile_error!("This example requires the 'image' feature");

use cloudtiff::CloudTiff;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;
use tokio;
use tokio::fs::File as AsyncFile;
use tokio::sync::Mutex;
use tracing_subscriber;

// const SAMPLE_COG: &str = "data/sample.tif";
const SAMPLE_COG: &str = "data/taupo_dem.tif";
const TILE_SIZE: u32 = 512;

#[tokio::main]
async fn main() {
    println!("Example: cloudtiff wmts tile generation");

    // Logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // COG
    println!("Opening `{SAMPLE_COG}`");
    let mut file = File::open(SAMPLE_COG).unwrap();
    let cog = CloudTiff::open(&mut file).unwrap();
    println!("{cog}");

    // Tile
    let t_tiles = Instant::now();
    let thread_safe_file = Arc::new(Mutex::new(AsyncFile::open(SAMPLE_COG).await.unwrap()));
    let n_tiles = cog
        .renderer()
        .with_async_reader(thread_safe_file)
        .render_wmts_tile_tree_async((TILE_SIZE, TILE_SIZE), "./data/tiles/tile_{x}_{y}_{z}.png")
        .await
        .unwrap();
    let dt = t_tiles.elapsed().as_secs_f64();
    println!(
        "Rendered {n_tiles} tiles in {:.3}ms ({:.1}fps)",
        dt * 1e3,
        n_tiles as f64 / dt
    );
}
