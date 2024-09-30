use async_http_range_reader::{AsyncHttpRangeReader, CheckSupportMethod};
use cloudtiff::CloudTiff;
use http::HeaderMap;
use image::DynamicImage;
use reqwest;
use std::sync::Arc;
use std::time::Instant;
use tokio;
use tokio::sync::Mutex;

const URL: &str = "https://sentinel-cogs.s3.amazonaws.com/sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";
const OUTPUT_FILE: &str = "data/http_preview.tif";

type HttpReader = Arc<Mutex<AsyncHttpRangeReader>>;

#[tokio::main]
async fn main() {
    println!("Example: cloudtiff + Async HTTP");

    // Connection
    let client = reqwest::Client::new();
    let url = URL.parse().unwrap();
    let (range_reader, _header) =
        AsyncHttpRangeReader::new(client, url, CheckSupportMethod::Head, HeaderMap::default())
            .await
            .unwrap();

    // Thread-safe Async HTTP Byte Range Reader
    let reader = Arc::new(Mutex::new(range_reader));
    handler(reader.clone()).await;
}

async fn handler(reader: HttpReader) {
    // COG
    let t_cog = Instant::now();
    let cog = CloudTiff::open_async(reader.clone()).await.unwrap();
    println!("Indexed COG in {}ms", t_cog.elapsed().as_millis());
    println!("{cog}");

    // Preview
    let t_preview = Instant::now();
    let preview = cog
        .render_image_with_mp_limit_async(reader.clone(), 0.4)
        .await
        .unwrap();
    println!(
        "Got preview in {:.6} seconds",
        t_preview.elapsed().as_secs_f64()
    );
    println!("{}", preview);

    // Metrics
    let ranges = reader.lock().await.requested_ranges().await;
    let requests = ranges.len();
    let byte_count: u64 = ranges.iter().map(|r| r.end - r.start).sum();
    println!("Made {requests} requests and downloaded {byte_count} bytes.");

    // Image
    let img: DynamicImage = preview.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
}
