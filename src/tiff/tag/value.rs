use std::fmt::Display;

use super::{Tag, TagType};

pub enum TagValue {
    Empty,
    String(String),
    Number(f64),
    Array(Vec<f64>),
    Undefined,
}

impl Display for TagValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_num = |v: f64| {
            if v.fract() == 0.0 {
                format!("{}", v as u64)
            } else {
                format!("{}", v as u64)
            }
        };
        match self {
            TagValue::Empty => write!(f, ""),
            TagValue::String(s) => write!(f, "{}", s.replace("\n", "\\n")),
            TagValue::Number(v) => write!(f, "{}", fmt_num(*v)),
            TagValue::Array(arr) => {
                write!(f, "[")?;
                let mut once = true;
                for v in arr.iter() {
                    if once {
                        once = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", fmt_num(*v))?;
                }
                write!(f, "]")
            }
            TagValue::Undefined => write!(f, "Undefined"),
        }
    }
}

impl From<&Tag> for TagValue {
    fn from(tag: &Tag) -> TagValue {
        let data_count = tag.data.len() / tag.datatype.size_in_bytes();
        match (tag.datatype, data_count) {
            (_, 0) => TagValue::Empty,
            (TagType::Byte, 1) => TagValue::Number(tag.data[0] as f64),
            (TagType::Byte, _) => TagValue::Array(tag.data.iter().map(|v| *v as f64).collect()),
            (TagType::Ascii, _) => match String::from_utf8(tag.data.clone()) {
                Ok(s) => TagValue::String(s),
                Err(_) => TagValue::Undefined,
            },
            (TagType::Short, 1) => match tag.data[..]
                .try_into()
                .ok()
                .and_then(|arr| tag.endian.parse::<2, u16>(arr).ok())
            {
                Some(v) => TagValue::Number(v as f64),
                None => TagValue::Undefined,
            },
            (TagType::Short, _) => TagValue::Array(
                tag.data
                    .chunks_exact(2)
                    .map(|c| {
                        c.try_into()
                            .ok()
                            .and_then(|arr| tag.endian.parse::<2, u16>(arr).map(|v| v as f64).ok())
                            .unwrap_or(f64::NAN)
                    })
                    .collect::<Vec<_>>(),
            ),
            (TagType::Long, 1) => match tag.data[..]
                .try_into()
                .ok()
                .and_then(|arr| tag.endian.parse::<4, u32>(arr).ok())
            {
                Some(v) => TagValue::Number(v as f64),
                None => TagValue::Undefined,
            },
            (TagType::Long, _) => TagValue::Array(
                tag.data
                    .chunks_exact(4)
                    .map(|c| {
                        c.try_into()
                            .ok()
                            .and_then(|arr| // chunk_exact(4) guarantees [u8; 4]
                        tag.endian
                            .parse::<4,u32>(arr)
                            .map(|v| v as f64).ok())
                            .unwrap_or(f64::NAN)
                    })
                    .collect::<Vec<_>>(),
            ),
            (TagType::Rational, 1) => {
                let numerator = tag.data[0..4]
                    .try_into()
                    .ok()
                    .and_then(|arr| tag.endian.parse::<4, u32>(arr).ok().map(|v| v as f64))
                    .unwrap_or(f64::NAN);

                let denominator = tag.data[4..8]
                    .try_into()
                    .ok()
                    .and_then(|arr| tag.endian.parse::<4, u32>(arr).ok().map(|v| v as f64))
                    .unwrap_or(f64::NAN);

                TagValue::Number(numerator / denominator)
            }
            (TagType::Rational, _) => TagValue::Array(
                tag.data
                    .chunks_exact(8)
                    .map(|c| {
                        let numerator = c[0..4]
                            .try_into()
                            .ok()
                            .and_then(|arr| tag.endian.parse::<4, u32>(arr).ok().map(|v| v as f64))
                            .unwrap_or(f64::NAN);

                        let denominator = c[4..8]
                            .try_into()
                            .ok()
                            .and_then(|arr| tag.endian.parse::<4, u32>(arr).ok().map(|v| v as f64))
                            .unwrap_or(f64::NAN);

                        numerator / denominator
                    })
                    .collect(),
            ),
            (TagType::SByte, 1) => TagValue::Number(i8::from_be_bytes([tag.data[0]]) as f64),
            (TagType::SByte, _) => TagValue::Array(
                tag.data
                    .iter()
                    .map(|v| i8::from_be_bytes([*v]) as f64)
                    .collect(),
            ),
            (TagType::Undefined, _) => TagValue::Undefined,
            (TagType::SShort, 1) => match tag.data[..]
                .try_into()
                .ok()
                .and_then(|arr| tag.endian.parse::<2, i16>(arr).ok())
            {
                Some(v) => TagValue::Number(v as f64),
                None => TagValue::Undefined,
            },
            (TagType::SShort, _) => TagValue::Array(
                tag.data
                    .chunks_exact(2)
                    .map(|c| {
                        c.try_into()
                            .ok()
                            .and_then(|arr| tag.endian.parse::<2, i16>(arr).map(|v| v as f64).ok())
                            .unwrap_or(f64::NAN)
                    })
                    .collect::<Vec<_>>(),
            ),
            (TagType::SLong, 1) => match tag.data[..]
                .try_into()
                .ok()
                .and_then(|arr| tag.endian.parse::<4, i32>(arr).ok())
            {
                Some(v) => TagValue::Number(v as f64),
                None => TagValue::Undefined,
            },
            (TagType::SLong, _) => TagValue::Array(
                tag.data
                    .chunks_exact(4)
                    .map(|c| {
                        c.try_into()
                            .ok()
                            .and_then(|arr| tag.endian.parse::<4, i32>(arr).map(|v| v as f64).ok())
                            .unwrap_or(f64::NAN)
                    })
                    .collect::<Vec<_>>(),
            ),
            (TagType::SRational, 1) => {
                let numerator = tag.data[0..4]
                    .try_into()
                    .ok()
                    .and_then(|arr| tag.endian.parse::<4, i32>(arr).ok().map(|v| v as f64))
                    .unwrap_or(f64::NAN);

                let denominator = tag.data[4..8]
                    .try_into()
                    .ok()
                    .and_then(|arr| tag.endian.parse::<4, i32>(arr).ok().map(|v| v as f64))
                    .unwrap_or(f64::NAN);

                TagValue::Number(numerator / denominator)
            }
            (TagType::SRational, _) => TagValue::Array(
                tag.data
                    .chunks_exact(8)
                    .map(|c| {
                        let numerator = c[0..4]
                            .try_into()
                            .ok()
                            .and_then(|arr| tag.endian.parse::<4, i32>(arr).ok().map(|v| v as f64))
                            .unwrap_or(f64::NAN);

                        let denominator = c[4..8]
                            .try_into()
                            .ok()
                            .and_then(|arr| tag.endian.parse::<4, i32>(arr).ok().map(|v| v as f64))
                            .unwrap_or(f64::NAN);

                        numerator / denominator
                    })
                    .collect(),
            ),
            (TagType::Float, 1) => match tag.data[..]
                .try_into()
                .ok()
                .and_then(|arr| tag.endian.parse::<4, f32>(arr).ok())
            {
                Some(v) => TagValue::Number(v as f64),
                None => TagValue::Undefined,
            },
            (TagType::Float, _) => TagValue::Array(
                tag.data
                    .chunks_exact(4)
                    .map(|c| {
                        c.try_into()
                            .ok()
                            .and_then(|arr| tag.endian.parse::<4, f32>(arr).map(|v| v as f64).ok())
                            .unwrap_or(f64::NAN)
                    })
                    .collect::<Vec<_>>(),
            ),
            (TagType::Double, 1) => match tag.data[..]
                .try_into()
                .ok()
                .and_then(|arr| tag.endian.parse::<8, f64>(arr).ok())
            {
                Some(v) => TagValue::Number(v as f64),
                None => TagValue::Undefined,
            },
            (TagType::Double, _) => TagValue::Array(
                tag.data
                    .chunks_exact(8)
                    .map(|c| {
                        c.try_into()
                            .ok()
                            .and_then(|arr| tag.endian.parse::<8, f64>(arr).ok())
                            .unwrap_or(f64::NAN)
                    })
                    .collect::<Vec<_>>(),
            ),
            // TagType::IFD => todo!(),
            // TagType::Long8 => todo!(),
            // TagType::SLong8 => todo!(),
            // TagType::IFD8 => todo!(),
            _ => TagValue::Undefined,
        }
    }
}
