use core::f64;
use std::fmt;

const RANGE_ERROR_MSG: &str = "Value must be on the closed range [0.0, 1.0]";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitFloat(f64);

impl UnitFloat {
    pub const MIN: UnitFloat = UnitFloat(0.0);
    pub const MAX: UnitFloat = UnitFloat(1.0);

    pub fn new<V: TryInto<f64>>(value: V) -> Result<Self, String> {
        let Ok(v) = value.try_into() else {
            return Err("Value could not be interpreted as f64".to_string());
        };
        if v >= 0.0 && v <= 1.0 {
            Ok(Self(v))
        } else {
            Err(RANGE_ERROR_MSG.to_string())
        }
    }

    pub fn as_f64(self) -> f64 {
        self.0
    }

    pub fn zero() -> Self {
        Self::MIN
    }

    pub fn one() -> Self {
        Self::MAX
    }

    pub fn min() -> Self {
        Self::MIN
    }

    pub fn max() -> Self {
        Self::MAX
    }
}

impl From<UnitFloat> for f64 {
    fn from(unit_float: UnitFloat) -> Self {
        unit_float.as_f64()
    }
}

impl fmt::Display for UnitFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitRange {
    pub min: UnitFloat,
    pub max: UnitFloat,
}

impl UnitRange {
    pub fn new(min: f64, max: f64) -> Option<Self> {
        Some(Self {
            min: UnitFloat::new(min.min(max)).ok()?,
            max: UnitFloat::new(min.max(max)).ok()?,
        })
    }
}

impl Default for UnitRange {
    fn default() -> Self {
        Self {
            min: UnitFloat::MIN,
            max: UnitFloat::MAX,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct UnitRegion {
    pub x: UnitRange,
    pub y: UnitRange,
}

impl UnitRegion {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Option<Self> {
        Some(Self {
            x: UnitRange::new(min_x, max_x)?,
            y: UnitRange::new(min_y, max_y)?,
        })
    }
    pub fn from_unit_ranges<R: Into<UnitRange>>(x: R, y: R) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    pub fn as_f64(&self) -> (f64, f64, f64, f64) {
        (self.x.min.0, self.y.min.0, self.x.max.0, self.y.max.0)
    }

    pub fn x(&self) -> &UnitRange {
        &self.x
    }

    pub fn y(&self) -> &UnitRange {
        &self.y
    }

    pub fn x_min(&self) -> UnitFloat {
        self.x.min
    }

    pub fn y_min(&self) -> UnitFloat {
        self.y.min
    }

    pub fn x_max(&self) -> UnitFloat {
        self.x.max
    }

    pub fn y_max(&self) -> UnitFloat {
        self.y.max
    }
}
