const UNIT_SCALE: f64 = 16_777_216.0;

#[derive(Clone, Copy, PartialEq, Debug, PartialOrd, Ord, Eq)]
pub struct LinearDistance(i64);

impl From<f32> for LinearDistance {
    fn from(value: f32) -> Self {
        LinearDistance((value * UNIT_SCALE as f32) as i64)
    }
}

impl From<f64> for LinearDistance {
    fn from(value: f64) -> Self {
        LinearDistance((value * UNIT_SCALE) as i64)
    }
}

impl From<LinearDistance> for f32 {
    fn from(value: LinearDistance) -> Self {
        value.0 as f32 / UNIT_SCALE as f32
    }
}

impl std::ops::AddAssign<f32> for LinearDistance {
    fn add_assign(&mut self, rhs: f32) {
        *self = LinearDistance::from(f32::from(*self) + rhs);
    }
}

impl std::ops::Mul<f32> for LinearDistance {
    type Output = LinearDistance;

    fn mul(self, rhs: f32) -> Self::Output {
        LinearDistance::from(f32::from(self) * rhs)
    }
}

impl std::ops::Mul<f64> for LinearDistance {
    type Output = LinearDistance;

    fn mul(self, rhs: f64) -> Self::Output {
        LinearDistance::from(f32::from(self) * rhs as f32)
    }
}

impl std::ops::Sub<LinearDistance> for LinearDistance {
    type Output = LinearDistance;

    fn sub(self, rhs: LinearDistance) -> Self::Output {
        LinearDistance(self.0 - rhs.0)
    }
}
