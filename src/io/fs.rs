#![cfg(feature = "fs")]

use super::ReadRange;
use super::ReadRangeAsync;
use futures::future::BoxFuture;
use futures::FutureExt;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::{Read, Result, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use tokio::fs::File as TokioFile;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

impl ReadRange for File {
    fn read_range(&self, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut file_clone = self.try_clone()?;
        file_clone.seek(SeekFrom::Start(start))?;
        let mut buffer = vec![0; (end - start) as usize];
        file_clone.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}

impl ReadRangeAsync for TokioFile {
    fn read_range_async(&self, start: u64, end: u64) -> BoxFuture<'static, Result<Vec<u8>>> {
        // Yes, it is rather ugly... but so is async.
        let maybe_cloned = futures::executor::block_on(self.try_clone())
            .map_err(|e| Error::new(ErrorKind::Other, e));
        async move {
            let mut file_clone = maybe_cloned?;
            file_clone.seek(SeekFrom::Start(start)).await?;
            let mut buffer = vec![0; (end - start) as usize];
            file_clone.read_exact(&mut buffer).await?;
            Ok(buffer)
        }
        .boxed()
    }
}

#[derive(Clone, Debug)]
pub struct PathReader(PathBuf);

impl PathReader {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self(path.as_ref().to_path_buf())
    }
}

impl ReadRange for PathReader {
    fn read_range(&self, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut file = File::open(self.0.clone())?;
        file.seek(SeekFrom::Start(start))?;
        let mut buffer = vec![0; (end - start) as usize];
        file.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}

impl ReadRangeAsync for PathReader {
    fn read_range_async(&self, start: u64, end: u64) -> BoxFuture<'static, Result<Vec<u8>>> {
        let path = self.0.clone();
        async move {
            let mut file = TokioFile::open(path).await?;
            file.seek(SeekFrom::Start(start)).await?;
            let mut buffer = vec![0; (end - start) as usize];
            file.read_exact(&mut buffer).await?;
            Ok(buffer)
        }
        .boxed()
    }
}
