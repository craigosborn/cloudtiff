use std::fmt::Display;

#[derive(Clone, Debug)]

pub enum GeoKeyValue {
    Short(Vec<u16>),
    Ascii(String),
    Double(Vec<f64>),
    Undefined,
}

impl GeoKeyValue {
    pub fn to_number<T: From<u16> + From<f64>>(&self) -> Option<T> {
        match self {
            GeoKeyValue::Double(v) => {
                if v.len() == 1 {
                    Some(v[0].to_owned().into())
                } else {
                    None
                }
            }
            GeoKeyValue::Short(v) => {
                if v.len() == 1 {
                    Some(v[0].to_owned().into())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Display for GeoKeyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeoKeyValue::Ascii(s) => write!(f, "{}", s.replace("\n", "\\n")),
            GeoKeyValue::Short(v) => {
                if v.len() == 1 {
                    write!(f, "{}", v[0])
                } else {
                    write!(f, "{:?}", v)
                }
            }
            GeoKeyValue::Double(v) => {
                if v.len() == 1 {
                    write!(f, "{}", v[0])
                } else {
                    write!(f, "{:?}", v)
                }
            }
            GeoKeyValue::Undefined => write!(f, "Undefined"),
        }
    }
}
