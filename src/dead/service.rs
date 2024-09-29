use crate::reader::{ReadRange, ReadRangeAsync};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::io;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use tracing::*;

#[derive(Clone)]
pub struct Connection(Sender<ByteRangeRequest>);

impl ReadRange for Connection {
    fn read_range(&mut self, start: u64, end: u64) -> Result<Vec<u8>, io::Error> {
        let (tx, rx) = mpsc::channel();
        self.0
            .send(ByteRangeRequest {
                start: start,
                end: end,
                response: ByteRangeCallback::Sync(tx),
            })
            .unwrap();
        match rx.recv() {
            Ok(read_result) => read_result,
            Err(e) => Err(io::Error::new(io::ErrorKind::BrokenPipe, format!("{e:?}"))),
        }
    }
}

impl ReadRangeAsync for Connection {
    async fn read_range(&mut self, start: u64, end: u64) -> Result<Vec<u8>, io::Error> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        self.0
            .send(ByteRangeRequest {
                start: start,
                end: end,
                response: ByteRangeCallback::Async(tx),
            })
            .unwrap();
        match rx.recv().await {
            Some(read_result) => read_result,
            None => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                format!("ByteRangeService"),
            )),
        }
    }
}

pub struct ByteRangeService<R> {
    source: R,
    connections: Vec<Receiver<ByteRangeRequest>>,
}

#[derive(Debug)]
pub struct ByteRangeRequest {
    pub start: u64,
    pub end: u64,
    pub response: ByteRangeCallback,
}

#[derive(Debug)]
pub enum ByteRangeCallback {
    Sync(std::sync::mpsc::Sender<io::Result<Vec<u8>>>),
    Async(tokio::sync::mpsc::Sender<io::Result<Vec<u8>>>),
}

impl<R> ByteRangeService<R> {
    pub fn new(source: R) -> io::Result<Self> {
        Ok(Self {
            source,
            connections: vec![],
        })
    }

    pub fn create_connection(&mut self) -> Connection {
        let (tx, rx) = mpsc::channel();
        self.connections.push(rx);
        Connection(tx)
    }
}

impl<R: ReadRange> ByteRangeService<R> {
    pub fn serve(self) {
        let mut source = self.source;
        loop {
            let mut all_closed = true;

            for receiver_id in 0..self.connections.len() {
                match self.connections[receiver_id].try_recv() {
                    Ok(msg) => {
                        // If a message is received, print it
                        println!("Service received request: {:?}", msg);

                        let byte_range_response = source.read_range(msg.start, msg.end);
                        match msg.response {
                            ByteRangeCallback::Sync(cb) => match cb.send(byte_range_response) {
                                Ok(_) => (),
                                Err(e) => println!("Service Send Error: {e:?}"),
                            },
                            ByteRangeCallback::Async(_cb) => {
                                println!("async callback not available. Use ByteRangeService.serve_async instead.")
                                // cb.send(byte_range_response);
                            } // TODO poll it?
                        };
                        all_closed = false;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        all_closed = false;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {}
                }
            }

            if all_closed {
                println!("All receivers are closed.");
                break;
            }

            // tokio::time::sleep(Duration::from_micros(100)).await;
        }
    }
}

impl<R: ReadRangeAsync + Clone> ByteRangeService<R> {
    pub async fn serve_tokio_async(self) {
        futures::future::join_all(self.connections
            .into_iter()
            .enumerate()
            .map(|(index, connection)| (connection, self.source.clone(),index))
            .map(|(connection, mut source_clone, id)| {
                tokio::task::spawn(async move {
                    while let Ok(msg) = connection.recv() {
                        debug!("Tokio Msg received {}",msg.start);
                        let byte_range_response = source_clone.read_range(msg.start, msg.end).await;
                        match msg.response {
                            ByteRangeCallback::Sync(cb) => match cb.send(byte_range_response) {
                                Ok(_) => (),
                                Err(e) => println!("Service Send Error: {e:?}"),
                            },
                            ByteRangeCallback::Async(cb) => {
                                match cb.send(byte_range_response).await {
                                    Ok(_) => (),
                                    Err(e) => println!("Service Send Error: {e:?}"),
                                }
                            }
                        }
                        debug!("Tokio Msg responded {}",msg.start);
                    }
                    debug!("Tokio Msg closed {id}");
                })
            })).await;
    }
}

impl<R: ReadRange + Clone> ByteRangeService<R> {
    pub async fn serve_tokio(self) {
        futures::future::join_all(self.connections
            .into_iter()
            .enumerate()
            .map(|(index, connection)| (connection, self.source.clone(),index))
            .map(|(connection, mut source_clone, id)| {
                tokio::task::spawn(async move {
                    while let Ok(msg) = connection.recv() {
                        debug!("Tokio Msg received {}",msg.start);
                        let byte_range_response = source_clone.read_range(msg.start, msg.end);
                        match msg.response {
                            ByteRangeCallback::Sync(cb) => match cb.send(byte_range_response) {
                                Ok(_) => (),
                                Err(e) => println!("Service Send Error: {e:?}"),
                            },
                            ByteRangeCallback::Async(_cb) => {
                                println!("async callback not available. Use ByteRangeService.serve_async instead.")
                            },
                        }
                        debug!("Ray Msg responded {}",msg.start);
                    }
                    debug!("Ray Msg closed {id}");
                })
            })).await;
    }
}

impl<R: ReadRange + Clone> ByteRangeService<R> {
    pub fn serve_ray(self) {
        self.connections
            .into_iter()
            .enumerate()
            .map(|(index, connection)| (connection, self.source.clone(),index))
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(connection, mut source_clone, id)| {
                while let Ok(msg) = connection.recv() {
                    debug!("Ray Msg Received {id}");
                    let byte_range_response = source_clone.read_range(msg.start, msg.end);
                    match msg.response {
                        ByteRangeCallback::Sync(cb) => match cb.send(byte_range_response) {
                            Ok(_) => (),
                            Err(e) => println!("Service Send Error: {e:?}"),
                        },
                        ByteRangeCallback::Async(_cb) => {
                            println!("async callback not available. Use ByteRangeService.serve_async instead.")
                        },
                    }
                    debug!("Ray Msg closed {id}");
                }
            })
            .collect()
    }
}

impl<R: ReadRangeAsync> ByteRangeService<R> {
    pub async fn serve_async(&mut self) {
        loop {
            let mut all_closed = true;
            for receiver_id in 0..self.connections.len() {
                match self.connections[receiver_id].try_recv() {
                    Ok(msg) => {
                        let byte_range_response = self.source.read_range(msg.start, msg.end).await;
                        match msg.response {
                            ByteRangeCallback::Sync(cb) => match cb.send(byte_range_response) {
                                Ok(_) => (),
                                Err(e) => println!("Service Send Error: {e:?}"),
                            },
                            ByteRangeCallback::Async(cb) => {
                                match cb.send(byte_range_response).await {
                                    Ok(_) => (),
                                    Err(e) => println!("Service Send Error: {e:?}"),
                                }
                            }
                        };
                        all_closed = false;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        all_closed = false;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {}
                }
            }

            if all_closed {
                println!("All receivers are closed.");
                break;
            }

            tokio::time::sleep(Duration::from_micros(100)).await;
        }
    }
}
