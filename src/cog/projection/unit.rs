use core::f64;
use std::convert::TryFrom;
use std::fmt;

const MIN: UnitFloat = UnitFloat{v: 0.0};
const MAX: UnitFloat = UnitFloat{v: 1.0};

#[derive(Clone, Debug)]
pub struct UnitRegion {
    pub x_min: UnitFloat,
    pub x_max: UnitFloat,
    pub y_min: UnitFloat,
    pub y_max: UnitFloat,
}

impl Default for UnitRegion {
    fn default() -> Self {
        Self {
            x_min: MIN,
            x_max: MAX,
            y_min: MIN,
            y_max: MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitFloat {
    v: f64,
}

impl UnitFloat {
    pub fn as_f64(self) -> f64 {
        self.v
    }

    pub fn zero() -> Self {
        Self { v: 0.0 }
    }

    pub fn one() -> Self {
        Self { v: 1.0 }
    }

    pub fn min() -> Self {
        MIN
    }

    pub fn max() -> Self {
        MAX
    }
}

impl From<UnitFloat> for f64 {
    fn from(unit_float: UnitFloat) -> Self {
        unit_float.v
    }
}

impl TryFrom<f64> for UnitFloat {
    type Error = &'static str;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value >= 0.0 && value <= 1.0 {
            Ok(UnitFloat{v: value})
        } else {
            Err("Value must be between 0.0 and 1.0 inclusive.")
        }
    }
}

impl fmt::Display for UnitFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.v)
    }
}
