use egui::{Pos2, pos2};

use crate::{
    grid::{GRID_SIZE, MIN_TEXT_EDGE_LENGTH, SHIM},
    widget::{direction::RouteDirection, segmentkind::SegmentKind},
};

#[derive(Clone, PartialEq, Debug)]
pub struct RouteEdge {
    pub start: Pos2,
    pub end: Pos2,
    pub kind: SegmentKind,
}

impl RouteEdge {
    pub fn distance(&self, pos: Pos2) -> f32 {
        let start = self.start;
        let end = self.end;
        let line_vec = end - start;
        let line_len = line_vec.length();
        if line_len == 0.0 {
            return (pos - start).length();
        }
        let t = ((pos - start).dot(line_vec) / line_len.powi(2)).clamp(0.0, 1.0);
        let projection = start + line_vec * t;
        (pos - projection).length()
    }
    pub fn direction(&self) -> RouteDirection {
        if (self.end.x - self.start.x).abs() > (self.end.y - self.start.y).abs() {
            RouteDirection::Horizontal
        } else {
            RouteDirection::Vertical
        }
    }
    pub fn length(&self) -> f32 {
        (self.end - self.start).length()
    }
    pub fn text_anchor(&self) -> Option<Pos2> {
        if self.direction() == RouteDirection::Horizontal && self.length() >= MIN_TEXT_EDGE_LENGTH {
            Some(pos2((self.start.x + self.end.x) / 2.0, self.start.y - SHIM))
        } else {
            None
        }
    }
    pub fn center(&self) -> Pos2 {
        self.start + (self.end - self.start) * 0.5
    }
    pub fn waypoint_position_start(&self) -> Pos2 {
        // Avoid the routing gutters.
        let dir = (self.end - self.start).normalized();
        match self.kind {
            SegmentKind::WaypointToWaypoint(_, _) => self.start,
            SegmentKind::StartToWaypoint(_) | SegmentKind::StartToEnd => {
                self.start + dir * GRID_SIZE
            }
            SegmentKind::WaypointToEnd(_) => self.start,
        }
    }
    pub fn waypoint_position_end(&self) -> Pos2 {
        // Avoid the routing gutters.
        let dir = (self.end - self.start).normalized();
        match self.kind {
            SegmentKind::WaypointToWaypoint(_, _) => self.end,
            SegmentKind::StartToWaypoint(_) => self.end,
            SegmentKind::WaypointToEnd(_) | SegmentKind::StartToEnd => self.end - dir * GRID_SIZE,
        }
    }
}
