#![cfg(feature = "s3")]

use super::AsyncReadRange;
use aws_sdk_s3::{self, operation::get_object::builders::GetObjectFluentBuilder, Client};
use futures::future::BoxFuture;
use futures::FutureExt;
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
        let end = start + buf.len() as u64;
        println!("s3 buf: {}", buf.len());
        let req = self.request.clone().range(format!("bytes={start}-{end}"));
        async move {
            let response = req
                .send()
                .await
                .map_err(|e| Error::new(ErrorKind::NotConnected, format!("{e:?}")))?;
            match response.body.collect().await {
                Ok(slice) => Ok(slice.to_vec()),
                Err(e) => Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("ByteStream Error: {e:?}"),
                )),
            }
        }
        .boxed()
    }
}
