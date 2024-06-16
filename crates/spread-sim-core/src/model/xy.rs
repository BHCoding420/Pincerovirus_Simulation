use std::{fmt::Display, ops};

use serde::{Deserialize, Serialize};

/// Represents a two-dimensional vector with an *x*- and a *y*-component.
///
/// Note that you can do arithmetic with [`Xy`], e.g.:
/// ```
/// # use spread_sim_core::model::xy::Xy;
/// assert_eq!(Xy::new(3, 4) + Xy::new(2, 3), Xy::new(5, 7));
/// assert_eq!(Xy::new(3, 4) + 2, Xy::new(5, 6));
/// assert_eq!(Xy::new(3, 4) - (2, 1), Xy::new(1, 3));
/// ```
///
/// Checkout [the implementations](#trait-implementations) of [`ops::Add`] and
/// [`ops::Sub`] for details.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Copy, Clone, Hash, Default)]
pub struct Xy {
    pub x: isize,
    pub y: isize,
}

impl Xy {
    /// The *origin*, i.e., a vector with both components set to zero.
    pub fn zero() -> Self {
        Self::default()
    }

    /// Constructs a vector with the given *x*- and *y*-components.
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    /// Limits the value of both components to the specified range.
    pub fn limit(&self, min: isize, max: isize) -> Self {
        Self {
            x: self.x.min(max).max(min),
            y: self.y.min(max).max(min),
        }
    }

    /// Limits the values of the respective components to the specified ranges.
    pub fn limit_xy(&self, min: &Xy, max: &Xy) -> Self {
        Self {
            x: self.x.min(max.x).max(min.x),
            y: self.y.min(max.y).max(min.y),
        }
    }
}

impl Display for Xy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl ops::Add for Xy {
    type Output = Xy;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl ops::Add<isize> for Xy {
    type Output = Xy;

    fn add(self, rhs: isize) -> Self::Output {
        Self::new(self.x + rhs, self.y + rhs)
    }
}

impl ops::Add<(isize, isize)> for Xy {
    type Output = Xy;

    fn add(self, rhs: (isize, isize)) -> Self::Output {
        Self::new(self.x + rhs.0, self.y + rhs.1)
    }
}

impl ops::Sub for Xy {
    type Output = Xy;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl ops::Sub<isize> for Xy {
    type Output = Xy;

    fn sub(self, rhs: isize) -> Self::Output {
        Self::new(self.x - rhs, self.y - rhs)
    }
}

impl ops::Sub<(isize, isize)> for Xy {
    type Output = Xy;

    fn sub(self, rhs: (isize, isize)) -> Self::Output {
        Self::new(self.x - rhs.0, self.y - rhs.1)
    }
}

impl From<(isize, isize)> for Xy {
    fn from(value: (isize, isize)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<Xy> for (isize, isize) {
    fn from(value: Xy) -> Self {
        (value.x, value.y)
    }
}
