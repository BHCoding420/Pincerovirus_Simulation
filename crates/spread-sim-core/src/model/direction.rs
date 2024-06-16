use serde::{Deserialize, Serialize};

use super::xy::Xy;

/// Represents a direction of movement.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    #[serde(rename = "N")]
    North,
    #[serde(rename = "E")]
    East,
    #[serde(rename = "S")]
    South,
    #[serde(rename = "W")]
    West,
    #[serde(rename = "NE")]
    NorthEast,
    #[serde(rename = "NW")]
    NorthWest,
    #[serde(rename = "SE")]
    SouthEast,
    #[serde(rename = "SW")]
    SouthWest,
    #[serde(rename = "X")]
    None,
}

impl Direction {
    /// Converts an [`Xy`] vector into a direction.
    pub fn from_vector(vector: Xy) -> Self {
        match (vector.x, vector.y) {
            (0, -1) => Self::North,
            (1, 0) => Self::East,
            (0, 1) => Self::South,
            (-1, 0) => Self::West,
            (1, -1) => Self::NorthEast,
            (-1, -1) => Self::NorthWest,
            (1, 1) => Self::SouthEast,
            (-1, 1) => Self::SouthWest,
            (0, 0) => Self::None,
            _ => panic!("invalid direction vector {:?}", vector),
        }
    }

    /// Converts the direction into an [`Xy`] vector.
    pub fn vector(self) -> Xy {
        match self {
            Self::North => Xy::new(0, -1),
            Self::East => Xy::new(1, 0),
            Self::South => Xy::new(0, 1),
            Self::West => Xy::new(-1, 0),
            Self::NorthEast => Xy::new(1, -1),
            Self::NorthWest => Xy::new(-1, -1),
            Self::SouthEast => Xy::new(1, 1),
            Self::SouthWest => Xy::new(-1, 1),
            Self::None => Xy::new(0, 0),
        }
    }

    /// Turns a numeric index into a direction.
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::North,
            1 => Self::East,
            2 => Self::South,
            3 => Self::West,
            4 => Self::NorthEast,
            5 => Self::NorthWest,
            6 => Self::SouthEast,
            7 => Self::SouthWest,
            _ => Self::None,
        }
    }
}
