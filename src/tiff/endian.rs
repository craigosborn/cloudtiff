use eio::{FromBytes, ReadExt};
use std::io::{Read, Result};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Endian {
    Big,
    Little,
}

impl Endian {
    pub fn decode<const N: usize, T: FromBytes<N>>(&self, bytes: [u8; N]) -> Result<T> {
        match self {
            Endian::Big => bytes.as_slice().read_be(),
            Endian::Little => bytes.as_slice().read_le(),
        }
    }
    pub fn read<const N: usize, T: FromBytes<N>>(&self, stream: &mut impl Read) -> Result<T> {
        let mut buf = [0u8; N];
        stream.read_exact(&mut buf)?;
        self.decode(buf)
    }
}
