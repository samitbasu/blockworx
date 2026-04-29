use crate::state::WaypointId;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SegmentKind {
    StartToWaypoint(WaypointId),
    WaypointToWaypoint(WaypointId, WaypointId),
    WaypointToEnd(WaypointId),
    StartToEnd,
}

impl SegmentKind {
    pub fn is_wp_to_wp(&self) -> bool {
        matches!(self, SegmentKind::WaypointToWaypoint(_, _))
    }
}
