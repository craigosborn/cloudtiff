use cloudtiff::{AsyncReadRange, HttpReader};
use std::time::Instant;
use tokio::io::AsyncReadExt;

const URL: &str = "http://sentinel-cogs.s3.amazonaws.com/sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";

#[tokio::main]
async fn main() {
    let mut reader = HttpReader::new(URL).unwrap();
    let mut buf = vec![0; 10];

    // A
    let t0 = Instant::now();
    reader.read_range_async(0, &mut buf).await.unwrap();
    println!(
        "AsyncReadRange in {:.3}ms: 0x{:02X?}",
        t0.elapsed().as_secs_f32() * 1e3,
        buf
    );

    // B
    let t0 = Instant::now();
    reader.read(&mut buf).await.unwrap();
    println!(
        "AsyncRead      in {:.3}ms: 0x{:02X?}",
        t0.elapsed().as_secs_f32() * 1e3,
        buf
    );

    // A
    let t0 = Instant::now();
    reader.read_range_async(10, &mut buf).await.unwrap();
    println!(
        "AsyncReadRange in {:.3}ms: 0x{:02X?}",
        t0.elapsed().as_secs_f32() * 1e3,
        buf
    );

    // B
    let t0 = Instant::now();
    reader.read(&mut buf).await.unwrap();
    println!(
        "AsyncRead      in {:4.3}ms: 0x{:02X?}",
        t0.elapsed().as_secs_f32() * 1e3,
        buf
    );
}
