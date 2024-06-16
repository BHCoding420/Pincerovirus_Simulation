use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::xy::Xy;

/// Helper structure for deserialization.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RectangleData {
    /// The top-left coordinate of the rectangle.
    top_left: Xy,
    /// The size of the rectangle.
    size: Xy,
}

/// A rectangle.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase", try_from = "RectangleData")]
pub struct Rectangle {
    /// The top-left coordinate of the rectangle.
    pub top_left: Xy,
    /// The bottom-right coordinate of the rectangle.
    #[serde(skip)]
    pub bottom_right: Xy,
    /// The size of the rectangle.
    pub size: Xy,
}

impl Rectangle {
    /// Constructs a new [`Rectangle`].
    pub fn new(top_left: Xy, size: Xy) -> Self {
        Self {
            top_left,
            size,
            bottom_right: top_left + size,
        }
    }

    /// Checks whether this rectangle and another rectangle overlap.
    pub fn overlaps(&self, other: &Rectangle) -> bool {
        !(self.bottom_right.x <= other.top_left.x
            || other.bottom_right.x <= self.top_left.x
            || self.top_left.y >= other.bottom_right.y
            || other.top_left.y >= self.bottom_right.y)
    }

    /// Computes the intersection of two overlapping rectangles.
    ///
    /// # Panics
    ///
    /// Panics in case both rectangles do not overlap.
    pub fn intersect(&self, other: &Rectangle) -> Self {
        assert!(self.overlaps(other), "Rectangles must overlap!");
        let top_left = Xy::new(
            self.top_left.x.max(other.top_left.x),
            self.top_left.y.max(other.top_left.y),
        );
        let bottom_right = Xy::new(
            self.bottom_right.x.min(other.bottom_right.x),
            self.bottom_right.y.min(other.bottom_right.y),
        );
        let size = bottom_right - top_left;
        Self {
            top_left,
            bottom_right,
            size,
        }
    }

    /// Checks whether the rectangle contains a cell.
    pub fn contains(&self, cell: &Xy) -> bool {
        self.top_left.x <= cell.x
            && cell.x < self.bottom_right.x
            && self.top_left.y <= cell.y
            && cell.y < self.bottom_right.y
    }

    /// Returns an iterator over the cells of the rectangle.
    pub fn iter_cells(&self) -> CellIterator<'_> {
        CellIterator {
            next_cell: self.top_left,
            rectangle: self,
        }
    }
}

impl From<RectangleData> for Rectangle {
    fn from(value: RectangleData) -> Self {
        Self::new(value.top_left, value.size)
    }
}

impl Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rectangle({}, {})", self.top_left, self.size)
    }
}

impl<'rect> IntoIterator for &'rect Rectangle {
    type Item = Xy;

    type IntoIter = CellIterator<'rect>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_cells()
    }
}

/// An iterator over the cells of a rectangle.
pub struct CellIterator<'rect> {
    next_cell: Xy,
    rectangle: &'rect Rectangle,
}

impl<'rect> Iterator for CellIterator<'rect> {
    type Item = Xy;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_cell.y >= self.rectangle.bottom_right.y {
            return None;
        }
        let cell = self.next_cell;
        self.next_cell.x += 1;
        if self.next_cell.x >= self.rectangle.bottom_right.x {
            self.next_cell.x = self.rectangle.top_left.x;
            self.next_cell.y += 1;
        }
        Some(cell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_iterator() {
        assert!(Rectangle::new((1, 3).into(), (0, 0).into())
            .iter_cells()
            .next()
            .is_none());
        assert_eq!(
            Rectangle::new((1, 3).into(), (2, 2).into())
                .iter_cells()
                .map(<(isize, isize)>::from)
                .collect::<Vec<_>>(),
            [(1, 3), (2, 3), (1, 4), (2, 4)],
        )
    }

    #[test]
    fn test_overlaps() {
        let base = Rectangle::new(Xy::new(5, 10), Xy::new(3, 7));

        assert!(base.overlaps(&Rectangle::new(Xy::new(4, 9), Xy::new(120, 42))));
        assert!(base.overlaps(&Rectangle::new(Xy::new(6, 8), Xy::new(1, 3))));
        assert!(!base.overlaps(&Rectangle::new(Xy::new(6, 8), Xy::new(1, 2))));
    }
}
