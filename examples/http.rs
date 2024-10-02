#[cfg(not(feature = "http"))]
compile_error!("This example requires the 'http' feature");

use cloudtiff::{CloudTiff, HttpReader};
use image::DynamicImage;
use std::time::Instant;
use tokio;

const URL: &str = "https://sentinel-cogs.s3.amazonaws.com/sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";
const OUTPUT_FILE: &str = "data/http.jpg";
const PREVIEW_MEGAPIXELS: f64 = 1.0;


#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Example: cloudtiff + Async HTTP");

    handler().await;
}

async fn handler() {
    // COG
    let t_cog = Instant::now();
    let mut http_reader = HttpReader::new(URL).unwrap();
    let cog = CloudTiff::open_from_async_range_reader(&mut http_reader)
        .await
        .unwrap();
    println!("Indexed COG in {}ms", t_cog.elapsed().as_millis());
    println!("{cog}");

    // Preview
    let t_preview = Instant::now();
    let preview = cog
        .renderer()
        .with_mp_limit(PREVIEW_MEGAPIXELS)
        .with_async_range_reader(http_reader)
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
