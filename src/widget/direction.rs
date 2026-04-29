use egui::{Vec2, vec2};

#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug, PartialOrd, Ord)]
pub enum RouteDirection {
    Horizontal,
    Vertical,
}

impl From<RouteDirection> for Vec2 {
    fn from(value: RouteDirection) -> Self {
        match value {
            RouteDirection::Horizontal => vec2(0.0, -1.0),
            RouteDirection::Vertical => vec2(1.0, 0.0),
        }
    }
}
