// refs
// https://web.archive.org/web/20220119170528/http://www.exif.org/Exif2-2.PDF
// https://web.archive.org/web/20190624045241if_/http://www.cipa.jp:80/std/documents/e/DC-008-Translation-2019-E.pdf
// https://www.media.mit.edu/pia/Research/deepview/exif.

use super::Endian;
use eio::FromBytes;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::fmt::Display;

mod id;
mod value;

pub use id::TagId;
pub use value::TagValue;

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
    pub fn value(&self) -> TagValue {
        TagValue::from(self)
    }
    pub fn raw_values<const N: usize, T: FromBytes<N>>(&self) -> Vec<Option<T>> {
        // Does not coerce, will be None if requested type is not datatype
        self.data
            .chunks_exact(self.datatype.size_in_bytes())
            .map(|c| {
                c.try_into()
                    .ok()
                    .and_then(|arr| self.endian.decode(arr).ok())
            })
            .collect()
    }
    pub fn try_raw_values<const N: usize, T: FromBytes<N>>(&self) -> Option<Vec<T>> {
        self.raw_values().into_iter().collect()
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value_string = format!("{}", self.value());
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
    pub fn size_in_bytes(&self) -> usize {
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
