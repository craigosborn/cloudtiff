// refs
// https://web.archive.org/web/20220119170528/http://www.exif.org/Exif2-2.PDF
// https://web.archive.org/web/20190624045241if_/http://www.cipa.jp:80/std/documents/e/DC-008-Translation-2019-E.pdf
// https://www.media.mit.edu/pia/Research/deepview/exif.

use super::Endian;
use eio::FromBytes;
use num_enum::{FromPrimitive, IntoPrimitive};
use num_traits::{cast::NumCast, ToPrimitive};
use std::fmt::Display;

mod id;

pub use id::TagId;

#[derive(Clone, Debug)]
pub struct Tag {
    pub code: u16,
    pub datatype: TagType,
    pub count: usize,
    pub data: Vec<u8>,
    pub endian: Endian,
}

impl Tag {
    pub fn id(&self) -> Option<TagId> {
        TagId::try_from(self.code).ok()
    }

    pub fn value<T: NumCast + Copy>(&self) -> Option<T> {
        match self.values() {
            Some(v) if v.len() == 1 => Some(v[0]),
            _ => None,
        }
    }

    pub fn values<T: NumCast>(&self) -> Option<Vec<T>> {
        match self.datatype {
            TagType::Byte => self.decode::<1, u8, T>(),
            TagType::Ascii => self.decode::<1, u8, T>(),
            TagType::Short => self.decode::<2, u16, T>(),
            TagType::Long => self.decode::<4, u32, T>(),
            TagType::SByte => self.decode::<1, i8, T>(),
            TagType::Undefined => self.decode::<1, u8, T>(),
            TagType::SShort => self.decode::<2, i16, T>(),
            TagType::SLong => self.decode::<4, i32, T>(),
            TagType::Float => self.decode::<4, f32, T>(),
            TagType::Double => self.decode::<8, f64, T>(),
            TagType::Ifd => self.decode::<4, u32, T>(),
            TagType::Long8 => self.decode::<8, u64, T>(),
            TagType::SLong8 => self.decode::<8, i64, T>(),
            TagType::Ifd8 => self.decode::<8, u64, T>(),
            TagType::Unknown => self.decode::<1, u8, T>(),
            TagType::Rational => self.decode_rational::<4, u32, T>(),
            TagType::SRational => self.decode_rational::<4, i32, T>(),
        }
    }

    pub fn try_to_string(&self) -> Option<String> {
        match self.datatype {
            TagType::Ascii | TagType::Byte | TagType::Unknown => {
                String::from_utf8(self.data.clone()).ok()
            }
            _ => None,
        }
    }

    pub fn as_string_lossy(&self) -> String {
        match self.datatype {
            TagType::Ascii => String::from_utf8_lossy(&self.data).into_owned(),
            TagType::Float | TagType::Double | TagType::Rational | TagType::SRational => {
                match self.values::<f64>() {
                    Some(v) if v.len() == 1 => format!("{}", v[0]),
                    Some(v) => format!("{:?}", v),
                    None => format!("Undefined"),
                }
            }
            _ => match self.values::<i64>() {
                Some(v) if v.len() == 1 => format!("{}", v[0]),
                Some(v) => format!("{:?}", v),
                None => format!("Undefined"),
            },
        }
    }

    fn decode<const N: usize, A: FromBytes<N> + ToPrimitive, T: NumCast>(&self) -> Option<Vec<T>> {
        self.endian.decode_all_to_primative::<N, A, T>(&self.data)
    }

    fn decode_rational<const N: usize, A: FromBytes<N> + ToPrimitive, T: NumCast>(
        &self,
    ) -> Option<Vec<T>> {
        self.data
            .chunks_exact(2 * N)
            .map(|chunk| {
                chunk[..N]
                    .try_into()
                    .ok()
                    .and_then(|arr| {
                        self.endian
                            .decode::<N, A>(arr)
                            .ok()
                            .and_then(|v| v.to_f64())
                    })
                    .and_then(|numerator| {
                        chunk[N..]
                            .try_into()
                            .ok()
                            .and_then(|arr| {
                                self.endian
                                    .decode::<N, A>(arr)
                                    .ok()
                                    .and_then(|v| v.to_f64())
                            })
                            .and_then(|denominator| T::from(numerator / denominator))
                    })
            })
            .collect()
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value_string = format!("{}", self.as_string_lossy());
        if value_string.len() > 100 {
            value_string = format!("{}...", &value_string[..98])
        }
        let id_string = match self.id() {
            Some(id) => format!("{id:?}"),
            None => format!("Unknown({})", self.code),
        };
        write!(
            f,
            "{} {:?}[{}]: {}",
            id_string, self.datatype, self.count, value_string
        )
    }
}

#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, FromPrimitive)]
#[repr(u16)]
pub enum TagType {
    Byte = 1,
    Ascii = 2,
    Short = 3,
    Long = 4,
    Rational = 5,
    SByte = 6,
    Undefined = 7,
    SShort = 8,
    SLong = 9,
    SRational = 10,
    Float = 11,
    Double = 12,
    Ifd = 13,
    Long8 = 16,
    SLong8 = 17,
    Ifd8 = 18,

    #[num_enum(default)]
    Unknown = 0xFFFF,
}

impl TagType {
    pub const fn size_in_bytes(&self) -> usize {
        match self {
            TagType::Byte => 1,
            TagType::Ascii => 1,
            TagType::Short => 2,
            TagType::Long => 4,
            TagType::Rational => 8,
            TagType::SByte => 1,
            TagType::Undefined => 1,
            TagType::SShort => 2,
            TagType::SLong => 4,
            TagType::SRational => 8,
            TagType::Float => 4,
            TagType::Double => 8,
            TagType::Ifd => 4,
            TagType::Long8 => 8,
            TagType::SLong8 => 8,
            TagType::Ifd8 => 8,

            TagType::Unknown => 1,
        }
    }
}
