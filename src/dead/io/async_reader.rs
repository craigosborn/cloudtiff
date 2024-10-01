#![cfg(feature = "async")]

use super::{AsyncReadRange, AsyncReadSeek};
use core::task::{Context, Poll};
use futures::future::BoxFuture;
use futures::FutureExt;
use std::fmt::Debug;
use std::future::Future;
use std::io::{Result, SeekFrom};
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt, ReadBuf};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum AsyncReader {
    AsyncReadRange(Arc<(Box<dyn AsyncReadRange>, Mutex<u64>)>),
    AsyncReadSeek(Arc<Mutex<dyn AsyncReadSeek>>),
}

impl AsyncReader {
    pub fn from_reader<R: AsyncReadSeek + 'static>(reader: R) -> Self {
        Self::AsyncReadSeek(Arc::new(Mutex::new(reader)))
    }

    pub fn from_range_reader<R: AsyncReadRange + 'static>(reader: R) -> Self {
        Self::AsyncReadRange(Arc::new((Box::new(reader), Mutex::new(0))))
    }
}

impl AsyncReadRange for AsyncReader {
    fn read_range_async(&self, start: u64, buf: &mut [u8]) -> BoxFuture<'static, Result<Vec<u8>>> {
        let self_clone = self.clone();
        async move {
            match self_clone {
                Self::AsyncReadRange(reader) => reader.0.read_range_async(start, buf).await,
                Self::AsyncReadSeek(reader) => {
                    let mut locked_reader = reader.lock().await;
                    match locked_reader.seek(SeekFrom::Start(start)).await {
                        Ok(_) => locked_reader.read(buf).await
                        Err(e) => Err(e),
                    }
                }
            }
        }
        .boxed()
    }
}

impl AsyncRead for AsyncReader {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        let buf_len = buf.capacity();
        let mut fut: Pin<Box<dyn Future<Output = Result<()>>>> = match self.get_mut() {
            Self::AsyncReadRange(range_reader) => Box::pin(async move {
                let position = range_reader.1.lock().await;
                let end = *position + buf_len as u64;
                let buf_filled = buf.filled_mut();
                // println!("reading {}-{}", *position, end);
                match range_reader.0.read_range_async(*position, end).await {
                    Ok(bytes) => {
                        buf_filled.copy_from_slice(&bytes);
                        buf.advance(bytes.len());
                        println!("got bytes {}", bytes.len());
                        Ok(())
                    }
                    Err(e) => {
                        println!("error {e:?}");
                        Err(e)
                    }
                }
            }),
            Self::AsyncReadSeek(reader) => {
                Box::pin(async move {
                    let buf_filled = buf.filled_mut();
                    let mut locked_reader = reader.lock().await; // Lock the reader
                    match locked_reader.read(buf_filled).await {
                        Ok(bytes_read) => {
                            buf.advance(bytes_read);
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                })
            }
        };

        fut.poll_unpin(cx)
    }
}

// impl AsyncSeek for AsyncReader {
//     fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
//         match self {
//             Self::ReadRange((_reader, position)) => match pos {
//                 SeekFrom::Start(offset) => {
//                     *position = offset;
//                     Ok(*position)
//                 }
//                 SeekFrom::Current(offset) => {
//                     *position = position
//                         .checked_add(offset as u64)
//                         .ok_or(Error::new(ErrorKind::InvalidInput, "Seek overflow"))?;
//                     Ok(*position)
//                 }
//                 SeekFrom::End(_offset) => Err(Error::new(
//                     ErrorKind::Unsupported,
//                     "Seek from end not supported",
//                 )),
//             },
//             Self::ReadSeek(reader) => match reader.lock() {
//                 Ok(mut locked_reader) => locked_reader.seek(pos),
//                 Err(e) => Err(Error::new(ErrorKind::Other, format!("{e:?}"))),
//             },
//         }
//     }
// }
