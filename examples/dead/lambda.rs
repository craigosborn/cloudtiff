use aws_sdk_s3::{primitives::ByteStream, Client};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, info, warn};
use tracing_subscriber;

#[derive(Deserialize)]
struct Request {
    source: String,
    destination: String,
}

#[derive(Serialize)]
struct Response {
    destination: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .init();

    let shared_config = aws_config::load_defaults(aws_config::BehaviorVersion::v2024_03_28()).await;
    let client = Client::new(&shared_config);
    let shared_client = &client;
    lambda_runtime::run(service_fn(move |event: LambdaEvent<Request>| async move {
        handler(&shared_client, event).await
    }))
    .await
}

async fn handler(client: &Client, event: LambdaEvent<Request>) -> Result<Response, Error> {
    info!("Starting handler.");
    let t0 = Instant::now();

    let (source_bucket, source_prefix) = event
        .payload
        .source
        .split_once("/")
        .unwrap_or((&event.payload.source, ""));
    let Some((destination_bucket, destination_key)) = event.payload.destination.split_once("/")
    else {
        return Err("Bad destination".into());
    };

    let objects = client
        .list_objects_v2()
        .bucket(source_bucket)
        .prefix(source_prefix)
        .send()
        .await?;

    debug!("{} objects in bucket.", objects.contents().len());

    let downloads = objects
        .contents()
        .into_iter()
        .filter_map(|object| match object.key() {
            Some(key) => {
                let client_clone = client.clone();
                let bucket_string = source_bucket.to_string();
                let key_string = key.to_string();
                Some(async move { save_object(&client_clone, bucket_string, key_string).await })
            }
            None => {
                warn!("Failed To Get Object Key {:?}", object);
                None
            }
        })
        .collect();
    let files: Vec<PathBuf> = thottled_concurrency(downloads, 100)
        .await
        .into_iter()
        .filter_map(|download_result| match download_result {
            Some(Ok(pathbuf)) => Some(pathbuf),
            Some(Err(e)) => {
                warn!("Failed download {:?}", e);
                None
            }
            _ => None,
        })
        .collect();

    info!(
        "Downloaded {} objects in {}ms using tokio.",
        files.len(),
        t0.elapsed().as_millis()
    );
    let t1 = Instant::now();

    match stitch(files.clone()).await {
        Ok(ortho) => {
            if let Ok(tiff_bytes) = ortho.geotiff_bytes() {
                debug!(
                    "Slim worked, uploading {}bytes to {}/{}",
                    tiff_bytes.len(),
                    destination_bucket,
                    destination_key
                );
                let body = ByteStream::from(tiff_bytes);
                let upload_result = client
                    .put_object()
                    .bucket(destination_bucket)
                    .key(destination_key)
                    .body(body)
                    .send()
                    .await;
                debug!("Upload result {:?}", upload_result);
            } else {
                return Err("Failed to convert ortho to geotiff".into());
            }
        }
        Err(e) => return Err(e.into()),
    }

    info!("Created ortho in {}ms.", t1.elapsed().as_millis());
    info!("Handled in {}ms", t0.elapsed().as_millis());

    Ok(Response {
        destination: event.payload.destination,
    })
}
