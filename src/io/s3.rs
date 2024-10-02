#![cfg(feature = "s3")]

use super::AsyncReadRange;
use aws_sdk_s3::{self, operation::get_object::builders::GetObjectFluentBuilder, Client};
use futures::future::BoxFuture;
use std::io::{Error, ErrorKind, Result};

#[derive(Clone, Debug)]
pub struct S3Reader {
    request: GetObjectFluentBuilder,
}

impl S3Reader {
    pub fn new(client: Client, bucket: &str, key: &str) -> Self {
        let request = client.get_object().bucket(bucket).key(key);
        Self { request }
    }
    pub fn from_request_builder(request: GetObjectFluentBuilder) -> Self {
        Self { request }
    }
}

impl AsyncReadRange for S3Reader {
    fn read_range_async<'a>(&'a self, start: u64, buf: &'a mut [u8]) -> BoxFuture<Result<usize>> {
        let n = buf.len();
        let end = start + n as u64 - 1; // GOTCHA byte range includes end
        let request_builder = self.request.clone().range(format!("bytes={start}-{end}"));

        Box::pin(async move {
            let request = request_builder.send();
            let mut response = request
                .await
                .map_err(|e| Error::new(ErrorKind::NotConnected, format!("{e:?}")))?;

            let mut pos = 0;
            while let Some(bytes) = response.body.try_next().await.map_err(|err| {
                Error::new(
                    ErrorKind::Interrupted,
                    format!("Failed to read from S3 download stream: {err:?}"),
                )
            })? {
                let bytes_len = bytes.len();
                let bytes_top = bytes_len.min(n - pos);
                let buf_top = n.min(pos + bytes_len);
                buf[pos..buf_top].copy_from_slice(&bytes[..bytes_top]);
                pos += bytes_len;
            }
            Ok(pos)
        })
    }
}
