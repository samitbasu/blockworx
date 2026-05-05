use std::collections::BTreeSet;

use egui::{Pos2, Rect, Vec2, pos2, vec2};

use crate::{
    grid::{GRID_SIZE, LINE_RADIUS, ROUTE_TEXT_SIZE, SHIM},
    router::{RouterNG, TaggedPoint, WIRE_COST, point::Point},
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
    start: LineAnchor,
    edges: Store<EdgeId, RouteEdge>,
    finish: LineAnchor,
    start_pos: Pos2,
    end_pos: Pos2,
    waypoints: Store<WaypointId, Waypoint>,
    labels: Store<WireLabelId, WireLabel>,
    add_text_buttons: Vec<AddTextButton>,
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
    pub fn rip_and_reroute(&mut self, start: Pos2, end: Pos2, router: &mut RouterNG) {
        self.waypoints.iter_mut().for_each(|(_, wp)| wp.unlock());
        self.waypoints.retain(|wp| router.is_accessible(wp.pos));
        // TODO - remove redundant waypoints.
        let path = router.waypoint_path(start, &self.waypoints, end);
        let waypoints = std::mem::take(&mut self.waypoints);
        let labels = std::mem::take(&mut self.labels);
        let mut route = Self::build(self.start, self.finish, &path, waypoints, labels);
        route.update_waypoints();
        route.start_pos = start;
        route.end_pos = end;
        router.add_existing_route(route.edges.iter().map(|(_, edge)| edge), WIRE_COST);
        *self = route;
    }
    pub fn start_pos(&self) -> Pos2 {
        self.start_pos
    }
    pub fn end_pos(&self) -> Pos2 {
        self.end_pos
    }
    pub fn start(&self) -> LineAnchor {
        self.start
    }
    pub fn finish(&self) -> LineAnchor {
        self.finish
    }
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
            .map(|(_, edge)| edge.length())
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
        for (_, edge) in self.iter_edges() {
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
        for (_, edge) in self.iter_edges() {
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
    pub fn find_edge(&self, linear_distance: LinearDistance) -> Option<(EdgeId, &RouteEdge)> {
        let mut distance: f32 = linear_distance.into();
        for (id, edge) in self.iter_edges() {
            if edge.length() < distance {
                distance -= edge.length();
            } else {
                return Some((id, edge));
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
    pub fn iter_labels(&self) -> impl Iterator<Item = (WireLabelId, &WireLabel)> {
        self.labels.iter()
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
    pub fn update_waypoint(&mut self, waypoint_id: WaypointId, update: impl FnOnce(&mut Waypoint)) {
        if let Some(wp) = self.waypoint_mut(waypoint_id) {
            update(wp);
        }
    }
    pub fn iter_waypoints(&self) -> impl Iterator<Item = (WaypointId, &Waypoint)> {
        self.waypoints.iter()
    }
    pub fn iter_waypoints_mut(&mut self) -> impl Iterator<Item = (WaypointId, &mut Waypoint)> {
        self.waypoints.iter_mut()
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
        for (_, edge) in self.iter_edges() {
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
    pub fn iter_edges(&self) -> impl Iterator<Item = (EdgeId, &RouteEdge)> {
        self.edges.iter()
    }
    pub fn iter_edges_mut(&mut self) -> impl Iterator<Item = (EdgeId, &mut RouteEdge)> {
        self.edges.iter_mut()
    }
    pub fn update_label_positions(&mut self) {
        let mut labels = std::mem::take(&mut self.labels);
        labels.iter_mut().for_each(|(_, label)| {
            label.position = self
                .map_linear_distance_to_position(label.linear_distance)
                .location;
        });
        self.labels = labels;
    }
    fn make_add_text_button(&self, linear_distance: LinearDistance) -> Option<AddTextButton> {
        let pos_and_direction = self.map_linear_distance_to_position(linear_distance);
        let pos = pos_and_direction.perp_offset(SHIM);
        let (id, _) = self.find_edge(linear_distance)?;
        Some(AddTextButton {
            edge_id: id,
            linear_position: linear_distance,
            pos,
        })
    }
    pub fn update_waypoints(&mut self) {
        let mut labels = std::mem::take(&mut self.labels);
        for (_, label) in labels.iter_mut() {
            let desired_position = label.position;
            if let Some((_, edge)) = self.find_edge(label.linear_distance) {
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
            .map(|(_, label)| label.linear_distance)
            .chain(
                self.edges
                    .iter()
                    .flat_map(|(eid, _)| self.edge_points_to_avoid(eid).into_iter()),
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
            .flatten()
            .collect();
    }
    pub fn grid_points(&self) -> Vec<Point> {
        self.points().into_iter().map(|pos| pos.into()).collect()
    }
    pub fn hovered_corner(&self, hover_pos: Pos2) -> Option<(EdgeId, EdgeId)> {
        self.edges.windows(2).find_map(|edges| {
            let (edge_id1, edge1) = edges[0];
            let (edge_id2, edge2) = edges[1];
            if edge1.end.distance(hover_pos) <= LINE_RADIUS
                && edge1.direction() != edge2.direction()
            {
                Some((edge_id1, edge_id2))
            } else {
                None
            }
        })
    }
    pub fn hovered_edge(&self, hover_pos: Pos2) -> Option<EdgeId> {
        self.edges.iter().find_map(|(eid, edge)| {
            if edge.distance(hover_pos) <= LINE_RADIUS
                && edge.start.distance(hover_pos) > LINE_RADIUS
                && edge.end.distance(hover_pos) > LINE_RADIUS
            {
                Some(eid)
            } else {
                None
            }
        })
    }
    pub fn build(
        start: LineAnchor,
        finish: LineAnchor,
        points: &[TaggedPoint],
        waypoints: Store<WaypointId, Waypoint>,
        labels: Store<WireLabelId, WireLabel>,
    ) -> Self {
        // Scan through the set of points, and create a set of edges.
        // Each edge should be either horizontal or vertical,
        // and should continue as long as possible until the direction changes.
        let mut edges = Vec::new();
        for windows in points.windows(2) {
            let start = windows[0];
            let end = windows[1];
            if start.segment != end.segment {
                continue;
            }
            edges.push(RouteEdge {
                start: start.pos.into(),
                end: end.pos.into(),
                kind: start.segment,
            });
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
        let mut merged_edges = Store::default();
        let mut current_edge: Option<RouteEdge> = None;
        for edge in edges {
            if let Some(current) = &mut current_edge {
                if current.kind == edge.kind && current.direction() == edge.direction() {
                    current.end = edge.end;
                } else {
                    merged_edges.insert(current.clone());
                    current_edge = Some(edge);
                }
            } else {
                current_edge = Some(edge);
            }
        }
        if let Some(current) = current_edge {
            merged_edges.insert(current);
        }
        Self {
            start,
            edges: merged_edges,
            finish,
            start_pos,
            end_pos,
            waypoints: waypoints,
            labels: labels,
            add_text_buttons: Vec::new(),
        }
    }
    pub fn hit_text_anchor(&self, hover_pos: Pos2) -> Option<WireLabelId> {
        self.labels.iter().find_map(|(lid, label)| {
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
                return Some(lid);
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
