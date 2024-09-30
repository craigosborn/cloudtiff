use cloudtiff::{CloudTiff, AsyncReadRange, HttpReader};
use image::DynamicImage;
use std::{io::Cursor, time::Instant};
use tokio;
use tracing_subscriber;

const URL: &str = "https://sentinel-cogs.s3.amazonaws.com/sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";
const OUTPUT_FILE: &str = "data/http.tif";

#[cfg(not(all(feature = "http", feature = "async")))]
compile_error!("This example requires the ['http', 'async'] features to be enabled.");

#[tokio::main]
async fn main() {
    println!("Example: cloudtiff + Async HTTP");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set the maximum log level
        .with_thread_ids(true)
        .init();

    // Thread-safe Async HTTP Byte Range Reader
    let reader = HttpReader::new(URL).unwrap();

    handler(reader.clone()).await;
}

async fn handler(source: HttpReader) -> CloudTiff {
    // COG
    let t_cog = Instant::now();

    let header_bytes = source.read_range_async(0, 16_384).await.unwrap();
    let mut header_reader = Cursor::new(header_bytes);
    let cog = CloudTiff::open(&mut header_reader).unwrap();
    println!("Indexed COG in {}ms", t_cog.elapsed().as_millis());
    println!("{cog}");

    // Preview
    let t_preview = Instant::now();
    let preview = cog
        .renderer()
        .with_mp_limit(10.0)
        .with_async_range_reader(source)
        .render_async()
        .await
        .unwrap();
    
    println!(
        "Got preview in {:.6} seconds",
        t_preview.elapsed().as_secs_f64()
    );
    println!("{}", preview);

    // Image
    let img: DynamicImage = preview.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
    cog.clone()
}
