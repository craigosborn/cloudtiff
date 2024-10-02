use core::f64;
use std::fmt;
use std::ops::Sub;

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
            Err("Value must be on the closed range [0.0, 1.0]".to_string())
        }
    }

    pub fn new_saturated(v: f64) -> Self {
        Self(v.clamp(0.0, 1.0))
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

impl Sub for UnitFloat {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new_saturated(self.0 - rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D<T> {
    pub x: T,
    pub y: T,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval<T> {
    pub min: T,
    pub max: T,
}

impl<T> Interval<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<T: Copy + Sub<Output = T>> Interval<T> {
    pub fn range(&self) -> T {
        self.max - self.min
    }
}

impl Interval<UnitFloat> {
    fn unit() -> Self {
        Self {
            min: UnitFloat::MIN,
            max: UnitFloat::MAX,
        }
    }
    fn new_saturated(min: f64, max: f64) -> Self {
        let low = min.min(max);
        let high = min.max(max);
        Self {
            min: UnitFloat::new_saturated(low),
            max: UnitFloat::new_saturated(high),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Region<T> {
    pub x: Interval<T>,
    pub y: Interval<T>,
}

impl<T> Region<T> {
    pub fn new(min_x: T, min_y: T, max_x: T, max_y: T) -> Self {
        Self {
            x: Interval::new(min_x, max_x),
            y: Interval::new(min_y, max_y),
        }
    }

    pub fn x(&self) -> &Interval<T> {
        &self.x
    }

    pub fn y(&self) -> &Interval<T> {
        &self.y
    }
}

impl<T: Copy> Region<T> {
    pub fn as_tuple(&self) -> (T, T, T, T) {
        (self.x.min, self.y.min, self.x.max, self.y.max)
    }

    pub fn x_min(&self) -> T {
        self.x.min
    }

    pub fn y_min(&self) -> T {
        self.y.min
    }

    pub fn x_max(&self) -> T {
        self.x.max
    }

    pub fn y_max(&self) -> T {
        self.y.max
    }
}

impl Region<UnitFloat> {
    pub fn unit() -> Self {
        Self {
            x: Interval::unit(),
            y: Interval::unit(),
        }
    }

    pub fn new_saturated(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            x: Interval::new_saturated(min_x, max_x),
            y: Interval::new_saturated(min_y, max_y),
        }
    }
}

impl<T: Into<f64> + Copy> Region<T> {
    pub fn to_f64(&self) -> (f64, f64, f64, f64) {
        (
            self.x.min.into(),
            self.y.min.into(),
            self.x.max.into(),
            self.y.max.into(),
        )
    }
}
