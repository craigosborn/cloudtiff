use cloudtiff::{CloudTiff,ByteRangeService,PathReader};
use image::DynamicImage;
use std::time::Instant;
use tokio;
use tracing_subscriber;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/preview_ray.jpg";

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set the maximum log level
        .with_thread_ids(true)
        .init();

    let mut service = ByteRangeService::new(PathReader::new(SAMPLE_COG)).unwrap();
    let mut connection = service.create_connection();
    let _h = tokio::spawn(async move {
        service.serve_tokio().await;
    });

    let mut connection2 = connection.clone();

    let cog = CloudTiff::open_ray(&mut connection).unwrap();

    let t0 = Instant::now();
    let preview = cog.render_image_with_mp_limit_ray(&mut connection2, 10.0).unwrap();
    println!("Got preview in {:.3}ms", t0.elapsed().as_micros() as f64 / 1000.0);

    let img: DynamicImage = preview.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
}