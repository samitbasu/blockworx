#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub struct Cost(i64);

impl pathfinding::num_traits::Zero for Cost {
    fn zero() -> Self {
        COST_ZERO
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

const UNIT_SCALE: f64 = 16_777_216.0; // 2^24

impl From<Cost> for f64 {
    fn from(value: Cost) -> Self {
        value.0 as f64 / UNIT_SCALE
    }
}

impl From<f64> for Cost {
    fn from(value: f64) -> Self {
        Self((value * UNIT_SCALE) as i64)
    }
}

impl From<f32> for Cost {
    fn from(value: f32) -> Self {
        Self((value as f64 * UNIT_SCALE) as i64)
    }
}

impl Cost {
    pub const fn new(cost: f64) -> Self {
        Self((cost * UNIT_SCALE) as i64)
    }
}

impl std::ops::AddAssign<Cost> for Cost {
    fn add_assign(&mut self, rhs: Cost) {
        self.0 += rhs.0;
    }
}

impl std::ops::SubAssign<Cost> for Cost {
    fn sub_assign(&mut self, rhs: Cost) {
        self.0 -= rhs.0;
    }
}

impl std::ops::Add<Cost> for Cost {
    type Output = Self;

    fn add(self, rhs: Cost) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub<Cost> for Cost {
    type Output = Self;

    fn sub(self, rhs: Cost) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Mul<Cost> for i64 {
    type Output = Cost;

    fn mul(self, rhs: Cost) -> Self::Output {
        Cost(self * rhs.0)
    }
}

impl std::ops::Mul<f64> for Cost {
    type Output = Cost;

    fn mul(self, rhs: f64) -> Self::Output {
        Cost((self.0 as f64 * rhs) as i64)
    }
}

pub const COST_ZERO: Cost = Cost(0);

impl std::fmt::Display for Cost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0 as f64 / UNIT_SCALE)
    }
}
