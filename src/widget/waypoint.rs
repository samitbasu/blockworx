use crate::router::point::Point;
use egui::Pos2;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Waypoint {
    pub pos: Pos2,
    pub locked: bool,
}

impl Waypoint {
    pub fn unlock(&mut self) {
        self.locked = false;
    }
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

impl From<Waypoint> for Point {
    fn from(val: Waypoint) -> Self {
        val.pos.into()
    }
}
