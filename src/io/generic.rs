use std::io::{Error, ErrorKind, SeekFrom};

use super::*;

#[derive(Debug)]
pub struct GenericReader {
    reader: ReaderFlavor,
    position: u64,
}


impl GenericReader {
    pub fn new<R: ReadRange>(reader: R) -> Self {
        Self {
            reader: ReaderFlavor::Sync(Box::new(reader)),
            position: 0,
        }
    }

    pub fn new_async<R: AsyncReadRange>(reader: R) -> Self {
        Self {
            reader: ReaderFlavor::Async(Box::new(reader)),
            position: 0,
        }
    }

    pub fn into_inner(self) -> ReaderFlavor {
        self.reader
    }

    pub fn position(&self) -> u64 {
        self.position
    }
}

impl Read for GenericReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let start = self.position;
        let end = start + buf.len() as u64;

        let bytes = match &self.reader {
            ReaderFlavor::Async(reader) => {
                let future = reader.read_range_async(start, end);
                futures::executor::block_on(future).map_err(|e| Error::new(ErrorKind::Other, e))
            }
            ReaderFlavor::Sync(reader) => reader.read_range(start, end),
        }?;
        let n = bytes.len();

        buf[..n].copy_from_slice(&bytes);
        self.position += n as u64;

        Ok(n)
    }
}

impl Seek for GenericReader {
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
