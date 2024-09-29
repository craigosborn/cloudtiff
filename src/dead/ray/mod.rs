use super::{CloudTiff, CloudTiffError, CloudTiffResult};
use crate::reader::ReadRange;
use std::io::Cursor;

mod level;
mod render;

impl CloudTiff {
    pub fn open_ray<R: ReadRange>(source: &mut R) -> CloudTiffResult<Self> {
        let bytes = source
            .read_range(0, 16_384)
            .map_err(|e| CloudTiffError::ReadRangeError(format!("{e:?}")))?;
        let mut sync_reader = Cursor::new(bytes);
        Self::open(&mut sync_reader)
    }
}
