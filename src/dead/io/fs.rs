#![cfg(feature = "fs")]

use super::{AsyncReadSeek, ReadSeek};
use std::fs::File;
use std::io::{Read, Seek, BufReader};
use std::fmt::Debug;
use tokio::fs::File as TokioFile;

impl ReadSeek for File {}
impl ReadSeek for &'static mut File {}
impl<R: Read + Seek + Debug + Sync + Send + 'static> ReadSeek for BufReader<R> {}

impl AsyncReadSeek for TokioFile {}
impl AsyncReadSeek for &'static mut TokioFile {}

// impl ReadRange for File {
//     fn read_range(&self, start: u64, end: u64) -> Result<Vec<u8>> {
//         let mut file_clone = self.try_clone()?;
//         file_clone.seek(SeekFrom::Start(start))?;
//         let mut buffer = vec![0; (end - start) as usize];
//         file_clone.read_exact(&mut buffer)?;
//         Ok(buffer)
//     }
// }

// impl ReadRangeAsync for TokioFile {
//     fn read_range_async(&self, start: u64, end: u64) -> BoxFuture<'static, Result<Vec<u8>>> {
//         // Yes, it is rather ugly... but so is async.
//         let maybe_cloned = futures::executor::block_on(self.try_clone())
//             .map_err(|e| Error::new(ErrorKind::Other, e));
//         async move {
//             let mut file_clone = maybe_cloned?;
//             file_clone.seek(SeekFrom::Start(start)).await?;
//             let mut buffer = vec![0; (end - start) as usize];
//             file_clone.read_exact(&mut buffer).await?;
//             Ok(buffer)
//         }
//         .boxed()
//     }
// }
