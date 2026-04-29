// This is a lattice point on the grid.  The grid is double ended
// and the coordinates can be negative, so we use a signed integer.
// We use two newtype wrappers to handle the two axes.
macro_rules! define_coord {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(i32);

        impl $name {
            pub fn min(self, other: Self) -> Self {
                $name(self.0.min(other.0))
            }

            pub fn max(self, other: Self) -> Self {
                $name(self.0.max(other.0))
            }

            pub fn abs(self) -> i32 {
                self.0.abs()
            }

            pub fn raw(self) -> i32 {
                self.0
            }
        }

        impl From<f32> for $name {
            fn from(value: f32) -> Self {
                $name((value / crate::grid::GRID_SIZE).round() as i32)
            }
        }

        impl From<i32> for $name {
            fn from(value: i32) -> Self {
                $name(value)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::ops::Add for $name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                $name(self.0 + rhs.0)
            }
        }

        impl std::ops::Add<i32> for $name {
            type Output = Self;
            fn add(self, rhs: i32) -> Self {
                $name(self.0 + rhs)
            }
        }

        impl std::ops::Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                $name(self.0 - rhs.0)
            }
        }

        impl std::ops::Sub<i32> for $name {
            type Output = Self;
            fn sub(self, rhs: i32) -> Self {
                $name(self.0 - rhs)
            }
        }
    };
}

define_coord!(CoordX);
define_coord!(CoordY);

pub const INFINITY_X: CoordX = CoordX(i32::MAX >> 4);
pub const INFINITY_Y: CoordY = CoordY(i32::MAX >> 4);
pub const NEG_INFINITY_X: CoordX = CoordX(i32::MIN >> 4);
pub const NEG_INFINITY_Y: CoordY = CoordY(i32::MIN >> 4);
