use crate::router::coord::{CoordX, CoordY};

// A point on the grid is a pair of coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub x: CoordX,
    pub y: CoordY,
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl Point {
    pub fn manhattan_distance(self, other: Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

pub fn point(x: impl Into<CoordX>, y: impl Into<CoordY>) -> Point {
    Point {
        x: x.into(),
        y: y.into(),
    }
}

impl std::ops::Add<CoordX> for Point {
    type Output = Self;
    fn add(self, rhs: CoordX) -> Self {
        Point {
            x: self.x + rhs,
            y: self.y,
        }
    }
}

impl std::ops::Add<CoordY> for Point {
    type Output = Self;
    fn add(self, rhs: CoordY) -> Self {
        Point {
            x: self.x,
            y: self.y + rhs,
        }
    }
}

impl std::ops::Add<Point> for Point {
    type Output = Self;
    fn add(self, rhs: Point) -> Self {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

// Conversion from a Pos2 to a point cannot fail unless there is an
// overflow/underflow situation, which we do not handle.
impl From<egui::Pos2> for Point {
    fn from(pos: egui::Pos2) -> Self {
        Point {
            x: CoordX::from(pos.x),
            y: CoordY::from(pos.y),
        }
    }
}

// Round tripping will move any Pos2 to the nearest lattice point.
impl From<Point> for egui::Pos2 {
    fn from(point: Point) -> Self {
        egui::Pos2::new(
            (point.x.raw() as f32) * crate::grid::GRID_SIZE,
            (point.y.raw() as f32) * crate::grid::GRID_SIZE,
        )
    }
}
