use super::{CloudTiff, CloudTiffError, CloudTiffResult};
use crate::reader::ReadRangeAsync;
use std::io::Cursor;

mod level;
mod render;

impl CloudTiff {
    pub async fn open_par<R: ReadRangeAsync>(source: &mut R) -> CloudTiffResult<Self> {
        let bytes = source
            .read_range(0, 16_384)
            .await
            .map_err(|e| CloudTiffError::ReadRangeError(format!("{e:?}")))?;
        let mut sync_reader = Cursor::new(bytes);
        Self::open(&mut sync_reader)
    }
}
