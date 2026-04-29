use std::collections::BTreeSet;

use egui::{Pos2, Rect, Vec2, pos2, vec2};

use crate::{
    grid::{GRID_SIZE, LINE_RADIUS, ROUTE_TEXT_SIZE, SHIM, snap_to_grid},
    router::{TaggedPoint, point::Point},
    store::*,
    widget::{
        direction::RouteDirection, drawing::LineAnchor, edge::RouteEdge,
        linear_distance::LinearDistance, waypoint::Waypoint, wire_label::WireLabel,
    },
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AddTextButton {
    pub edge_id: EdgeId,
    pub linear_position: LinearDistance,
    pub pos: Pos2,
}

#[derive(PartialEq, Debug)]
pub struct AutoRoute {
    pub start: LineAnchor,
    pub edges: Store<EdgeId, RouteEdge>,
    pub finish: LineAnchor,
    pub start_pos: Pos2,
    pub end_pos: Pos2,
    pub waypoints: Store<WaypointId, Waypoint>,
    pub labels: Store<WireLabelId, WireLabel>,
    pub add_text_buttons: Vec<AddTextButton>,
}

pub struct LocAndDirection {
    pub location: Pos2,
    pub direction: RouteDirection,
}

impl LocAndDirection {
    pub fn perp_offset(&self, amount: f32) -> Pos2 {
        let perp: Vec2 = self.direction.into();
        self.location + perp * amount
    }
}

impl AutoRoute {
    fn edge_points_to_avoid(&self, edge_id: EdgeId) -> Vec<LinearDistance> {
        let edge = self.edge(edge_id).unwrap();
        let mut vec = Vec::new();
        if edge.start != self.start_pos {
            vec.push(self.distance_along_route(edge.start));
        }
        if edge.end != self.end_pos {
            vec.push(self.distance_along_route(edge.end));
        }
        vec.push(self.distance_along_route(edge.center()));
        vec
    }
    fn total_linear_distance(&self) -> LinearDistance {
        self.edges
            .iter()
            .map(|edge| edge.length())
            .sum::<f32>()
            .into()
    }
    // Convert a distance along the route to a point on the route.  This is the inverse
    // of distance_along_route.  If the distance is out of range, take the end anchor.
    pub fn map_linear_distance_to_position(
        &self,
        linear_distance: LinearDistance,
    ) -> LocAndDirection {
        let mut distance: f32 = linear_distance.into();
        for edge in &self.edges {
            if edge.length() < distance {
                distance -= edge.length();
            } else {
                let frac = distance / edge.length();
                return LocAndDirection {
                    location: edge.start + frac * (edge.end - edge.start),
                    direction: edge.direction(),
                };
            }
        }
        LocAndDirection {
            location: self.end_pos,
            direction: RouteDirection::Horizontal,
        }
    }
    // Calculate the distance along the route to reach point closest to the provided
    // position.
    fn distance_along_route(&self, pos: Pos2) -> LinearDistance {
        let mut distance = 0.0;
        for edge in &self.edges {
            let edge_distance = edge.distance(pos);
            if edge_distance <= LINE_RADIUS {
                // The point is close enough to this edge, so we calculate the distance along the route to this point.
                let start_to_pos = (pos - edge.start).length();
                let start_to_end = (edge.end - edge.start).length();
                if start_to_end > 0.0 {
                    distance += start_to_pos.min(start_to_end);
                }
                break;
            } else {
                // The point is not close to this edge, so we add the full length of this edge to the distance and continue.
                distance += edge.length();
            }
        }
        distance.into()
    }
    // Find the edge that corresponds to the provided LinearDistance
    pub fn find_edge(&self, linear_distance: LinearDistance) -> Option<&RouteEdge> {
        let mut distance: f32 = linear_distance.into();
        for edge in &self.edges {
            if edge.length() < distance {
                distance -= edge.length();
            } else {
                return Some(edge);
            }
        }
        self.edges.last()
    }
    pub fn alloc_wp(&mut self, pos: Pos2) -> WaypointId {
        self.waypoints.insert(Waypoint { pos, locked: false })
    }
    pub fn add_waypoint(&mut self, pos: Pos2) -> WaypointId {
        if let Some(wp) = self.hit_waypoint(pos, GRID_SIZE * 0.5) {
            return wp;
        }
        let wp = self.alloc_wp(pos);
        self.reorder_waypoints();
        wp
    }
    pub fn lock_waypoint(&mut self, id: WaypointId) {
        if let Some(wp) = self.waypoint_mut(id) {
            wp.locked = true;
        }
    }
    fn reorder_waypoints(&mut self) {
        let mut waypoints = std::mem::take(&mut self.waypoints);
        waypoints.sort_by_field(|wp| self.distance_along_route(wp.pos));
        self.waypoints = waypoints;
    }
    pub fn allocate_label(&mut self, pos: Pos2) -> WireLabelId {
        self.labels.insert(WireLabel {
            linear_distance: self.distance_along_route(pos),
            position: pos,
            text: String::new(),
        })
    }
    pub fn label(&self, label_id: WireLabelId) -> Option<&WireLabel> {
        self.labels.get(label_id)
    }
    pub fn label_mut(&mut self, label_id: WireLabelId) -> Option<&mut WireLabel> {
        self.labels.get_mut(label_id)
    }
    pub fn label_edit_details(&mut self, label_id: WireLabelId) -> Option<(Pos2, &mut WireLabel)> {
        let position = self.label(label_id)?.linear_distance;
        let position = self.map_linear_distance_to_position(position).location;
        let label = self.label_mut(label_id)?;
        Some((position, label))
    }
    pub fn waypoint(&self, waypoint_id: WaypointId) -> Option<&Waypoint> {
        self.waypoints.get(waypoint_id)
    }
    pub fn waypoint_mut(&mut self, waypoint_id: WaypointId) -> Option<&mut Waypoint> {
        self.waypoints.get_mut(waypoint_id)
    }
    pub fn hit_waypoint(&self, pos: Pos2, tolerance: f32) -> Option<WaypointId> {
        self.waypoints.iter().find_map(|(wp_id, wp)| {
            if wp.pos.distance(pos) <= tolerance {
                Some(wp_id)
            } else {
                None
            }
        })
    }
    pub fn drag_handles(&self) -> Vec<Pos2> {
        self.edges.values().map(|edge| edge.center()).collect()
    }
    pub fn points(&self) -> Vec<Pos2> {
        // We do not want the start and end points duplicated for internal edges.
        let mut points = Vec::new();
        points.push(self.start_pos);
        for edge in &self.edges {
            points.push(edge.end);
        }
        points
    }
    pub fn edge(&self, edge_index: EdgeId) -> Option<&RouteEdge> {
        self.edges.get(edge_index)
    }
    pub fn edge_mut(&mut self, edge_index: EdgeId) -> Option<&mut RouteEdge> {
        self.edges.get_mut(edge_index)
    }
    pub fn is_z_bend(&self, edge_index: EdgeId) -> bool {
        if let Some(ndx) = self.edges.iter().position(|edge| edge.id == edge_index)
            && ndx > 0
            && ndx < self.edges.len() - 1
        {
            return self.edges[ndx - 1].direction() == self.edges[ndx + 1].direction()
                && self.edges[ndx].direction() != self.edges[ndx - 1].direction();
        }
        false
    }
    pub fn move_edge(&mut self, edge_index: EdgeId, delta: Vec2) {
        if let Some(ndx) = self.edges.iter().position(|edge| edge.id == edge_index)
            && self.is_z_bend(edge_index)
        {
            // If this is the vertical segment of a horizontal z-bend, then
            // we calculate the new vertical position of the bend, and update
            // the start and end of the adjacent edges accordingly.
            let (start, end) = {
                let edge = &mut self.edges[ndx];
                let start = edge.start;
                match edge.direction() {
                    RouteDirection::Horizontal => {
                        let delta = vec2(0.0, delta.y);
                        let new_start: Pos2 = start + delta;
                        edge.start.y = new_start.y;
                        edge.end.y = new_start.y;
                    }
                    RouteDirection::Vertical => {
                        let delta = vec2(delta.x, 0.0);
                        let new_start: Pos2 = start + delta;
                        edge.start.x = new_start.x;
                        edge.end.x = new_start.x;
                    }
                }
                (edge.start, edge.end)
            };
            self.edges[ndx - 1].end = start;
            self.edges[ndx + 1].start = end;
        }
    }
    pub fn finish_drag(&mut self, edge_index: EdgeId) {
        if !self.is_z_bend(edge_index) {
            return;
        }
        self.edges.iter_mut().for_each(|edge| {
            edge.start = snap_to_grid(edge.start);
            edge.end = snap_to_grid(edge.end);
        });
        // Drop all waypoints that are no longer on the path.
        self.update_waypoints();
    }
    pub fn update_label_positions(&mut self) {
        let mut labels = std::mem::take(&mut self.labels);
        for label in &mut labels {
            label.position = self
                .map_linear_distance_to_position(label.linear_distance)
                .location;
        }
        self.labels = labels;
    }
    fn make_add_text_button(&self, linear_distance: LinearDistance) -> AddTextButton {
        let pos_and_direction = self.map_linear_distance_to_position(linear_distance);
        let pos = pos_and_direction.perp_offset(SHIM);
        let edge = self.find_edge(linear_distance).unwrap();
        AddTextButton {
            edge_id: edge.id,
            linear_position: linear_distance,
            pos,
        }
    }
    pub fn update_waypoints(&mut self) {
        let mut labels = std::mem::take(&mut self.labels);
        for label in &mut labels {
            let desired_position = label.position;
            if let Some(edge) = self.find_edge(label.linear_distance) {
                let edge_start = edge.start;
                let edge_end = edge.end;
                let edge_unit = (edge_end - edge_start).normalized();
                let edge_signed_length = (edge.end - edge.start).dot(edge_unit);
                let edge_projection =
                    ((desired_position - edge_start).dot(edge_unit)).clamp(0.0, edge_signed_length);
                let new_position = edge_start + edge_unit * edge_projection;
                let new_linear_distance = self.distance_along_route(new_position);
                label.linear_distance = new_linear_distance;
                label.position = self
                    .map_linear_distance_to_position(new_linear_distance)
                    .location;
            }
        }
        labels.retain(|l| !l.text.is_empty());
        self.labels = labels;
        let total_path_length = self.total_linear_distance();
        // Try to find an empty space.
        let occupied = self
            .labels
            .iter()
            .map(|label| label.linear_distance)
            .chain(
                self.edges
                    .iter()
                    .flat_map(|edge| self.edge_points_to_avoid(edge.id).into_iter()),
            )
            .map(|ld| (f32::from(ld) / (5.0 * GRID_SIZE)) as i64)
            .collect::<BTreeSet<i64>>();
        eprintln!("Occupied {:?}", occupied);
        let num_slots = (f32::from(total_path_length) / (5.0 * GRID_SIZE)) as i64;
        // Find the first empty space that is not occupied
        let first_empty_slot = (0..num_slots)
            .find(|ndx| !occupied.contains(ndx))
            .map(|ndx| LinearDistance::from(5.0 * GRID_SIZE * (ndx as f32 + 0.5)))
            .map(|ld| self.make_add_text_button(ld));
        let last_empty_slot = (0..num_slots)
            .rev()
            .find(|ndx| !occupied.contains(ndx))
            .map(|ndx| LinearDistance::from(5.0 * GRID_SIZE * (ndx as f32 + 0.5)))
            .map(|ld| self.make_add_text_button(ld));
        self.add_text_buttons = first_empty_slot
            .into_iter()
            .chain(last_empty_slot)
            .collect();
    }
    pub fn grid_points(&self) -> Vec<Point> {
        self.points().into_iter().map(|pos| pos.into()).collect()
    }
    pub fn hovered_corner(&self, hover_pos: Pos2) -> Option<(EdgeId, EdgeId)> {
        self.edges.windows(2).find_map(|edges| {
            let edge1 = &edges[0];
            let edge2 = &edges[1];
            if edge1.end.distance(hover_pos) <= LINE_RADIUS
                && edge1.direction() != edge2.direction()
            {
                Some((edge1.id, edge2.id))
            } else {
                None
            }
        })
    }
    pub fn hovered_edge(&self, hover_pos: Pos2) -> Option<EdgeId> {
        self.edges.iter().find_map(|edge| {
            if edge.distance(hover_pos) <= LINE_RADIUS
                && edge.start.distance(hover_pos) > LINE_RADIUS
                && edge.end.distance(hover_pos) > LINE_RADIUS
            {
                Some(edge.id)
            } else {
                None
            }
        })
    }
    pub fn build(
        start: LineAnchor,
        finish: LineAnchor,
        points: &[TaggedPoint],
        waypoints: &[Waypoint],
        labels: &[WireLabel],
    ) -> Self {
        // Scan through the set of points, and create a set of edges.
        // Each edge should be either horizontal or vertical,
        // and should continue as long as possible until the direction changes.
        let mut edges = Vec::new();
        let mut edge_id = 0;
        for windows in points.windows(2) {
            let start = windows[0];
            let end = windows[1];
            if start.segment != end.segment {
                continue;
            }
            edges.push(RouteEdge {
                id: edge_id.into(),
                start: start.pos.into(),
                end: end.pos.into(),
                kind: start.segment,
            });
            edge_id += 1;
        }
        let start_pos = if let Some(point) = points.first() {
            point.pos.into()
        } else {
            pos2(0.0, 0.0)
        };
        let end_pos = if let Some(point) = points.last() {
            point.pos.into()
        } else {
            pos2(0.0, 0.0)
        };
        // Scan through the edges, and as long as they have the same kind
        // and the same direction, merge them into a single edge.
        let mut merged_edges = Vec::new();
        let mut current_edge: Option<RouteEdge> = None;
        for edge in edges {
            if let Some(current) = &mut current_edge {
                if current.kind == edge.kind && current.direction() == edge.direction() {
                    current.end = edge.end;
                } else {
                    merged_edges.push(current.clone());
                    current_edge = Some(edge);
                }
            } else {
                current_edge = Some(edge);
            }
        }
        if let Some(current) = current_edge {
            merged_edges.push(current);
        }
        Self {
            start,
            edges: merged_edges,
            finish,
            start_pos,
            end_pos,
            waypoints: waypoints.to_vec(),
            labels: labels.to_vec(),
            add_text_buttons: Vec::new(),
        }
    }
    pub fn hit_text_anchor(&self, hover_pos: Pos2) -> Option<WireLabelId> {
        self.labels.iter().find_map(|label| {
            let pos_and_direction = self.map_linear_distance_to_position(label.linear_distance);
            let center_of_label = pos_and_direction.location
                + match pos_and_direction.direction {
                    RouteDirection::Horizontal => vec2(0.0, -ROUTE_TEXT_SIZE * 0.5),
                    RouteDirection::Vertical => vec2(ROUTE_TEXT_SIZE * 0.5, 0.0),
                };
            let label_size = match pos_and_direction.direction {
                RouteDirection::Horizontal => {
                    vec2(ROUTE_TEXT_SIZE * label.text.len() as f32, ROUTE_TEXT_SIZE)
                }
                RouteDirection::Vertical => {
                    vec2(ROUTE_TEXT_SIZE, ROUTE_TEXT_SIZE * label.text.len() as f32)
                }
            };
            let label_bb = Rect::from_center_size(center_of_label, label_size);
            if label_bb.contains(hover_pos) {
                return Some(label.id);
            }
            None
        })
    }
    pub fn text_anchors(&self) -> Vec<Pos2> {
        self.labels
            .values()
            .map(|label| {
                self.map_linear_distance_to_position(label.linear_distance)
                    .location
            })
            .collect()
    }
    pub fn all_add_text_buttons(&self) -> Vec<AddTextButton> {
        self.add_text_buttons.clone()
    }
    pub fn hit_add_text_button(&self, pos: Pos2) -> Option<&AddTextButton> {
        self.add_text_buttons
            .iter()
            .find(|button| button.pos.distance(pos) <= GRID_SIZE * 0.5)
    }
}
