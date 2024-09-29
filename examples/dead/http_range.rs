use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::header::RANGE;
use reqwest::Client;
use tokio;
use std::time::Instant;

const URL: &str = "https://sentinel-cogs.s3.amazonaws.com/sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a reqwest client
    let client = Client::new();
    // Create a stream of futures
    let mut requests = FuturesUnordered::new();
    // let mut responses = vec![];

    // Send all requests concurrently
    let t_req = Instant::now();
    let n = 350_000;
    for i in 0..10_u64 {
        let start = i * n;
        let end = i * n - 1;
        let client = client.clone();
        let range_header_value = format!("bytes={start}-{end}");
        println!("Request {i}");
        requests.push(async move {
        // responses.push(
            client
                .get(URL)
                .header(RANGE, range_header_value)
                .send()
                .await
        // )
        });
    }
    println!("Requests in {}ms", t_req.elapsed().as_secs_f32() * 1e3);

    // Process responses as they come in
    let t_res = Instant::now();
    while let Some(response) = requests.next().await {
    // for response in responses {
        match response {
            Ok(resp) => {
                println!("Received response: {:?}", resp.status());
                // let text = resp.text().await?;
                // println!("Response body: {}", text);
            }
            Err(e) => {
                eprintln!("Request failed: {}", e);
            }
        }
    }
    println!("Responses in {}ms", t_res.elapsed().as_secs_f32() * 1e3);

    Ok(())
}


// Request 0
// Request 1
// Request 2
// Request 3
// Request 4
// Request 5
// Request 6
// Request 7
// Request 8
// Request 9
// Requests in 8910.101ms
// Received response: 200
// Received response: 200
// Received response: 200
// Received response: 200
// Received response: 200
// Received response: 200
// Received response: 200
// Received response: 200
// Received response: 200
// Received response: 200
// Responses in 0.13204199ms