#![cfg(feature = "http")]

use super::AsyncReadRange;
use futures::future::BoxFuture;
use futures::FutureExt;
use reqwest::header::RANGE;
use reqwest::{Client, IntoUrl, Url};
use std::fmt;
use std::future::Future;
use std::io::{Error, ErrorKind, Result};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncRead;

pub struct HttpReader {
    url: Url,
    position: u64,
    _read_request: Option<PendingRequest>,
}

type PendingRequest = Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Sync + Send>>;

impl fmt::Debug for HttpReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpReader")
            .field("url", &self.url)
            .field("position", &self.position)
            .finish()
    }
}

impl HttpReader {
    pub fn new<U: IntoUrl>(url: U) -> Result<Self> {
        Ok(Self {
            url: url
                .into_url()
                .map_err(|e| Error::new(ErrorKind::AddrNotAvailable, format!("{e:?}")))?,
            position: 0,
            _read_request: None,
        })
    }

    pub fn get_or_create_read_request(&mut self, n: usize) -> PendingRequest {
        match self._read_request.take() {
            Some(req) => req,
            None => {
                let start = self.position;
                let end = start + n as u64 - 1; // GOTCHA byte range is inclusive
                let fut = Client::new()
                    .get(self.url.clone())
                    .header(RANGE, format!("bytes={start}-{end}"))
                    .send();
                Box::pin(fut.then(|result| async move {
                    match result {
                        Ok(response) => match response.bytes().await {
                            Ok(bytes) => Ok(bytes.to_vec()),
                            Err(e) => Err(Error::new(ErrorKind::InvalidData, format!("{e:?}"))),
                        },
                        Err(e) => Err(Error::new(ErrorKind::NotConnected, format!("{e:?}"))),
                    }
                }))
            }
        }
    }
}

impl AsyncReadRange for HttpReader {
    fn read_range_async<'a>(&'a self, start: u64, buf: &'a mut [u8]) -> BoxFuture<Result<usize>> {
        let end = start + buf.len() as u64 - 1; // GOTCHA byte range is inclusive
        let request_builder = Client::new()
            .get(self.url.clone())
            .header(RANGE, format!("bytes={start}-{end}"));

        Box::pin(async move {
            let request = request_builder.send();
            let response = request
                .await
                .map_err(|e| Error::new(ErrorKind::NotConnected, format!("{e:?}")))?;

            let bytes = response
                .bytes()
                .await
                .map_err(|e| Error::new(ErrorKind::InvalidData, format!("{e:?}")))?;
            let n = bytes.len();
            buf[..n].copy_from_slice(&bytes[..]);
            Ok(bytes.len())
        })
    }
}

impl AsyncRead for HttpReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let mut fut = self.get_or_create_read_request(buf.remaining());

        match fut.poll_unpin(cx) {
            Poll::Pending => {
                self._read_request = Some(fut);
                Poll::Pending
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Ready(Ok(bytes)) => {
                let n = bytes.len().max(buf.remaining());
                let target = buf.initialize_unfilled_to(n);
                target.copy_from_slice(&bytes[..]);
                buf.advance(n);
                self.position += n as u64;
                Poll::Ready(Ok(()))
            }
        }
    }
}
