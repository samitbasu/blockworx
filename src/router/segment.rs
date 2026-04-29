use crate::router::{
    coord::{CoordX, CoordY},
    cost::Cost,
};

// A linear segment is a start and end coordinate and a cost.
// A segment is either horizontal or vertical, and the cost is
// the cost of routing through that segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Segment<P> {
    pub start: P,
    pub end: P,
    pub cost: Cost,
}

pub type HSegment = Segment<CoordX>;
pub type VSegment = Segment<CoordY>;

pub fn hseg(start: impl Into<CoordX>, end: impl Into<CoordX>, cost: impl Into<Cost>) -> HSegment {
    HSegment {
        start: start.into(),
        end: end.into(),
        cost: cost.into(),
    }
}

pub fn vseg(start: impl Into<CoordY>, end: impl Into<CoordY>, cost: impl Into<Cost>) -> VSegment {
    VSegment {
        start: start.into(),
        end: end.into(),
        cost: cost.into(),
    }
}
