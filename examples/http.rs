use cloudtiff::{CloudTiff, HttpReader};
use image::DynamicImage;
use std::time::Instant;
use tokio;
use tracing_subscriber;

const URL: &str = "https://sentinel-cogs.s3.amazonaws.com/sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";
const OUTPUT_FILE: &str = "data/http.tif";

#[cfg(not(all(feature = "http", feature = "async")))]
compile_error!("This example requires the ['http', 'async'] features to be enabled.");

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Example: cloudtiff + Async HTTP");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_thread_ids(true)
        .init();

    handler().await;
}

async fn handler() {
    // COG
    let t_cog = Instant::now();

    // let mut reader = HttpReader(source);
    // let header_bytes = source.read_range_async(0, 16_384).await.unwrap();
    // let mut header_reader = Cursor::new(header_bytes);
    let mut reader = HttpReader::new(URL).unwrap();
    let cog = CloudTiff::open_async(&mut reader).await.unwrap();
    println!("Indexed COG in {}ms", t_cog.elapsed().as_millis());

    // Preview
    let t_preview = Instant::now();
    let preview = tokio::task::spawn_blocking(||async move {cog
        .renderer()
        .with_mp_limit(1.0)
        .with_async_range_reader(reader)
        .render_async()
        .await
        .unwrap();
    });
    
    // println!(
    //     "Got preview in {:.6} seconds",
    //     t_preview.elapsed().as_secs_f64()
    // );
    // println!("{}", preview);

    // // Image
    // let img: DynamicImage = preview.try_into().unwrap();
    // img.save(OUTPUT_FILE).unwrap();
    // println!("Image saved to {OUTPUT_FILE}");
    // cog.clone()
}
