use futures::future::BoxFuture;
use std::fmt::Debug;
use std::io::{Read, Result, Seek};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncRead, AsyncSeek};
use tokio::sync::Mutex as AsyncMutex;

// pub mod generic; // TODO
pub mod http;
pub mod path;
pub mod s3;
pub mod fs;

// pub trait Safe: Debug + Send + Sync + 'static {} // TODO

pub trait ReadSeek: Read + Seek + Debug + Send + Sync + 'static {}

pub trait AsyncReadSeek: AsyncRead + AsyncSeek + Debug + Send + Sync + 'static + Unpin {}

pub trait ReadRange: Debug + Send + Sync + 'static {
    fn read_range(&self, start: u64, end: u64) -> Result<Vec<u8>>;
}

pub trait AsyncReadRange: Debug + Send + Sync + 'static {
    fn read_range_async(&self, start: u64, end: u64) -> BoxFuture<'static, Result<Vec<u8>>>;
}

#[derive(Debug)]
pub enum ReaderFlavor {
    ReadRange(Arc<dyn ReadRange>),
    ReadSeek(Arc<Mutex<dyn ReadSeek>>),
}

#[derive(Debug, Clone)]
pub enum AsyncReaderFlavor {
    AsyncReadRange(Arc<dyn AsyncReadRange>),
    AsyncReadSeek(Arc<AsyncMutex<dyn AsyncReadSeek>>),
}

// impl AsyncReadRange for AsyncReaderFlavor {
//     fn read_range_async(&self, start: u64, end: u64) -> BoxFuture<'static, Result<Vec<u8>>> {
//         match self {
//             AsyncReaderFlavor::AsyncReadRange(reader) => reader.read_range_async(start, end),
//             AsyncReaderFlavor::AsyncReadSeek(reader) => {
//                 async move {
//                     let locked_reader = reader.lock().await;
//                     match locked_reader.seek(SeekFrom::Start(start)).await {
//                         Ok(_) => {
//                             let mut buffer = vec![0; (end - start) as usize];
//                             locked_reader.read(&mut buffer).await;
//                             Ok(buffer)
//                         }
//                         Err(e) => Err(e),
//                     }
//                 }.boxed()
//             }
//         }
//     }
// }
