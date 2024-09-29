use cloudtiff::{ByteRangeService, CloudTiff, HttpReader};
use image::DynamicImage;
use std::time::Instant;
use tokio;
use tracing_subscriber;

const SAMPL_URL: &str = "https://sentinel-cogs.s3.amazonaws.com/sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";
const OUTPUT_FILE: &str = "data/http_ray_preview.tif";

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set the maximum log level
        .with_thread_ids(true)
        .init();

    let reader = HttpReader::new(SAMPL_URL).unwrap();
    let mut service = ByteRangeService::new(reader).unwrap();
    let mut connection = service.create_connection();
    let _h = tokio::task::spawn(async move {
        service.serve_tokio_async().await;
    });

    let mut connection2 = connection.clone();

    let cog = CloudTiff::open_ray(&mut connection).unwrap();

    let t0 = Instant::now();
    let preview = cog
        .render_image_with_mp_limit_par(&mut connection2, 0.4)
        .await
        .unwrap();
    println!(
        "Got preview in {:.3}ms",
        t0.elapsed().as_micros() as f64 / 1000.0
    );

    let img: DynamicImage = preview.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
}
