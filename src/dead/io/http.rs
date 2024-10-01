#![cfg(feature = "http")]

use super::AsyncReadRange;
use futures::future::BoxFuture;
use futures::FutureExt;
use reqwest::header::RANGE;
use reqwest::{Client, IntoUrl, Url};
use std::io::{Error, ErrorKind, Result};

#[derive(Clone, Debug)]
pub struct HttpReader(Url);

impl HttpReader {
    pub fn new<U: IntoUrl>(url: U) -> Result<Self> {
        let inner = url
            .into_url()
            .map_err(|e| Error::new(ErrorKind::AddrNotAvailable, format!("{e:?}")))?;
        Ok(Self(inner))
    }
}

impl AsyncReadRange for HttpReader {
    fn read_range_async(&self, start: u64, end: u64, buf: &mut [u8]) -> BoxFuture<'static, Result<Vec<u8>>> {
        let url = self.0.clone();
        println!("http rra url: {url}");
        async move {
            let request = Client::new()
                .get(url)
                .header(RANGE, format!("bytes={start}-{end}"))
                .send();
            println!("http rra request range {}", format!("bytes={start}-{end}"));

            let response = request
                .await
                .map_err(|e| Error::new(ErrorKind::NotConnected, format!("{e:?}")))?;

            println!("http rra response: {response:?}");
            let bytes = response
                .bytes()
                .await
                .map_err(|e| Error::new(ErrorKind::InvalidData, format!("{e:?}")))?;
            Ok(bytes.to_vec())
        }
        .boxed()
    }
}
