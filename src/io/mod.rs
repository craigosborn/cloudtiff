use futures::future::BoxFuture;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};

pub mod fs;
pub mod http;
pub mod s3;

pub trait ReadRange: Send + Sync + 'static {
    fn read_range(&self, start: u64, end: u64) -> Result<Vec<u8>>;
}

pub trait ReadRangeAsync: Send + Sync + 'static {
    fn read_range_async(&self, start: u64, end: u64) -> BoxFuture<'static, Result<Vec<u8>>>;
}

pub struct RangeReader {
    reader: Flavor,
    position: u64,
}

pub enum Flavor {
    Sync(Box<dyn ReadRange>),
    Async(Box<dyn ReadRangeAsync>),
}

impl RangeReader {
    pub fn new<R: ReadRange>(reader: R) -> Self {
        RangeReader {
            reader: Flavor::Sync(Box::new(reader)),
            position: 0,
        }
    }

    pub fn new_async<R: ReadRangeAsync>(reader: R) -> Self {
        RangeReader {
            reader: Flavor::Async(Box::new(reader)),
            position: 0,
        }
    }

    pub fn into_inner(self) -> Flavor {
        self.reader
    }

    pub fn position(&self) -> u64 {
        self.position
    }
}

impl Read for RangeReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let start = self.position;
        let end = start + buf.len() as u64;

        let bytes = match &self.reader {
            Flavor::Async(reader) => {
                let future = reader.read_range_async(start, end);
                futures::executor::block_on(future).map_err(|e| Error::new(ErrorKind::Other, e))
            }
            Flavor::Sync(reader) => reader.read_range(start, end),
        }?;
        let n = bytes.len();

        buf[..n].copy_from_slice(&bytes);
        self.position += n as u64;

        Ok(n)
    }
}

impl Seek for RangeReader {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Start(offset) => {
                self.position = offset;
                Ok(self.position)
            }
            SeekFrom::Current(offset) => {
                self.position = self
                    .position
                    .checked_add(offset as u64)
                    .ok_or(Error::new(ErrorKind::InvalidInput, "Seek overflow"))?;
                Ok(self.position)
            }
            SeekFrom::End(_offset) => Err(Error::new(
                ErrorKind::Unsupported,
                "Seek from end not supported",
            )),
        }
    }
}
