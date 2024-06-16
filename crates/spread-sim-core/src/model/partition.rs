use serde::{Deserialize, Serialize};

/// Represents a partition of the grid into patches.
///
/// A partition is represented by a list of strictly ascending *x*- and
/// *y*-coordinates that “cut” the grid into patches.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Partition {
    pub x: Vec<isize>,
    pub y: Vec<isize>,
}

impl Partition {
    pub fn new(x: Vec<isize>, y: Vec<isize>) -> Self {
        Self { x, y }
    }
}
