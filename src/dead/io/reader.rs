use super::{ReadRange, ReadSeek};
use std::fmt::Debug;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum SyncReader {
    // TODO is Mutex necessary when AsyncReader exists?
    ReadRange(Arc<(Box<dyn ReadRange>, Mutex<u64>)>),
    ReadSeek(Arc<Mutex<dyn ReadSeek>>),
}

impl SyncReader {
    pub fn from_reader<R: ReadSeek + 'static>(reader: R) -> Self {
        Self::ReadSeek(Arc::new(Mutex::new(reader)))
    }

    pub fn from_range_reader<R: ReadRange + 'static>(reader: R) -> Self {
        Self::ReadRange(Arc::new((Box::new(reader), Mutex::new(0))))
    }
}

impl ReadRange for SyncReader {
    fn read_range(&self, start: u64, end: u64, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::ReadRange(reader) => reader.0.read_range(start, end, buf),
            Self::ReadSeek(reader) => match reader.lock() {
                Ok(mut locked_reader) => match locked_reader.seek(SeekFrom::Start(start)) {
                    Ok(_) => {
                        let mut buffer = vec![0; (end - start) as usize];
                        locked_reader.read(&mut buffer)
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(Error::new(ErrorKind::Other, format!("{e:?}"))),
            },
        }
    }
}

impl Read for SyncReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::ReadRange(range_reader) => {
                let mut position = range_reader.1.lock().map_err(|e| Error::new(ErrorKind::Other, format!("{e:?}")))?;
                let end = *position + buf.len() as u64;
                let bytes_read = range_reader.0.read_range(*position, end, buf)?;
                *position += bytes_read as u64;
                Ok(bytes_read)
            }
            Self::ReadSeek(reader) => match reader.lock() {
                Ok(mut locked_reader) => locked_reader.read(buf),
                Err(e) => Err(Error::new(ErrorKind::Other, format!("{e:?}"))),
            },
        }
    }
}

impl Seek for SyncReader {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match self {
            Self::ReadRange(range_reader) => {
                let mut position = range_reader
                    .1
                    .lock()
                    .map_err(|e| Error::new(ErrorKind::Other, format!("{e:?}")))?;
                match pos {
                    SeekFrom::Start(offset) => {
                        *position = offset;
                        Ok(*position)
                    }
                    SeekFrom::Current(offset) => {
                        *position = position
                            .checked_add(offset as u64)
                            .ok_or(Error::new(ErrorKind::InvalidInput, "Seek overflow"))?;
                        Ok(*position)
                    }
                    SeekFrom::End(_offset) => Err(Error::new(
                        ErrorKind::Unsupported,
                        "Seek from end not supported",
                    )),
                }
            }
            Self::ReadSeek(reader) => match reader.lock() {
                Ok(mut locked_reader) => locked_reader.seek(pos),
                Err(e) => Err(Error::new(ErrorKind::Other, format!("{e:?}"))),
            },
        }
    }
}
