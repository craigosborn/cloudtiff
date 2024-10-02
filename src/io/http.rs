#![cfg(feature = "http")]

// TODO impl tokio::io::AsyncRead

use super::AsyncReadRange;
use futures::future::BoxFuture;
use reqwest::header::RANGE;
use reqwest::{Client, IntoUrl, Url};
use std::fmt;
use std::io::{Error, ErrorKind, Result};

pub struct HttpReader {
    url: Url,
    position: u64,
}

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
        })
    }
}

impl AsyncReadRange for HttpReader {
    fn read_range_async<'a>(&'a self, start: u64, buf: &'a mut [u8]) -> BoxFuture<Result<usize>> {
        let end = start + buf.len() as u64 - 1; // GOTCHA byte range is inclusive
        let request_builder = Client::new()
            .get(self.url.clone())
            .header(RANGE, format!("bytes={start}-{end}"));
        // .timeout(Duration::from_millis(1000));

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

    // fn read_range_to_vec_async<'a>(
    //     &'a self,
    //     start: u64,
    //     end: u64,
    // ) -> BoxFuture<'a, Result<Vec<u8>>> {
    //     let request_builder = Client::new()
    //         .get(self.url.clone())
    //         .header(RANGE, format!("bytes={start}-{end}"));

    //     Box::pin(async move {
    //         let request = request_builder.send();
    //         let response = request
    //             .await
    //             .map_err(|e| Error::new(ErrorKind::NotConnected, format!("{e:?}")))?;

    //         match response.bytes().await {
    //             Ok(bytes) => Ok(bytes.to_vec()),
    //             Err(e) => Err(Error::new(ErrorKind::InvalidData, format!("{e:?}"))),
    //         }
    //     })
    // }
}
