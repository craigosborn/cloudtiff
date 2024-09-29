use futures::stream::{self, StreamExt};
use std::io::SeekFrom;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::Mutex;
// use tracing::*;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG) // Set the maximum log level
    //     .with_thread_ids(true)
    //     .init();

    let file = File::open("data/sample.tif").await.unwrap();
    let am_file = Arc::new(Mutex::new(file));

    let results: Vec<_> = stream::iter(0..3)
        .map(|id| do_task(am_file.clone(), id))
        .buffer_unordered(5)
        .collect()
        .await;

    println!("results: {results:?}");
}

async fn do_task(file: Arc<Mutex<File>>, id: usize) -> f64 {
    let bytes = read_file(file, id).await;

    tokio::task::spawn_blocking(move || { process_file(bytes, id) }).await.unwrap()
}

async fn read_file(file: Arc<Mutex<File>>, _id: usize) -> Vec<u8> {
    // debug!("{id}: Awaiting lock");
    // let t_lock = Instant::now();
    let mut locked_file = file.lock().await;

    // debug!(
    //     "{id}: After {}ms got lock",
    //     t_lock.elapsed().as_micros() as f64 / 1e3
    // );

    // let t_drop = Instant::now();

    locked_file.seek(SeekFrom::Start(0)).await.unwrap();

    // debug!("{id}: Seeking");
    let mut bytes = vec![0; 100];
    locked_file.read_exact(&mut bytes).await.unwrap();
    // debug!("{id}: Got bytes");

    std::mem::drop(locked_file);
    // debug!(
    //     "{id}: After {}ms dropped lock",
    //     t_drop.elapsed().as_micros() as f64 / 1e3
    // );

    bytes
}

fn process_file(bytes: Vec<u8>, _id: usize) -> f64 {
    // debug!("{id}: processing bytes");
    let t_processing = Instant::now();

    // Processing stand-in
    while Instant::now() - t_processing < Duration::from_millis(100) {
        // Busy-wait loop: Do nothing but consume CPU cycles
    }

    // debug!(
    //     "{id}: processing took {}ms",
    //     t_processing.elapsed().as_micros() as f64 / 1e3
    // );

    bytes.iter().map(|v| *v as f64).sum()
}
