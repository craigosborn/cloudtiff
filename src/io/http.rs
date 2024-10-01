use super::AsyncReadRange;
use core::task::{Context, Poll};
use futures::future::BoxFuture;
use futures::FutureExt;
use reqwest::header::RANGE;
use reqwest::{Client, IntoUrl, Url};
use std::fmt::Debug;
use std::future::Future;
use std::io::{Error, ErrorKind, Result};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::ReadBuf;
use tokio::io::{AsyncRead, AsyncSeek};
use tokio::sync::Mutex as AsyncMutex;

#[derive(Clone, Debug)]
pub struct HttpReader {
    url: Url,
    position: Arc<AsyncMutex<u64>>,
}

impl HttpReader {
    pub fn new<U: IntoUrl>(url: U) -> Result<Self> {
        Ok(Self {
            url: url
                .into_url()
                .map_err(|e| Error::new(ErrorKind::AddrNotAvailable, format!("{e:?}")))?,
            position: Arc::new(AsyncMutex::new(0)),
        })
    }
}

impl AsyncReadRange for HttpReader {
    fn read_range_async(&self, start: u64, buf: &mut [u8]) -> BoxFuture<'static, Result<usize>> {
        let end = start + buf.len() as u64;
        println!("buf: {}", buf.len());
        let request_builder = Client::new()
            .get(self.url.clone())
            .header(RANGE, format!("bytes={start}-{end}"))
            .timeout(Duration::from_millis(1000));

        async move {
            println!("requesting: {}", format!("bytes={start}-{end}"));
            let request = request_builder.send();
            println!("Sent request");
            let response = request
                .await
                .map_err(|e| Error::new(ErrorKind::NotConnected, format!("{e:?}")))?;
            println!("Got response");

            let bytes = response
                .bytes()
                .await
                .map_err(|e| Error::new(ErrorKind::InvalidData, format!("{e:?}")))?;
            // buf.copy_from_slice(&bytes[..]);
            Ok(bytes.len())
        }
        .boxed()
    }
}

// fn read_range_async(&self, start: u64, end: u64, buf: &mut [u8]) -> BoxFuture<'static, Result<Vec<u8>>> {
//     let url = self.0.clone();
//     println!("http rra url: {url}");
//     async move {
//         let request = Client::new()
//             .get(url)
//             .header(RANGE, format!("bytes={start}-{end}"))
//             .send();
//         println!("http rra request range {}", format!("bytes={start}-{end}"));

//         let response = request
//             .await
//             .map_err(|e| Error::new(ErrorKind::NotConnected, format!("{e:?}")))?;

//         println!("http rra response: {response:?}");
//         let bytes = response
//             .bytes()
//             .await
//             .map_err(|e| Error::new(ErrorKind::InvalidData, format!("{e:?}")))?;
//         Ok(bytes.to_vec())
//     }
//     .boxed()
// }
// }

impl AsyncRead for HttpReader {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        let mut fut: Pin<Box<dyn Future<Output = Result<()>>>> = async move {
            let position = self.position.lock().await;
            let buf_filled = buf.initialize_unfilled();
            println!("BUF_FILLED: {}", buf_filled.len());
            match self.read_range_async(*position, buf_filled).await {
                Ok(bytes_read) => {
                    buf.advance(bytes_read);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        .boxed();

        fut.poll_unpin(cx)
    }
}

impl AsyncSeek for HttpReader {
    fn start_seek(
        self: std::pin::Pin<&mut Self>,
        _position: std::io::SeekFrom,
    ) -> std::io::Result<()> {
        todo!()
    }

    fn poll_complete(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<u64>> {
        todo!()
    }

    // fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
    //     match self {
    //         Self::ReadRange((_reader, position)) => match pos {
    //             SeekFrom::Start(offset) => {
    //                 *position = offset;
    //                 Ok(*position)
    //             }
    //             SeekFrom::Current(offset) => {
    //                 *position = position
    //                     .checked_add(offset as u64)
    //                     .ok_or(Error::new(ErrorKind::InvalidInput, "Seek overflow"))?;
    //                 Ok(*position)
    //             }
    //             SeekFrom::End(_offset) => Err(Error::new(
    //                 ErrorKind::Unsupported,
    //                 "Seek from end not supported",
    //             )),
    //         },
    //         Self::ReadSeek(reader) => match reader.lock() {
    //             Ok(mut locked_reader) => locked_reader.seek(pos),
    //             Err(e) => Err(Error::new(ErrorKind::Other, format!("{e:?}"))),
    //         },
    //     }
    // }
}
