use crate::router::{cost::Cost, point::Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelOrientation {
    Horizontal,
    Vertical,
}

// A routing channel has a seed coordinate and a cost.
// the seed coordinate is the point that anchors the
// channel, which can then be vertical or horizontal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Channel {
    pub seed: Point,
    pub cost: Cost,
    pub orientation: ChannelOrientation,
}

pub fn h_channel(seed: impl Into<Point>, cost: impl Into<Cost>) -> Channel {
    Channel {
        seed: seed.into(),
        cost: cost.into(),
        orientation: ChannelOrientation::Horizontal,
    }
}

pub fn v_channel(seed: impl Into<Point>, cost: impl Into<Cost>) -> Channel {
    Channel {
        seed: seed.into(),
        cost: cost.into(),
        orientation: ChannelOrientation::Vertical,
    }
}
