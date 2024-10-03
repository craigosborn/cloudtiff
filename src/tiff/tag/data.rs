use super::TagType;
use crate::tiff::Endian;

#[derive(Clone, Debug)]
pub enum TagData {
    Byte(Vec<u8>),
    Ascii(Vec<u8>),
    Short(Vec<u16>),
    Long(Vec<u32>),
    Rational(Vec<(u32, u32)>),
    SByte(Vec<i8>),
    Undefined(Vec<u8>),
    SShort(Vec<i16>),
    SLong(Vec<i32>),
    SRational(Vec<(i32, i32)>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    Ifd(u32),
    Long8(Vec<u64>),
    SLong8(Vec<i64>),
    Ifd8(u64),
    Unknown(Vec<u8>),
}

impl TagData {
    pub fn from_string(s: &str) -> Self{
        Self::Ascii(s.as_bytes().to_vec())
    }

    pub fn from_short(v: u16) -> Self{
        Self::Short(vec![v])
    }

    pub fn from_long(v: u32) -> Self{
        Self::Long(vec![v])
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Byte(vec) => vec.len(),
            Self::Ascii(vec) => vec.len(),
            Self::Short(vec) => vec.len(),
            Self::Long(vec) => vec.len(),
            Self::Rational(vec) => vec.len(),
            Self::SByte(vec) => vec.len(),
            Self::Undefined(vec) => vec.len(),
            Self::SShort(vec) => vec.len(),
            Self::SLong(vec) => vec.len(),
            Self::SRational(vec) => vec.len(),
            Self::Float(vec) => vec.len(),
            Self::Double(vec) => vec.len(),
            Self::Ifd(_) => 1,
            Self::Long8(vec) => vec.len(),
            Self::SLong8(vec) => vec.len(),
            Self::Ifd8(_) => 1,
            Self::Unknown(vec) => vec.len(),
        }
    }

    pub fn tag_type(&self) -> TagType {
        match self {
            Self::Byte(_) => TagType::Byte,
            Self::Ascii(_) => TagType::Ascii,
            Self::Short(_) => TagType::Short,
            Self::Long(_) => TagType::Long,
            Self::Rational(_) => TagType::Rational,
            Self::SByte(_) => TagType::SByte,
            Self::Undefined(_) => TagType::Undefined,
            Self::SShort(_) => TagType::SShort,
            Self::SLong(_) => TagType::SLong,
            Self::SRational(_) => TagType::SRational,
            Self::Float(_) => TagType::Float,
            Self::Double(_) => TagType::Double,
            Self::Ifd(_) => TagType::Ifd,
            Self::Long8(_) => TagType::Long8,
            Self::SLong8(_) => TagType::SLong8,
            Self::Ifd8(_) => TagType::Ifd8,
            Self::Unknown(_) => TagType::Unknown,
        }
    }

    pub fn bytes(&self, endian: Endian) -> Vec<u8> {
        match self {
            Self::Byte(vec) => endian.encode_all(vec),
            Self::Ascii(vec) => endian.encode_all(vec),
            Self::Short(vec) => endian.encode_all(vec),
            Self::Long(vec) => endian.encode_all(vec),
            Self::Rational(vec) => vec
                .iter()
                .map(|(a, b)| {
                    endian
                        .encode(*a)
                        .into_iter()
                        .chain(endian.encode(*b).into_iter())
                        .collect::<Vec<u8>>()
                })
                .flatten()
                .collect(),
            Self::SByte(vec) => endian.encode_all(vec),
            Self::Undefined(vec) => endian.encode_all(vec),
            Self::SShort(vec) => endian.encode_all(vec),
            Self::SLong(vec) => endian.encode_all(vec),
            Self::SRational(vec) => vec
                .iter()
                .map(|(a, b)| {
                    endian
                        .encode(*a)
                        .into_iter()
                        .chain(endian.encode(*b).into_iter())
                        .collect::<Vec<u8>>()
                })
                .flatten()
                .collect(),
            Self::Float(vec) => endian.encode_all(vec),
            Self::Double(vec) => endian.encode_all(vec),
            Self::Ifd(v) => endian.encode(*v).to_vec(),
            Self::Long8(vec) => endian.encode_all(vec),
            Self::SLong8(vec) => endian.encode_all(vec),
            Self::Ifd8(v) => endian.encode(*v).to_vec(),
            Self::Unknown(vec) => endian.encode_all(vec),
        }
    }
}
