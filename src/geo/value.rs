use std::fmt::Display;

use num_traits::NumCast;

#[derive(Clone, Debug)]

pub enum GeoKeyValue {
    Short(Vec<u16>),
    Ascii(String),
    Double(Vec<f64>),
    Undefined,
}

impl GeoKeyValue {
    pub fn as_string(&self) -> Option<&String> {
        match self {
            GeoKeyValue::Ascii(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number<T: NumCast>(&self) -> Option<T> {
        match self {
            GeoKeyValue::Double(v) if v.len() == 1 => T::from(v[0]),
            GeoKeyValue::Short(v) if v.len() == 1 => T::from(v[0]),
            _ => None,
        }
    }

    pub fn as_vec<T: NumCast>(&self) -> Option<Vec<T>> {
        match self {
            GeoKeyValue::Double(v) => v.iter().map(|x| T::from(*x)).collect(),
            GeoKeyValue::Short(v) => v.iter().map(|x| T::from(*x)).collect(),
            _ => None,
        }
    }
}

impl Display for GeoKeyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeoKeyValue::Ascii(s) => write!(f, "{}", s.replace("\n", "\\n")),
            GeoKeyValue::Double(v) if v.len() == 1 => write!(f, "{}", v[0]),
            GeoKeyValue::Short(v) if v.len() == 1 => write!(f, "{}", v[0]),
            GeoKeyValue::Double(v) => write!(f, "{v:?}"),
            GeoKeyValue::Short(v) => write!(f, "{v:?}"),
            GeoKeyValue::Undefined => write!(f, "Undefined"),
        }
    }
}
