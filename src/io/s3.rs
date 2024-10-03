#![cfg(feature = "s3")]

use super::AsyncReadRange;
use aws_sdk_s3::{self, operation::get_object::builders::GetObjectFluentBuilder, Client};
use futures::future::BoxFuture;
use std::fmt;
use std::io::{Error, ErrorKind, Result};

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

impl fmt::Debug for S3Reader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("S3Reader")
            .field("bucket", &self.request.get_bucket().as_ref())
            .field("key", &self.request.get_key().as_ref())
            .finish()
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
