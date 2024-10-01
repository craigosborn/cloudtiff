#![cfg(feature = "fs")]

use super::AsyncReadRange;
use super::ReadRange;
use super::ReadSeek;
use futures::future::BoxFuture;
use futures::FutureExt;
use std::fmt::Debug;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::{Read, Result, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use tokio::fs::File as TokioFile;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

// TODO impl AsyncReadSeek

#[derive(Clone, Debug)]
pub struct PathReader {
    path: PathBuf,
    position: u64,
}

impl PathReader {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            position: 0,
        }
    }
}

impl ReadRange for PathReader {
    fn read_range(&self, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut file = File::open(self.path.clone())?;
        file.seek(SeekFrom::Start(start))?;
        let mut buffer = vec![0; (end - start) as usize];
        file.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}

impl AsyncReadRange for PathReader {
    fn read_range_async(
        &self,
        start: u64,
        end: u64,
        buf: &'static mut [u8],
    ) -> BoxFuture<'static, Result<usize>> {
        let path = self.path.clone();
        async move {
            let mut file = TokioFile::open(path).await?;
            file.seek(SeekFrom::Start(start)).await?;
            file.read(buf).await
        }
        .boxed()
    }
}

impl ReadSeek for PathReader {}

impl Read for PathReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut file = File::open(self.path.clone())?;
        file.seek(SeekFrom::Start(self.position))?;
        let n = file.read(buf)?;
        self.position += n as u64;
        Ok(n)
    }
}

impl Seek for PathReader {
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
