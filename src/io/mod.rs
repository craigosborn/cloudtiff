// I/O Traits
//   Includes ReadRange and AsyncReadRange stateless I/O
//   These are supersets of Read + Seek and AsyncRead + AsyncSeek respectively
//   Key difference is self is immutable, making it powerful abstraction for http byte-range requests
//   Required methods
//     fn read_range(&self, start: u64, buf: &mut [u8]) -> Result<usize> { ... }
//   Provided methods
//     fn read_range_exact(&self, start: u64, buf: &mut [u8]) -> Result<()> { ... }
//     fn read_range_to_vec(&self, start: u64, end: u64) -> Result<()> { ... }
//   Async has similar

use futures::future::BoxFuture;
use futures::FutureExt;
use std::io::{Error, ErrorKind, Read, Result, Seek};
use std::sync::Mutex;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};
use tokio::sync::Mutex as TokioMutex;

pub mod http;
pub mod s3;
pub trait ReadRange {
    /// Read bytes from a specific offset
    ///
    /// This is a superset of std::io::{Read + Seek} with a key difference that
    /// self is immutable. This is a useful abstraction for concurrent I/O.
    ///
    /// Required methods
    ///   fn read_range(&self, start: u64, buf: &mut [u8]) -> Result<usize>;
    ///
    /// Provided methods
    ///   fn read_range_exact(&self, start: u64, buf: &mut [u8]) -> Result<()> { ... }
    ///   fn read_range_to_vec(&self, start: u64, end: u64) -> Result<()> { ... }

    fn read_range(&self, start: u64, buf: &mut [u8]) -> Result<usize>;

    fn read_range_exact(&self, start: u64, buf: &mut [u8]) -> Result<()> {
        let n = buf.len();
        let bytes_read = self.read_range(start, buf)?;
        if bytes_read == n {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::UnexpectedEof,
                format!("Failed to completely fill buffer: {bytes_read} < {n}"),
            ))
        }
    }

    fn read_range_to_vec(&self, start: u64, end: u64) -> Result<Vec<u8>> {
        let n = (end - start) as usize;
        let mut buf = vec![0; n];
        let _bytes_read = self.read_range_exact(start, &mut buf)?;
        Ok(buf)
    }
}

impl<R: Read + Seek> ReadRange for Mutex<R> {
    fn read_range(&self, start: u64, buf: &mut [u8]) -> Result<usize> {
        let mut locked_self = self
            .lock()
            .map_err(|e| Error::new(ErrorKind::Other, format!("{e:?}")))?;
        locked_self.seek(std::io::SeekFrom::Start(start))?;
        locked_self.read(buf)
    }
}

pub trait AsyncReadRange: Send + Sync {
    /// Asynchronously read bytes from a specific offset
    ///
    /// This is a superset of tokio::io::{AsyncRead + AsyncSeek} with a key difference that
    /// self is immutable. This is a useful abstraction for concurrent http byte-range requests.
    ///
    /// Required methods
    ///   fn read_range(&self, start: u64, buf: &mut [u8]) -> Result<usize>;
    ///
    /// Provided methods
    ///   fn read_range_exact(&self, start: u64, buf: &mut [u8]) -> Result<()> { ... }
    ///   fn read_range_to_vec(&self, start: u64, end: u64) -> Result<()> { ... }

    fn read_range_async<'a>(
        &'a self,
        start: u64,
        buf: &'a mut [u8],
    ) -> BoxFuture<'a, Result<usize>>;

    fn read_range_exact_async<'a>(
        &'a self,
        start: u64,
        buf: &'a mut [u8],
    ) -> BoxFuture<'a, Result<usize>> {
        let n = buf.len();
        async move {
            match self.read_range_async(start, buf).await {
                Ok(bytes_read) if bytes_read == n => Ok(bytes_read),
                Ok(bytes_read) => Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("Failed to completely fill buffer: {bytes_read} < {n}"),
                )),
                Err(e) => Err(e),
            }
        }
        .boxed()
    }

    // fn read_range_to_vec_async(&'a self, start: u64, end: u64) -> BoxFuture<Result<Vec<u8>>> {
    // TODO LIFETIMES ARE HARD
    //     let n = (end - start) as usize;
    //     let mut buffer =  vec![0; n];
    //     let buf: &'a Vec<u8> =  &mut buffer;
    //     async move {
    //         match self.read_range_async(start, buf).await {
    //             Ok(bytes_read) if bytes_read == n => Ok(buffer),
    //             Ok(bytes_read) => Err(Error::new(
    //                 ErrorKind::UnexpectedEof,
    //                 format!("Failed to completely fill buffer: {bytes_read} < {n}"),
    //             )),
    //             Err(e) => Err(e),
    //         }
    //     }
    //     .boxed()
    // }
}

impl<R: AsyncRead + AsyncSeek + Send + Sync + Unpin> AsyncReadRange for TokioMutex<R> {
    fn read_range_async<'a>(&'a self, start: u64, buf: &'a mut [u8]) -> BoxFuture<Result<usize>> {
        // Yes, it is rather ugly... but so is async.
        async move {
            let mut locked_self = self.lock().await;
            locked_self.seek(std::io::SeekFrom::Start(start)).await?;
            locked_self.read(buf).await
        }
        .boxed()
    }
}
