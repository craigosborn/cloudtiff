use futures::future;
// use std::fs::File;
// use std::io::{self, Seek, SeekFrom};
use rayon;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::io::{self, SeekFrom};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};
use tokio;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tracing_subscriber;

const SAMPLE_COG: &str = "data/sample.tif";

type Connection = Sender<ByteRangeRequest>;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set the maximum log level
        .with_thread_ids(true)
        .init();

    let mut service = ByteRangeService::new(SAMPLE_COG).await.unwrap();

    let n_con = 1000;
    let connections_a: Vec<Connection> = (0..n_con)
        .into_iter()
        .map(|_i| service.create_connection())
        .collect();
    let connections_b: Vec<Connection> = (0..n_con)
        .into_iter()
        .map(|_i| service.create_connection())
        .collect();

    let h = tokio::spawn(async move {
        service.serve().await;
    });

    let n = 10000;

    let t_tokio = Instant::now();
    let task_sum = process_tokio(connections_a, n).await;
    println!("Tokio: {task_sum} in {:.3}ms",t_tokio.elapsed().as_secs_f64() * 1e3);

    let t_rayon = Instant::now();
    let task_sum = process_rayon(connections_b, n);
    println!("Rayon: {task_sum} in {:.3}ms",t_rayon.elapsed().as_secs_f64() * 1e3);

    // let _ = h.await;
}

async fn process_tokio(connections: Vec<Connection>, n: u64) -> u64 {
    let tasks = connections
        .into_iter()
        .enumerate()
        .map(|(i, con)| {
            let start = 1_000_000 + (i as u64) * n;
            let end = 1_000_000 + ((i + 1) as u64) * n;
            (i, con, start, end)
        })
        .map(|(i, con, start, end)| {
            tokio::spawn(async move {
                let (tx, rx) = mpsc::channel();
                con.send(ByteRangeRequest {
                    start,
                    end,
                    response: tx,
                })
                .unwrap();
                let byte_range_response = rx.recv();
                println!("Tokio task {i} received");
                match byte_range_response {
                    Ok(Ok(v)) => v[0] as u64,
                    _ => 0,
                }
            })
        });
    future::join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap_or(0))
        .sum()
}

fn process_rayon(connections: Vec<Connection>, n: u64) -> u64 {
    connections
        .into_iter()
        .enumerate()
        .map(|(i, con)| {
            let start = 2_000_000 + (i as u64) * n;
            let end = 2_000_000 + ((i + 1) as u64) * n;
            (i, con, start, end)
        })
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(i, con, start, end)| {
            let (tx, rx) = mpsc::channel();
            con.send(ByteRangeRequest {
                start: *start,
                end: *end,
                response: tx,
            })
            .unwrap();
            let byte_range_response = rx.recv();
            println!("Rayon task {i} received");
            match byte_range_response {
                Ok(Ok(v)) => v[0] as u64,
                _ => 0,
            }
        })
        .sum()
}

struct ByteRangeService {
    source: File,
    connections: Vec<Receiver<ByteRangeRequest>>,
}

#[derive(Debug)]
pub struct ByteRangeRequest {
    pub start: u64,
    pub end: u64,
    pub response: Sender<io::Result<Vec<u8>>>,
}

#[derive(Debug)]
pub struct ByteRangeReader {
    pub start: u64,
    pub end: u64,
    pub response: Sender<io::Result<Vec<u8>>>,
}

impl ByteRangeService {
    pub async fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let source = File::open(path).await?;
        Ok(Self {
            source,
            connections: vec![],
        })
    }

    pub fn create_connection(&mut self) -> Connection {
        let (tx, rx) = mpsc::channel();
        self.connections.push(rx);
        tx
    }

    pub async fn serve(&mut self) {
        loop {
            let mut all_closed = true;

            for receiver_id in 0..self.connections.len() {
                match self.connections[receiver_id].try_recv() {
                    Ok(msg) => {
                        // If a message is received, print it
                        println!("Service received request: {:?}", msg);

                        let byte_range_response = self.read_byte_range(msg.start, msg.end).await;
                        match msg.response.send(byte_range_response) {
                            Ok(_) => (),
                            Err(e) => println!("Service Send Error: {e:?}"),
                        }
                        all_closed = false;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        // No message available in this receiver
                        all_closed = false;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        // This receiver has been disconnected
                        // We don't set `all_closed = false` here so it can close when all receivers are disconnected
                    }
                }
            }

            // If all receivers are closed and no messages are left, break the loop
            if all_closed {
                println!("All receivers are closed.");
                break;
            }

            // Sleep for a bit to prevent busy waiting
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
    }

    async fn read_byte_range(&mut self, start: u64, end: u64) -> io::Result<Vec<u8>> {
        self.source.seek(SeekFrom::Start(start)).await?;
        let mut buffer = vec![0; (end - start) as usize];
        self.source.read_exact(&mut buffer).await?;
        Ok(buffer)
    }
}
