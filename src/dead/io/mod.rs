use futures::future::BoxFuture;
use futures::FutureExt;
use std::fmt::Debug;
use std::io::{Error, ErrorKind, Read, Result, Seek};
use tokio::io::{AsyncRead, AsyncSeek};

pub mod async_reader;
pub mod fs;
pub mod http;
pub mod path;
pub mod reader;
pub mod s3;

pub trait ReadSeek: Read + Seek + Debug + Send + Sync {}

pub trait ReadRange: Debug + Send + Sync {
    fn read_range(&self, start: u64, end: u64, buf: &mut [u8]) -> Result<usize>;
}

pub trait ReadRangeExt {
    fn read_range_exact(&self, start: u64, end: u64, buf: &mut [u8]) -> Result<()>;
}

impl<R: ReadRange> ReadRangeExt for R {
    fn read_range_exact(&self, start: u64, end: u64, buf: &mut [u8]) -> Result<()> {
        let n = (end - start) as usize;
        let bytes_read = self.read_range(start, end, buf)?;
        if bytes_read == n {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::UnexpectedEof,
                format!("Failed to completely fill buffer: {bytes_read} < {n}"),
            ))
        }
    }
}

pub trait AsyncReadSeek: AsyncRead + AsyncSeek + Debug + Send + Sync + Unpin {}

pub trait AsyncReadRange: Debug + Send + Sync {
    fn read_range_async<'a>(
        &self,
        start: u64,
        buf: &mut [u8],
    ) -> BoxFuture<'a, Result<usize>>;

    fn read_range_exact_async<'a>(
        &'a self,
        start: u64,
        buf: &'a mut [u8],
    ) -> BoxFuture<Result<()>> {
        let n = buf.len() as usize;
        async move {
            match self.read_range_async(start, buf).await {
                Ok(bytes_read) => {
                    if bytes_read == n {
                        Ok(())
                    } else {
                        Err(Error::new(
                            ErrorKind::UnexpectedEof,
                            format!("Failed to completely fill buffer: {bytes_read} < {n}"),
                        ))
                    }
                }
                Err(e) => Err(e),
            }
        }
        .boxed()
    }

    fn read_range_to_vec(&'a self, start: u64, end: u64) -> BoxFuture<'a, Result<Vec<u8>>> {
        let n = (end - start) as usize;
        let mut buf = vec![0; n];
        async move {
            self.read_range_async(start, &mut buf).await?;
            Ok(buf)
        }
        .boxed()
    }
}
