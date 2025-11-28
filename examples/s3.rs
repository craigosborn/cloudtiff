#![cfg(feature = "s3")]

use aws_config::{self, Region};
use aws_sdk_s3::{config::Config, Client};
use cloudtiff::{CloudTiff, S3Reader};
use image::DynamicImage;
use std::io::{self, Write};
use std::time::Instant;

// https://docs.rs/object_store/0.11.0/object_store/
// https://crates.io/crates/aws-sdk-s3

const BUCKET_NAME: &str = "sentinel-cogs";
const OBJECT_NAME: &str = "sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";
const OUTPUT_FILE: &str = "data/s3.jpg";
const PREVIEW_MEGAPIXELS: f64 = 1.0;

#[tokio::main]
async fn main() {
    println!("Example: cloudtiff async s3");

    // Ask to use AWS credentials
    let consent: &str = "ok";
    print!(
        r#"This example will use your default AWS environmental credentials to make a request. Type "{consent}" to continue: "#
    );
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() != consent {
        println!("Exiting.");
        return;
    }

    // Configure S3 Reader
    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let config = Config::new(&sdk_config)
        .to_builder()
        .region(Some(Region::from_static("us-west-2")))
        .build();
    let client = Client::from_conf(config);
    let reader = S3Reader::new(client, BUCKET_NAME, OBJECT_NAME);

    // Use S3 Reader to read a cloud tiff
    handler(reader).await;
}

async fn handler(source: S3Reader) {
    // COG
    let t_cog = Instant::now();
    let cog = CloudTiff::open_from_async_range_reader(&source)
        .await
        .unwrap();
    println!("Indexed COG in {}ms", t_cog.elapsed().as_millis());
    println!("{cog}");

    let t_preview = Instant::now();
    let preview = cog
        .renderer()
        .with_mp_limit(PREVIEW_MEGAPIXELS)
        .with_async_reader(source)
        .render()
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
