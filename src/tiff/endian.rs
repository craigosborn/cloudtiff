use eio::{FromBytes, ReadExt, ToBytes};
use num_traits::{cast::NumCast, ToPrimitive};
use std::io::{Read, Result};
use std::mem;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Endian {
    Big,
    Little,
}

impl Endian {
    pub fn read<const N: usize, T: FromBytes<N>>(&self, stream: &mut impl Read) -> Result<T> {
        let mut buf = [0u8; N];
        stream.read_exact(&mut buf)?;
        self.decode(buf)
    }

    pub fn decode<const N: usize, T: FromBytes<N>>(&self, bytes: [u8; N]) -> Result<T> {
        match self {
            Endian::Big => bytes.as_slice().read_be(),
            Endian::Little => bytes.as_slice().read_le(),
        }
    }

    pub fn decode_all<const N: usize, T: FromBytes<N>>(&self, bytes: &[u8]) -> Option<Vec<T>> {
        bytes
            .chunks_exact(mem::size_of::<T>())
            .map(|chunk| {
                chunk
                    .try_into()
                    .ok()
                    .and_then(|arr| self.decode::<N, T>(arr).ok())
            })
            .collect()
    }

    pub fn decode_to_primative<const N: usize, A: FromBytes<N> + ToPrimitive, T: NumCast>(
        &self,
        bytes: [u8; N],
    ) -> Option<T> {
        self.decode::<N, A>(bytes).ok().and_then(|v| T::from(v))
    }

    pub fn decode_all_to_primative<const N: usize, A: FromBytes<N> + ToPrimitive, T: NumCast>(
        &self,
        bytes: &[u8],
    ) -> Option<Vec<T>> {
        self.decode_all::<N, A>(bytes)?
            .into_iter()
            .map(|v| T::from(v))
            .collect()
    }

    pub fn encode<const N: usize, T: ToBytes<N>>(&self, value: T) -> [u8; N] {
        match self {
            Endian::Big => value.to_be_bytes(),
            Endian::Little => value.to_le_bytes(),
        }
    }

    pub fn encode_all<const N: usize, T: ToBytes<N> + Copy>(&self, values: &[T]) -> Vec<u8> {
        values.iter().flat_map(|v| self.encode(*v)).collect()
    }
}
