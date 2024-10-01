use aws_config::{self, Region};
use aws_sdk_s3::{config::Config, Client};
use cloudtiff::{CloudTiff, S3Reader};
use image::DynamicImage;
use std::{io::Cursor, time::Instant};
use tokio;
use tracing_subscriber;

// https://docs.rs/object_store/0.11.0/object_store/
// https://crates.io/crates/aws-sdk-s3

#[cfg(not(all(feature = "s3", feature = "async")))]
compile_error!("This example requires the ['http', 'async'] features to be enabled.");

const BUCKET_NAME: &str = "sentinel-cogs";
const OBJECT_NAME: &str = "sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";
const OUTPUT_FILE: &str = "data/s3.tif";

#[tokio::main]
async fn main() {
    println!("Example: cloudtiff + Async HTTP");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_thread_ids(true)
        .init();

    // Thread-safe Async S3 Byte Range Reader
    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let config = Config::new(&sdk_config)
        .to_builder()
        .region(Some(Region::from_static("us-west-2")))
        .build();
    let client = Client::from_conf(config);
    let reader = S3Reader::new(client, BUCKET_NAME, OBJECT_NAME);

    handler(reader.clone()).await;
}

async fn handler(source: S3Reader) {
    // COG
    let t_cog = Instant::now();

    let header_bytes = source.read_range_async(0, 16_384).await.unwrap();
    let mut header_reader = Cursor::new(header_bytes);
    let cog = CloudTiff::open(&mut header_reader).unwrap();
    println!("Indexed COG in {}ms", t_cog.elapsed().as_millis());
    println!("{cog}");

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
}
