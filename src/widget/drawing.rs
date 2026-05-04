use egui::{
    Align2, Color32, PointerButton, Pos2, Rect, Response, Stroke, StrokeKind, TextEdit, Ui,
    epaint::TextShape, pos2, vec2,
};

use crate::{
    grid::{
        GRID_SIZE, MOVE_HOVER_DISTANCE, PORT_HEIGHT, PORT_RADIUS, ROUTE_TEXT_SIZE, SHIM, grid_rect,
        round_to_grid, snap_to_grid,
    },
    render::{
        FocusResult, estimate_bbox_for_pin_text, get_control_pin_bbox, get_hamburger_rect,
        render_path_with_chamfered_corners, render_rect_box,
    },
    router::{RouterNG, RouterNGBuilder, TaggedPoint, WIRE_COST, cost::COST_ZERO},
    state::*,
    store::*,
    widget::{
        auto_route::AutoRoute,
        direction::RouteDirection,
        pin::{BoxKind, PinSide},
        rect_box::{RectBox, control_corner, resize_rect},
        waypoint::Waypoint,
    },
};

const GRIP_SHIM: f32 = 4.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LineAnchor {
    pub rect: RectId,
    pub pin: PinId,
}

#[derive(Default)]
pub struct Drawing {
    rect_boxes: Store<RectId, RectBox>,
    auto_routes: Store<RouteId, AutoRoute>,
    state: State,
    auto_route: Vec<TaggedPoint>,
    reroute: bool,
    ripup_set: Vec<RouteId>,
}

enum RouteRenderMode {
    Normal,
    Highlighted,
    Selected,
}

impl Drawing {
    pub fn rect(&self, id: RectId) -> Option<&RectBox> {
        self.rect_boxes.get(id)
    }
    pub fn rect_mut(&mut self, id: RectId) -> Option<&mut RectBox> {
        self.rect_boxes.get_mut(id)
    }
    pub fn add_rect_box(&mut self, start: Pos2, end: Pos2) -> RectId {
        self.rect_boxes.insert(RectBox::new(
            "Untitled".to_string(),
            Rect::from_two_pos(start, end),
        ))
    }
    pub fn add_port_box(&mut self, pin_name: String, side: PinSide, inner: Rect) -> RectId {
        self.rect_boxes
            .insert(RectBox::new_port(pin_name, side, inner))
    }
    pub fn routing_box(&self, id: RectId) -> Option<Rect> {
        let rect = self.rect(id)?.gui_rect();
        if let State::MovingRect(inner) = &self.state
            && inner.rect == id
        {
            Some(grid_rect(rect.translate(inner.delta_pos)))
        } else if let State::ResizingRect(inner) = &self.state
            && inner.rect == id
        {
            Some(grid_rect(resize_rect(&rect, inner.mode, inner.delta_pos)))
        } else {
            Some(rect)
        }
    }
    pub fn anchor(&self, anchor: LineAnchor) -> Option<Pos2> {
        let effective_rect = self.routing_box(anchor.rect)?;
        if let State::PinDragged(inner) = &self.state
            && anchor.rect == inner.rect
            && anchor.pin == inner.pin
        {
            let center_line = effective_rect.center().x;
            let rect = self.rect(anchor.rect)?;
            let pin_pos = rect.anchor_point(anchor.pin)?;
            let current_pos = pin_pos + inner.delta_pos;
            let anchor_x = if current_pos.x < center_line {
                effective_rect.left() - GRID_SIZE
            } else {
                effective_rect.right() + GRID_SIZE
            };
            let anchor_y = round_to_grid(current_pos.y);
            Some(snap_to_grid(pos2(anchor_x, anchor_y)))
        } else {
            self.rect(anchor.rect)
                .and_then(|rect| rect.anchor_point_with_rect(effective_rect, anchor.pin))
        }
    }
    pub fn iter_anchors(&self) -> impl Iterator<Item = LineAnchor> + '_ {
        self.rect_boxes.iter().flat_map(|(rect_id, rect)| {
            rect.iter_pins().map(move |(pin_id, _)| LineAnchor {
                rect: rect_id,
                pin: pin_id,
            })
        })
    }
    pub fn iter_anchor_positions(&self) -> impl Iterator<Item = (LineAnchor, Pos2)> + '_ {
        self.iter_anchors()
            .filter_map(|anchor| self.anchor(anchor).map(|pos| (anchor, pos)))
    }
    fn render_route(&self, ui: &mut Ui, route: &AutoRoute, mode: RouteRenderMode) {
        let route_stroke = match mode {
            RouteRenderMode::Normal => (1.7, Color32::DARK_GREEN),
            RouteRenderMode::Highlighted => (2.5, Color32::LIGHT_GREEN.gamma_multiply(0.3)),
            RouteRenderMode::Selected => (2.5, Color32::LIGHT_GREEN),
        };
        let points = render_path_with_chamfered_corners(&route.points());
        points.render(ui, route_stroke);
        let text_color = match mode {
            RouteRenderMode::Normal => Color32::DARK_GREEN,
            RouteRenderMode::Highlighted => Color32::LIGHT_GREEN.gamma_multiply(0.3),
            RouteRenderMode::Selected => Color32::LIGHT_GREEN,
        };
        for (_, label) in route.iter_labels() {
            let loc_and_direction = route.map_linear_distance_to_position(label.linear_distance);
            let pos = loc_and_direction.location;
            match loc_and_direction.direction {
                RouteDirection::Horizontal => {
                    ui.painter().text(
                        pos + vec2(0.0, -SHIM / 4.0),
                        egui::Align2::CENTER_BOTTOM,
                        &label.text,
                        egui::FontId::monospace(ROUTE_TEXT_SIZE),
                        text_color,
                    );
                }
                RouteDirection::Vertical => {
                    // Rotate the text by 90 degrees
                    // TODO - save the galley for reuse
                    let galley = ui.ctx().fonts_mut(|fv| {
                        fv.layout_no_wrap(
                            label.text.clone(),
                            egui::FontId::monospace(ROUTE_TEXT_SIZE),
                            text_color,
                        )
                    });
                    let mut text = TextShape::new(pos, galley, Color32::WHITE)
                        .with_angle_and_anchor(std::f32::consts::FRAC_PI_2, Align2::LEFT_BOTTOM);
                    let text_rect = text.visual_bounding_rect();
                    let v_delta = text_rect.center().y - pos.y;
                    text.pos = text.pos - vec2(0.0, v_delta);
                    ui.painter().add(text);
                }
            }
        }
        if matches!(mode, RouteRenderMode::Selected) {
            for (_, wp) in route.iter_waypoints() {
                ui.painter().circle(
                    wp.pos,
                    PORT_RADIUS,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (0.5, Color32::BLACK),
                );
            }
            for dh in route.drag_handles() {
                ui.painter().rect(
                    Rect::from_center_size(dh, vec2(PORT_RADIUS * 2.0, PORT_RADIUS * 2.0)),
                    PORT_RADIUS / 4.0,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (0.5, Color32::BLACK),
                    StrokeKind::Middle,
                );
            }
            for ta in route.text_anchors() {
                // Use a triangle for the text anchor
                Self::draw_text_anchor(
                    ui,
                    ta,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (0.5, Color32::BLACK),
                );
            }
            for at in route.all_add_text_buttons() {
                Self::draw_add_text_button(
                    ui,
                    at.pos,
                    Color32::LIGHT_YELLOW.linear_multiply(0.5),
                    (0.5, Color32::BLACK),
                );
            }
        }
    }
    fn draw_text_anchor(ui: &mut Ui, ta: Pos2, fill: Color32, stroke: impl Into<Stroke>) {
        let stroke: Stroke = stroke.into();
        // Use a diamond for the text anchor
        ui.painter().add(egui::Shape::convex_polygon(
            [
                ta + vec2(0.0, -PORT_RADIUS),
                ta + vec2(PORT_RADIUS, 0.0),
                ta + vec2(0.0, PORT_RADIUS),
                ta + vec2(-PORT_RADIUS, 0.0),
            ]
            .into(),
            fill,
            stroke,
        ));
    }
    fn draw_add_text_button(ui: &mut Ui, at: Pos2, fill: Color32, stroke: impl Into<Stroke>) {
        let stroke: Stroke = stroke.into();
        ui.painter().rect(
            Rect::from_center_size(at, vec2(PORT_RADIUS * 2.0, PORT_RADIUS * 2.0)),
            PORT_RADIUS / 4.0,
            fill,
            stroke,
            StrokeKind::Middle,
        );
        ui.painter().text(
            at,
            Align2::CENTER_CENTER,
            "T",
            egui::FontId::monospace(ROUTE_TEXT_SIZE * 0.8),
            Color32::LIGHT_YELLOW,
        );
    }
    pub fn render(&mut self, ui: &mut Ui) {
        ui.output_mut(|o| o.cursor_icon = self.state.cursor());
        (-100..=100).map(|y| y as f32 * GRID_SIZE).for_each(|h| {
            ui.painter().hline(
                -10_000.0f32..=10_000.0f32,
                h,
                (0.15, Color32::LIGHT_GRAY.linear_multiply(0.3)),
            );
        });
        (-100..=100).map(|x| x as f32 * GRID_SIZE).for_each(|v| {
            ui.painter().vline(
                v,
                -10_000.0f32..=10_000.0f32,
                (0.15, Color32::LIGHT_GRAY.linear_multiply(0.3)),
            );
        });
        for route in self.auto_routes.values() {
            self.render_route(ui, &route, RouteRenderMode::Normal);
        }
        for (id, rect_box) in self.rect_boxes.iter_mut() {
            if render_rect_box(id, rect_box, &self.state, ui) == FocusResult::LostFocus {
                self.state = Selected { rect: id }.into();
            }
        }
        if let State::AddingRect(AddingRect { start_pos, end_pos }) = &self.state {
            let rect = Rect::from_two_pos(*start_pos, *end_pos);
            ui.painter().rect(
                rect,
                3.0,
                Color32::TRANSPARENT,
                (1.0, Color32::DARK_RED),
                StrokeKind::Middle,
            );
        }
        if let State::InProgressAutoRoute(inner) = &self.state {
            let points = self
                .auto_route
                .iter()
                .map(|p| p.pos.into())
                .collect::<Vec<Pos2>>();
            let points = render_path_with_chamfered_corners(&points);
            points.render(ui, (0.5, Color32::LIGHT_YELLOW));
            inner.waypoints.iter().for_each(|(_, wp)| {
                ui.painter()
                    .circle_filled(wp.pos, PORT_RADIUS, Color32::LIGHT_YELLOW);
            });
        }
        if let State::ProposedAutoRoute(inner) = &self.state {
            let points = self
                .auto_route
                .iter()
                .map(|p| p.pos.into())
                .collect::<Vec<Pos2>>();
            let points = render_path_with_chamfered_corners(&points);
            points.render(ui, (1.5, Color32::LIGHT_YELLOW));
            if let Some(start_pos) = self.anchor(inner.start) {
                ui.painter().circle(
                    start_pos,
                    PORT_RADIUS,
                    Color32::DARK_RED,
                    (0.5, Color32::DARK_RED),
                );
            }
            if let Some(end_pos) = self.anchor(inner.finish) {
                ui.painter().circle(
                    end_pos,
                    PORT_RADIUS,
                    Color32::DARK_RED,
                    (0.5, Color32::DARK_RED),
                );
            }
        }
        if let State::RouteHovered(target) = &self.state
            && let Some(route) = self.auto_routes.get(target.id)
        {
            self.render_route(ui, route, RouteRenderMode::Highlighted);
        }
        if let State::RouteSelected(target) = &self.state
            && let Some(route) = self.auto_routes.get(target.id)
        {
            self.render_route(ui, route, RouteRenderMode::Selected);
        }
        if let State::RouteEdgeHovered(target) = &self.state
            && let Some(route) = self.auto_routes.get(target.id)
        {
            self.render_route(ui, route, RouteRenderMode::Selected);
            if let Some(edge) = route.edge(target.edge_index) {
                let edge_start: Pos2 = edge.start;
                let edge_end: Pos2 = edge.end;
                let edge_dir = (edge_end - edge_start).normalized();
                let edge_start = edge_start + edge_dir * PORT_RADIUS;
                let edge_end = edge_end - edge_dir * PORT_RADIUS;
                ui.painter().line_segment(
                    [edge_start, edge_end],
                    (2.5, Color32::LIGHT_GREEN.gamma_multiply(0.7)),
                );
            }
        }
        if let State::RouteCornerHovered(target) = &self.state
            && let Some(route) = self.auto_routes.get(target.id)
        {
            self.render_route(ui, route, RouteRenderMode::Highlighted);
            if let Some(edge_1) = route.edge(target.edge_1) {
                let edge_1_end: Pos2 = edge_1.end;
                ui.painter().circle(
                    edge_1_end,
                    PORT_RADIUS,
                    Color32::LIGHT_RED.linear_multiply(0.5),
                    (0.5, Color32::BLACK),
                );
            }
        }
        if let State::RouteEdgeDragged(target) = &self.state
            && let Some(route) = self.auto_routes.get(target.id)
        {
            let projected_path = route
                .points()
                .into_iter()
                .map(snap_to_grid)
                .collect::<Vec<Pos2>>();
            let points = render_path_with_chamfered_corners(&projected_path);
            points.render(ui, (1.5, Color32::GRAY.gamma_multiply(0.2)));
        }
        if let State::WaypointHovered(target) = &self.state
            && let Some(route) = self.auto_routes.get(target.route)
        {
            self.render_route(ui, route, RouteRenderMode::Highlighted);
            if let Some(wp) = route.waypoint(target.waypoint) {
                ui.painter().circle(
                    wp.pos,
                    PORT_RADIUS,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (0.5, Color32::WHITE),
                );
            }
        }
        if let State::WaypointDragged(target) = &self.state
            && let Some(route) = self.auto_routes.get(target.route)
        {
            if let Some(wp) = route.waypoint(target.waypoint) {
                self.render_route(ui, route, RouteRenderMode::Selected);
                ui.painter().circle(
                    wp.pos,
                    PORT_RADIUS,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (1.0, Color32::WHITE),
                );
            }
        }
        if let State::TextAnchorHovered(target) = &self.state
            && let Some(route) = self.auto_routes.get(target.route)
        {
            self.render_route(ui, route, RouteRenderMode::Selected);
            if let Some(label) = route.label(target.label_id) {
                let loc_and_direction =
                    route.map_linear_distance_to_position(label.linear_distance);
                let pos = loc_and_direction.location;
                Self::draw_text_anchor(
                    ui,
                    pos,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (0.5, Color32::GRAY),
                );
            }
        }
        if let State::TextAnchorDragged(target) = &self.state
            && let Some(route) = self.auto_routes.get(target.route)
        {
            self.render_route(ui, route, RouteRenderMode::Selected);
            if let Some(label) = route.label(target.label_id) {
                let loc_and_direction =
                    route.map_linear_distance_to_position(label.linear_distance);
                let pos = loc_and_direction.location;
                Self::draw_text_anchor(
                    ui,
                    pos,
                    Color32::LIGHT_GREEN.linear_multiply(0.5),
                    (1.0, Color32::WHITE),
                );
            }
        }
        if let State::EditingRouteLabelText(target) = &self.state
            && let Some(route) = self.auto_routes.get_mut(target.id)
            && let Some((label_center, label)) = route.label_edit_details(target.label_id)
        {
            let editor_width = 25.0;
            let editor_position =
                Rect::from_center_size(label_center, vec2(editor_width, ROUTE_TEXT_SIZE));
            eprintln!("Editor position: {}", editor_position);
            let response = ui.place(
                editor_position,
                TextEdit::singleline(&mut label.text)
                    .font(egui::FontId::monospace(ROUTE_TEXT_SIZE))
                    .desired_width(f32::INFINITY),
            );
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                // Do something.
            }
        }
        if let State::AddTextButtonHovered(target) = &self.state {
            Self::draw_add_text_button(
                ui,
                target.button.pos,
                Color32::LIGHT_YELLOW,
                (0.5, Color32::WHITE),
            );
        }
    }
    fn handle_route_hover_check(&self, id: RouteId, response: Response) -> State {
        if let Some(hover_pos) = response.hover_pos()
            && let Some(route) = self.auto_routes.get(id)
        {
            for (waypoint_id, waypoint) in route.iter_waypoints() {
                if waypoint.pos.distance(hover_pos) <= PORT_RADIUS * 1.5 {
                    return WaypointHovered {
                        route: id,
                        waypoint: waypoint_id,
                    }
                    .into();
                }
            }
            if let Some((edge_1, edge_2)) = route.hovered_corner(hover_pos) {
                return RouteCornerHovered { id, edge_1, edge_2 }.into();
            }
            if let Some(edge_id) = route.hovered_edge(hover_pos)
                && let Some(edge) = route.edge(edge_id)
                && hover_pos.distance(edge.center()) <= PORT_RADIUS * 1.5
            {
                return RouteEdgeHovered {
                    id,
                    edge_index: edge_id,
                    direction: edge.direction(),
                }
                .into();
            }
            if let Some(label_id) = route.hit_text_anchor(hover_pos) {
                return TextAnchorHovered {
                    route: id,
                    label_id,
                }
                .into();
            }
            if let Some(button) = route.hit_add_text_button(hover_pos) {
                return AddTextButtonHovered {
                    route: id,
                    button: button.clone(),
                }
                .into();
            }
        }
        RouteSelected { id }.into()
    }
    fn handle_add_text(&self, response: Response) -> State {
        if let Some(pos) = response.hover_pos() {
            for (id, route) in self.auto_routes.iter() {
                if let Some(edge_id) = route.hovered_edge(pos) {
                    return AddTextHoveredRoute {
                        route: id,
                        edge_id,
                        pos,
                    }
                    .into();
                }
            }
        }
        State::AddText
    }
    fn handle_add_text_hovered_route(
        &mut self,
        inner: AddTextHoveredRoute,
        response: Response,
    ) -> State {
        if response.clicked_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
            && let Some(route) = self.auto_routes.get_mut(inner.route)
        {
            let label = route.allocate_label(pos);
            return EditingRouteLabelText {
                id: inner.route,
                label_id: label,
            }
            .into();
        }
        if let Some(route) = self.auto_routes.get(inner.route)
            && let Some(pos) = response.hover_pos()
            && let Some(edge) = route.hovered_edge(pos)
        {
            return AddTextHoveredRoute {
                route: inner.route,
                edge_id: edge,
                pos: pos,
            }
            .into();
        }
        State::AddText
    }
    fn handle_idle_state(&self, response: Response) -> State {
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(pos_start) = response.interact_pointer_pos()
        {
            if let Some((id, _)) = self
                .rect_boxes
                .iter()
                .find(|(_, r)| r.gui_rect().contains(pos_start))
            {
                return MovingRect {
                    rect: id,
                    delta_pos: vec2(0.0, 0.0),
                }
                .into();
            }
            return AddingRect {
                start_pos: pos_start,
                end_pos: snap_to_grid(pos_start),
            }
            .into();
        } else if response.is_pointer_button_down_on()
            && response
                .ctx
                .input(|i| i.pointer.button_down(PointerButton::Secondary))
        {
            return State::Panning;
        }
        if response.clicked_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            for (id, rect_box) in self.rect_boxes.iter() {
                if rect_box.gui_rect().contains(pos) {
                    return Selected { rect: id }.into();
                }
            }
            for (id, route) in self.auto_routes.iter() {
                if route.hovered_edge(pos).is_some() {
                    return RouteSelected { id }.into();
                }
            }
        }
        if let Some(hover_pos) = response.hover_pos() {
            if let Some(id) = self.route_hit(hover_pos) {
                return RouteHovered { id }.into();
            }
        }
        State::Idle
    }
    fn route_hit(&self, pos: Pos2) -> Option<RouteId> {
        for (id, route) in self.auto_routes.iter() {
            if route.hovered_edge(pos).is_some() {
                return Some(id);
            }
        }
        None
    }
    fn handle_selected_state(&mut self, inner: Selected, response: Response) -> State {
        let rect = inner.rect;
        if response.double_clicked_by(PointerButton::Primary) {
            if self.rect(rect).map(|b| b.is_port()).unwrap_or(false) {
                if let Some(bbox) = self.rect(rect) {
                    if let Some((pin_id, _)) = bbox.iter_pins().next() {
                        return EditingPinText { rect, pin: pin_id }.into();
                    }
                }
            } else {
                return EditingName { rect }.into();
            }
        }
        if response.clicked_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            if let Some(bbox) = self.rect_mut(rect)
                && let Some(pin) = bbox.add_pin_button_east()
                && pos.distance(pin) <= PORT_RADIUS
                && let Some(next_offset) = bbox.next_pin_offset(PinSide::East)
            {
                bbox.add_pin("port".into(), PinSide::East, next_offset);
                self.reroute = true;
                return Selected { rect }.into();
            }
            if let Some(bbox) = self.rect_mut(rect)
                && let Some(pin) = bbox.add_pin_button_west()
                && pos.distance(pin) <= PORT_RADIUS
                && let Some(next_offset) = bbox.next_pin_offset(PinSide::West)
            {
                bbox.add_pin("port".into(), PinSide::West, next_offset);
                self.reroute = true;
                return Selected { rect }.into();
            }
            if let Some((id, _)) = self
                .rect_boxes
                .iter()
                .find(|(_, r)| r.gui_rect().contains(pos))
            {
                return Selected { rect: id }.into();
            } else {
                return State::idle();
            }
        }
        if response.drag_started_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
            && let Some(hbox) = self.rect(rect)
            && let Some(lid) = hbox.iter_pins().find_map(|(lid, _)| {
                if hbox.pin_head_pos(lid)?.distance(pos) <= PORT_RADIUS {
                    Some(lid)
                } else {
                    None
                }
            })
        {
            return PinDragged {
                rect,
                pin: lid,
                delta_pos: vec2(0.0, 0.0),
            }
            .into();
        }
        if response.drag_started_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            if let Some((id, _)) = self
                .rect_boxes
                .iter()
                .find(|(_, r)| r.gui_rect().contains(pos))
            {
                return MovingRect {
                    rect: id,
                    delta_pos: vec2(0.0, 0.0),
                }
                .into();
            }
            return AddingRect {
                start_pos: pos,
                end_pos: snap_to_grid(pos),
            }
            .into();
        }
        if let Some(hover_pos) = response.hover_pos()
            && let Some(bbox) = self.rect(rect)
        {
            for mode in bbox.resize_modes() {
                if hover_pos.distance(control_corner(&bbox.gui_rect(), *mode)) < MOVE_HOVER_DISTANCE
                {
                    return PotentialResize { rect, mode: *mode }.into();
                }
            }
            let is_port = bbox.kind() == BoxKind::Port;
            for (pid, pin) in bbox.iter_pins() {
                if !is_port {
                    let pin_bbox = estimate_bbox_for_pin_text(bbox.gui_rect(), pin);
                    if pin_bbox.contains(hover_pos) {
                        eprintln!("Hovering over label {}", pin.text);
                        return PinLabelHovered { rect, pin: pid }.into();
                    }
                    let hamburger_rect = get_hamburger_rect(bbox.gui_rect(), pin).expand(GRIP_SHIM);
                    if hamburger_rect.contains(hover_pos) {
                        eprintln!("Hovering over grip for label {}", pin.text);
                        return PinLabelGripHovered { rect, pin: pid }.into();
                    }
                }
                let pin_location = get_control_pin_bbox(bbox.gui_rect(), pin);
                if pin_location.contains(hover_pos) {
                    eprintln!("Hovering over pin for label {}", pin.text);
                    return PinHeadHovered { rect, pin: pid }.into();
                }
            }
        }
        Selected { rect }.into()
    }
    fn handle_potential_resize(&self, inner: PotentialResize, response: Response) -> State {
        let PotentialResize { rect, mode } = inner;
        if let Some(hover_pos) = response.hover_pos()
            && let Some(bbox) = self.rect(rect)
            && hover_pos.distance(control_corner(&bbox.gui_rect(), mode)) >= MOVE_HOVER_DISTANCE
        {
            return Selected { rect }.into();
        }
        if response.drag_started_by(egui::PointerButton::Primary) {
            return ResizingRect {
                rect,
                mode,
                delta_pos: vec2(0.0, 0.0),
            }
            .into();
        }
        PotentialResize { rect, mode }.into()
    }
    fn handle_pin_label_hovered(&self, inner: PinLabelHovered, response: Response) -> State {
        let PinLabelHovered { rect, pin } = inner;
        if let Some(hover_pos) = response.hover_pos()
            && let Some(bbox) = self.rect(rect)
            && let Some(pin) = bbox.pin(pin)
        {
            let pin_bbox = estimate_bbox_for_pin_text(bbox.gui_rect(), pin);
            if !pin_bbox.contains(hover_pos) {
                return Selected { rect }.into();
            }
        }
        if response.double_clicked_by(egui::PointerButton::Primary) {
            return EditingPinText { rect, pin }.into();
        }
        PinLabelHovered { rect, pin }.into()
    }
    fn handle_route_label_hovered(&self, route: RouteLabelHovered, response: Response) -> State {
        if let Some(hover_pos) = response.hover_pos()
            && let Some(auto_route) = self.auto_routes.get(route.id)
        {
            let Some(edge_index) = auto_route.hovered_edge(hover_pos) else {
                return State::Idle;
            };
            if edge_index != route.edge_index {
                return State::Idle;
            }
        }
        route.into()
    }
    fn handle_pin_label_grip_hovered(
        &self,
        inner: PinLabelGripHovered,
        response: Response,
    ) -> State {
        let PinLabelGripHovered { rect, pin } = inner;
        if response.drag_started_by(egui::PointerButton::Primary) || response.dragged() {
            eprintln!("Starting to drag port label grip");
            return PinDragged {
                rect,
                pin,
                delta_pos: vec2(0.0, 0.0),
            }
            .into();
        }
        if let Some(hover_pos) = response.hover_pos()
            && let Some(bbox) = self.rect(rect)
            && let Some(pin) = bbox.pin(pin)
        {
            let hamburger_rect = get_hamburger_rect(bbox.gui_rect(), pin).expand(GRIP_SHIM);
            if !hamburger_rect.contains(hover_pos) {
                return Selected { rect }.into();
            }
        }
        PinLabelGripHovered { rect, pin }.into()
    }
    fn handle_pin_head_hovered(&self, inner: PinHeadHovered, response: Response) -> State {
        let PinHeadHovered { rect, pin } = inner;
        if let Some(hover_pos) = response.hover_pos()
            && let Some(bbox) = self.rect(rect)
            && let Some(pin) = bbox.pin(pin)
        {
            let pin_location = get_control_pin_bbox(bbox.gui_rect(), pin);
            if !pin_location.contains(hover_pos) {
                return Selected { rect }.into();
            }
        }
        if response.clicked_by(egui::PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            return InProgressAutoRoute {
                start: LineAnchor { rect, pin },
                waypoints: Store::default(),
                head: pos,
            }
            .into();
        }
        PinHeadHovered { rect, pin }.into()
    }
    fn handle_route_corner_hovered(
        &mut self,
        inner: RouteCornerHovered,
        response: Response,
    ) -> State {
        if (response.drag_started_by(egui::PointerButton::Primary) || response.dragged())
            && let Some(pos) = response.interact_pointer_pos()
            && let Some(route) = self.auto_routes.get_mut(inner.id)
        {
            eprintln!("Starting to drag route corner");
            if let Some(waypoint_id) = route.hit_waypoint(pos, PORT_RADIUS) {
                route.lock_waypoint(waypoint_id);
                return WaypointDragged {
                    route: inner.id,
                    waypoint: waypoint_id,
                    delta_pos: vec2(0.0, 0.0),
                }
                .into();
            }
            if let Some(_) = route.edge(inner.edge_1) {
                let waypoint_id = route.add_waypoint(snap_to_grid(pos));
                route.lock_waypoint(waypoint_id);
                return WaypointDragged {
                    route: inner.id,
                    waypoint: waypoint_id,
                    delta_pos: vec2(0.0, 0.0),
                }
                .into();
            }
        }
        self.handle_route_hover_check(inner.id, response)
    }
    fn handle_route_hovered(&mut self, _inner: RouteHovered, response: Response) -> State {
        if response.clicked_by(egui::PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            if let Some(id) = self.route_hit(pos) {
                return RouteSelected { id }.into();
            }
        }
        if let Some(hover_pos) = response.hover_pos()
            && let Some(id) = self.route_hit(hover_pos)
        {
            return RouteHovered { id }.into();
        }
        return State::Idle;
    }
    fn handle_route_selected(&self, inner: RouteSelected, response: Response) -> State {
        if response.clicked_by(egui::PointerButton::Primary) {
            return State::Idle;
        }
        self.handle_route_hover_check(inner.id, response)
    }
    fn handle_route_edge_hovered(&mut self, target: RouteEdgeHovered, response: Response) -> State {
        if (response.drag_started_by(egui::PointerButton::Primary) || response.dragged())
            && let Some(route) = self.auto_routes.get_mut(target.id)
            && let Some(edge) = route.edge(target.edge_index).cloned()
        {
            eprintln!("Starting to drag route edge");
            eprintln!("Raw edge: {:?}", edge);
            let wp1 = route.add_waypoint(edge.waypoint_position_start());
            let wp2 = route.add_waypoint(edge.waypoint_position_end());
            route.lock_waypoint(wp1);
            route.lock_waypoint(wp2);
            self.reroute = true;
            self.ripup_set.push(target.id);
            if wp1 == wp2 {
                return WaypointDragged {
                    route: target.id,
                    waypoint: wp1,
                    delta_pos: vec2(0.0, 0.0),
                }
                .into();
            }
            return RouteEdgeDragged {
                id: target.id,
                direction: edge.direction(),
                start_waypoint: wp1,
                end_waypoint: wp2,
                delta_pos: vec2(0.0, 0.0),
            }
            .into();
        }
        self.handle_route_hover_check(target.id, response)
    }
    fn handle_add_text_button_hovered(
        &mut self,
        inner: AddTextButtonHovered,
        response: Response,
    ) -> State {
        if response.clicked_by(PointerButton::Primary)
            && let Some(route) = self.auto_routes.get_mut(inner.route)
        {
            let lin_pos = inner.button.linear_position;
            let loc_and_dir = route.map_linear_distance_to_position(lin_pos);
            let label_id = route.allocate_label(loc_and_dir.location);
            return EditingRouteLabelText {
                id: inner.route,
                label_id,
            }
            .into();
        }
        self.handle_route_hover_check(inner.route, response)
    }
    fn handle_text_anchor_hovered(
        &mut self,
        inner: TextAnchorHovered,
        response: Response,
    ) -> State {
        if response.drag_started_by(egui::PointerButton::Primary) || response.dragged() {
            eprintln!("Starting to drag text anchor");
            return TextAnchorDragged {
                route: inner.route,
                label_id: inner.label_id,
                delta_pos: vec2(0.0, 0.0),
            }
            .into();
        }
        if response.double_clicked_by(egui::PointerButton::Primary) {
            return EditingRouteLabelText {
                id: inner.route,
                label_id: inner.label_id,
            }
            .into();
        }
        self.handle_route_hover_check(inner.route, response)
    }
    fn handle_text_anchor_dragged(
        &mut self,
        inner: TextAnchorDragged,
        response: Response,
    ) -> State {
        if response.dragged_by(egui::PointerButton::Primary)
            && let Some(route) = self.auto_routes.get_mut(inner.route)
            && let Some(label) = route.label(inner.label_id)
        {
            let delta = response.drag_delta();
            let label_distance = label.linear_distance;
            let (direction, flip_sign) = if let Some((_, edge)) = route.find_edge(label_distance) {
                let dir = edge.direction();
                let flip_sign = match dir {
                    RouteDirection::Horizontal => edge.start.x > edge.end.x,
                    RouteDirection::Vertical => edge.start.y > edge.end.y,
                };
                (dir, flip_sign)
            } else {
                (RouteDirection::Horizontal, false)
            };
            if let Some(label) = route.label_mut(inner.label_id) {
                match direction {
                    RouteDirection::Horizontal => {
                        label.linear_distance += if flip_sign { -delta.x } else { delta.x };
                    }
                    RouteDirection::Vertical => {
                        label.linear_distance += if flip_sign { -delta.y } else { delta.y };
                    }
                }
            }
            route.update_label_positions();
            return inner.into();
        } else if (response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged())
            && let Some(route) = self.auto_routes.get_mut(inner.route)
        {
            route.update_waypoints();
            return RouteSelected { id: inner.route }.into();
        }
        inner.into()
    }
    fn handle_waypoint_hovered(&mut self, inner: WaypointHovered, response: Response) -> State {
        if (response.drag_started_by(egui::PointerButton::Primary) || response.dragged())
            && let Some(route) = self.auto_routes.get_mut(inner.route)
        {
            eprintln!("Starting to drag waypoint");
            route.lock_waypoint(inner.waypoint);
            return WaypointDragged {
                route: inner.route,
                waypoint: inner.waypoint,
                delta_pos: vec2(0.0, 0.0),
            }
            .into();
        }
        self.handle_route_hover_check(inner.route, response)
    }
    fn handle_waypoint_dragged(&mut self, inner: WaypointDragged, response: Response) -> State {
        if response.dragged_by(egui::PointerButton::Primary)
            && let Some(route) = self.auto_routes.get_mut(inner.route)
        {
            let delta = response.drag_delta();
            if let Some(wp) = route.waypoint_mut(inner.waypoint) {
                wp.pos += delta;
            }
            self.reroute = true;
            self.ripup_set.push(inner.route);
            return inner.into();
        } else if (response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged())
            && let Some(route) = self.auto_routes.get_mut(inner.route)
        {
            route.iter_waypoints_mut().for_each(|(_, wp)| {
                wp.pos = snap_to_grid(wp.pos);
                wp.unlock();
            });
            self.reroute = true;
            return RouteSelected { id: inner.route }.into();
        }
        inner.into()
    }
    fn handle_resizing_rect(&mut self, inner: ResizingRect, response: Response) -> State {
        let ResizingRect {
            rect,
            mode,
            delta_pos,
        } = inner;
        if response.dragged_by(egui::PointerButton::Primary) {
            let mut delta = response.drag_delta();
            if self.rect(rect).map(|b| b.is_port()).unwrap_or(false) {
                delta.y = 0.0;
            }
            return ResizingRect {
                rect,
                mode,
                delta_pos: delta_pos + delta,
            }
            .into();
        } else if (response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged())
            && let Some(bbox) = self.rect_mut(rect)
        {
            let new_rect = grid_rect(resize_rect(&bbox.gui_rect(), mode, delta_pos));
            *bbox.gui_rect_mut() = if bbox.is_port() {
                Rect::from_min_size(
                    pos2(new_rect.min.x, bbox.gui_rect().min.y),
                    vec2(new_rect.width(), PORT_HEIGHT),
                )
            } else {
                new_rect
            };
            return Selected { rect }.into();
        }
        ResizingRect {
            rect,
            mode,
            delta_pos,
        }
        .into()
    }
    fn handle_moving_rect(&mut self, inner: MovingRect, response: Response) -> State {
        let MovingRect { rect, delta_pos } = inner;
        if response.dragged_by(egui::PointerButton::Primary) {
            let delta = response.drag_delta();
            return MovingRect {
                rect,
                delta_pos: delta_pos + delta,
            }
            .into();
        } else if (response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged())
            && let Some(bbox) = self.rect_mut(rect)
        {
            *bbox.gui_rect_mut() = grid_rect(bbox.gui_rect().translate(delta_pos));
            return Selected { rect }.into();
        }
        MovingRect { rect, delta_pos }.into()
    }
    fn handle_adding_rect(&mut self, inner: AddingRect, response: Response) -> State {
        let AddingRect { start_pos, end_pos } = inner;
        if response.dragged_by(egui::PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            return AddingRect {
                start_pos: snap_to_grid(start_pos),
                end_pos: snap_to_grid(pos),
            }
            .into();
        } else if response.drag_stopped_by(egui::PointerButton::Primary) {
            let candidate_rect = Rect::from_two_pos(start_pos, end_pos);
            if candidate_rect.width() > GRID_SIZE && candidate_rect.height() > GRID_SIZE {
                let rect = self.add_rect_box(start_pos, end_pos);
                return Selected { rect }.into();
            } else {
                return State::idle();
            }
        }
        if response
            .ctx
            .input(|i| i.pointer.button_down(PointerButton::Secondary))
        {
            return State::panning();
        }
        AddingRect { start_pos, end_pos }.into()
    }
    fn handle_panning(&self, response: Response) -> State {
        if response.drag_stopped() {
            return State::idle();
        }
        State::panning()
    }
    fn handle_editing_name(&self, inner: EditingName, response: Response) -> State {
        let rect = inner.rect;
        if response.clicked() {
            return Selected { rect }.into();
        }
        EditingName { rect }.into()
    }
    fn handle_editing_pin_text(&self, inner: EditingPinText, response: Response) -> State {
        let EditingPinText { rect, pin } = inner;
        if response.clicked() {
            return Selected { rect }.into();
        }
        EditingPinText { rect, pin }.into()
    }
    fn handle_editing_route_label_text(
        &mut self,
        inner: EditingRouteLabelText,
        response: Response,
    ) -> State {
        if response.clicked() {
            if let Some(route) = self.auto_routes.get_mut(inner.id) {
                route.update_waypoints();
            }
            return State::Idle;
        }
        inner.into()
    }
    fn handle_route_edge_dragged(&mut self, target: RouteEdgeDragged, response: Response) -> State {
        if response.dragged_by(egui::PointerButton::Primary) {
            let mut delta = response.drag_delta();
            if target.direction == RouteDirection::Horizontal {
                delta.x = 0.0;
            } else {
                delta.y = 0.0;
            }
            let route = self.auto_routes.get_mut(target.id).unwrap();
            route.waypoint_mut(target.start_waypoint).unwrap().pos += delta;
            route.waypoint_mut(target.end_waypoint).unwrap().pos += delta;
            self.reroute = true;
            self.ripup_set.push(target.id);
            return target.into();
        } else if response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged() {
            let route = self.auto_routes.get_mut(target.id).unwrap();
            if let Some(start_wp) = route.waypoint_mut(target.start_waypoint) {
                start_wp.pos = snap_to_grid(start_wp.pos);
                start_wp.unlock();
            }
            if let Some(end_wp) = route.waypoint_mut(target.end_waypoint) {
                end_wp.pos = snap_to_grid(end_wp.pos);
                end_wp.unlock();
            }
            self.reroute = true;
            return RouteSelected { id: target.id }.into();
        }
        State::RouteEdgeDragged(target)
    }
    fn handle_pin_dragged(&mut self, inner: PinDragged, response: Response) -> State {
        let PinDragged {
            rect,
            pin,
            delta_pos,
        } = inner;
        if let Some(pos) = response.interact_pointer_pos()
            && let Some(rbox) = self.rect_mut(rect)
        {
            let center_line = rbox.gui_rect().center().x;
            if let Some(pin) = rbox.pins_mut(pin) {
                if pos.x < center_line {
                    pin.side = PinSide::West;
                } else {
                    pin.side = PinSide::East;
                }
            }
        }
        if response.dragged_by(egui::PointerButton::Primary) {
            let delta = response.drag_delta();
            return PinDragged {
                rect,
                pin,
                delta_pos: delta_pos + delta,
            }
            .into();
        } else if response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged() {
            if let Some(rbox) = self.rect_mut(rect) {
                rbox.update_pin_offset(pin, delta_pos.y);
            }
            self.reroute = true;
            return Selected { rect }.into();
        }
        PinDragged {
            rect,
            pin,
            delta_pos,
        }
        .into()
    }
    fn handle_in_progress_auto_routing(
        &mut self,
        mut auto_route: InProgressAutoRoute,
        response: Response,
    ) -> State {
        if response.clicked_by(egui::PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            let _ = auto_route.waypoints.insert(Waypoint {
                pos: snap_to_grid(pos),
                locked: true,
            });
            return auto_route.into();
        }
        if let Some(pos) = response.hover_pos() {
            auto_route.head = pos;
            if let Some((tail, _)) = self
                .iter_anchor_positions()
                .find(|&(_, anchor_pos)| anchor_pos.distance(pos) < PORT_RADIUS)
                && tail != auto_route.start
            {
                return ProposedAutoRoute {
                    start: auto_route.start,
                    waypoints: auto_route.waypoints,
                    finish: tail,
                }
                .into();
            }
        }
        State::InProgressAutoRoute(auto_route)
    }
    fn handle_proposed_auto_route(
        &mut self,
        mut proposed_route: ProposedAutoRoute,
        response: Response,
    ) -> State {
        if response.clicked_by(egui::PointerButton::Primary) {
            let mut waypoints = std::mem::take(&mut proposed_route.waypoints);
            waypoints.iter_mut().for_each(|(_, wp)| wp.unlock());
            let mut route = AutoRoute::build(
                proposed_route.start,
                proposed_route.finish,
                &self.auto_route,
                waypoints,
                Store::default(),
            );
            route.update_waypoints();
            let id = self.auto_routes.insert(route);
            return RouteSelected { id }.into();
        }
        if let Some(pos) = response.hover_pos() {
            if let Some((tail, _)) = self
                .iter_anchor_positions()
                .find(|&(_, anchor_pos)| anchor_pos.distance(pos) < PORT_RADIUS)
                && tail != proposed_route.start
            {
                return ProposedAutoRoute {
                    start: proposed_route.start,
                    waypoints: proposed_route.waypoints,
                    finish: tail,
                }
                .into();
            }
            return InProgressAutoRoute {
                start: proposed_route.start,
                waypoints: proposed_route.waypoints,
                head: pos,
            }
            .into();
        }
        proposed_route.into()
    }
    pub fn update_state(&mut self, response: Response) {
        let old_state = std::mem::take(&mut self.state);
        let mut route_fixup = false;
        self.reroute = false;
        self.state = match old_state {
            State::Idle => self.handle_idle_state(response),
            State::AddText => self.handle_add_text(response),
            State::AddTextHoveredRoute(inner) => {
                self.handle_add_text_hovered_route(inner, response)
            }
            State::RouteHovered(route) => self.handle_route_hovered(route, response),
            State::RouteSelected(route) => self.handle_route_selected(route, response),
            State::RouteLabelHovered(inner) => self.handle_route_label_hovered(inner, response),
            State::RouteEdgeHovered(route) => self.handle_route_edge_hovered(route, response),
            State::RouteCornerHovered(route) => self.handle_route_corner_hovered(route, response),
            State::WaypointHovered(inner) => self.handle_waypoint_hovered(inner, response),
            State::TextAnchorHovered(inner) => self.handle_text_anchor_hovered(inner, response),
            State::TextAnchorDragged(inner) => self.handle_text_anchor_dragged(inner, response),
            State::AddTextButtonHovered(inner) => {
                self.handle_add_text_button_hovered(inner, response)
            }
            State::WaypointDragged(inner) => self.handle_waypoint_dragged(inner, response),
            State::RouteEdgeDragged(inner) => {
                route_fixup = true;
                self.handle_route_edge_dragged(inner, response)
            }
            State::Selected(inner) => self.handle_selected_state(inner, response),
            State::PotentialResize(inner) => self.handle_potential_resize(inner, response),
            State::PinLabelHovered(inner) => self.handle_pin_label_hovered(inner, response),
            State::PinLabelGripHovered(inner) => {
                self.handle_pin_label_grip_hovered(inner, response)
            }
            State::PinHeadHovered(inner) => self.handle_pin_head_hovered(inner, response),
            State::ResizingRect(inner) => {
                route_fixup = true;
                self.handle_resizing_rect(inner, response)
            }
            State::MovingRect(inner) => {
                route_fixup = true;
                self.handle_moving_rect(inner, response)
            }
            State::AddingRect(inner) => {
                route_fixup = true;
                self.handle_adding_rect(inner, response)
            }
            State::Panning => self.handle_panning(response),
            State::EditingName(inner) => self.handle_editing_name(inner, response),
            State::EditingPinText(inner) => self.handle_editing_pin_text(inner, response),
            State::EditingRouteLabelText(inner) => {
                self.handle_editing_route_label_text(inner, response)
            }
            State::PinDragged(inner) => {
                route_fixup = true;
                self.handle_pin_dragged(inner, response)
            }
            State::InProgressAutoRoute(inner) => {
                route_fixup = true;
                self.handle_in_progress_auto_routing(inner, response)
            }
            State::ProposedAutoRoute(inner) => {
                route_fixup = true;
                self.handle_proposed_auto_route(inner, response)
            }
        };
        if route_fixup || self.reroute {
            self.update_graph();
        }
    }
    fn build_router(&self) -> RouterNG {
        let mut builder = RouterNGBuilder::default();
        for (id, rect_box) in self.rect_boxes.iter() {
            let Some(effective_rect) = self.routing_box(id) else {
                continue;
            };
            builder.add_block(effective_rect.left_top(), effective_rect.right_bottom());
            for (pid, pin) in rect_box.iter_pins() {
                let Some(anchor_pos) = rect_box.anchor_point_with_rect(effective_rect, pid) else {
                    continue;
                };
                let anchor_pos = match pin.side {
                    PinSide::East => anchor_pos + vec2(GRID_SIZE, 0.0),
                    PinSide::West => anchor_pos - vec2(GRID_SIZE, 0.0),
                };
                builder.add_h_channel(anchor_pos, COST_ZERO);
            }
        }
        builder.build()
    }
    fn update_graph(&mut self) {
        let mut router = self.build_router();
        // First all routes that haven't changed
        let mut routes = std::mem::take(&mut self.auto_routes);
        for (id, route) in routes.iter_mut() {
            let Some(anchor_start) = self.anchor(route.start()) else {
                continue;
            };
            let Some(anchor_end) = self.anchor(route.finish()) else {
                continue;
            };
            let anchor_start = snap_to_grid(anchor_start);
            let anchor_end = snap_to_grid(anchor_end);
            if Some(route.start_pos()) == self.anchor(route.start())
                && Some(route.end_pos()) == self.anchor(route.finish())
                && !router.is_route_blocked(route.iter_edges().map(|(_, edge)| edge))
                && !self.ripup_set.contains(&id)
            {
                router.add_existing_route(route.iter_edges().map(|(_, edge)| edge), WIRE_COST);
            } else {
                route.rip_and_reroute(anchor_start, anchor_end, &mut router);
            }
        }
        self.auto_routes = routes;
        if let State::InProgressAutoRoute(inner) = &self.state
            && let Some(start_pos) = self.anchor(inner.start)
        {
            eprintln!("Auto-routing from {:?} to {:?}", inner.start, inner.head);
            let head_pos = snap_to_grid(inner.head);
            self.auto_route = router.waypoint_path(start_pos, &inner.waypoints, head_pos);
        }
        if let State::ProposedAutoRoute(inner) = &self.state
            && let Some(start_pos) = self.anchor(inner.start)
            && let Some(end_pos) = self.anchor(inner.finish)
        {
            let start_pos = snap_to_grid(start_pos);
            let end = snap_to_grid(end_pos);
            self.auto_route = router.waypoint_path(start_pos, &inner.waypoints, end);
        }
    }
    pub fn demo() -> Self {
        demo_drawing()
    }
}

pub fn demo_drawing() -> Drawing {
    let mut drawing = Drawing::default();
    let origin_1 = pos2(330.0, 300.0);
    let size = vec2(200.0, 200.0);
    let box1_id = drawing.add_rect_box(origin_1, origin_1 + size);
    let box1 = drawing.rect_mut(box1_id).unwrap();
    let box1_pin1 = box1.add_pin(
        "i.1.write_logic".to_string(),
        PinSide::West,
        GRID_SIZE * 1.0,
    );
    let box1_anchor1 = LineAnchor {
        rect: box1_id,
        pin: box1_pin1,
    };
    let box1_pin2 = box1.add_pin(
        "i.0.write_logic".to_string(),
        PinSide::West,
        GRID_SIZE * 2.0,
    );
    let box1_anchor2 = LineAnchor {
        rect: box1_id,
        pin: box1_pin2,
    };
    let origin_2 = pos2(0.0, 0.0);
    let box2_id = drawing.add_rect_box(origin_2, origin_2 + size);
    let box2 = drawing.rect_mut(box2_id).unwrap();
    let box2_port1 = box2.add_pin("o.1.read_logic".to_string(), PinSide::East, GRID_SIZE * 1.0);
    let box2_anchor1 = LineAnchor {
        rect: box2_id,
        pin: box2_port1,
    };
    let box2_pin2 = box2.add_pin("o.0.read_logic".to_string(), PinSide::East, GRID_SIZE * 2.0);
    let box2_anchor2 = LineAnchor {
        rect: box2_id,
        pin: box2_pin2,
    };
    // Create a route
    /*
     *
     * ProposedAutoRoute(ProposedAutoRoute { start: LineAnchor { rect: RectId(0), pin: PinId(0) }, waypoints: [Waypoint { pos: [-285.0 240.0], id: WaypointId(0), label: None, locked: true }, Waypoint { pos: [-285.0 -90.0], id: WaypointId(1), label: None, locked: true }, Waypoint { pos: [210.0 -90.0], id: WaypointId(2), label: None, locked: true }, Waypoint { pos: [210.0 15.0], id: WaypointId(3), label: None, locked: true }], finish: LineAnchor { rect: RectId(1), pin: PinId(0) } })
     *
     */
    let mut waypoints = Store::<WaypointId, Waypoint>::default();
    [
        Waypoint {
            pos: pos2(-240.0, 240.0),
            locked: true,
        },
        Waypoint {
            pos: pos2(-240.0, -90.0),
            locked: true,
        },
        Waypoint {
            pos: pos2(255.0, -90.0),
            locked: true,
        },
        Waypoint {
            pos: pos2(255.0, 15.0),
            locked: true,
        },
    ]
    .into_iter()
    .for_each(|wp| {
        waypoints.insert(wp);
    });
    let mut router = drawing.build_router();
    let start = drawing.anchor(box1_anchor1).unwrap();
    let finish = drawing.anchor(box2_anchor1).unwrap();
    let path = router.waypoint_path(start, &waypoints, finish);
    let mut route = AutoRoute::build(
        box1_anchor1,
        box2_anchor1,
        &path,
        waypoints,
        Store::default(),
    );
    route.update_waypoints();
    drawing.auto_routes.insert(route);
    let mut router = drawing.build_router();
    let start = drawing.anchor(box1_anchor2).unwrap();
    let finish = drawing.anchor(box2_anchor2).unwrap();
    let path = router.waypoint_path(start, &Store::default(), finish);
    let mut route = AutoRoute::build(
        box1_anchor2,
        box2_anchor2,
        &path,
        Store::default(),
        Store::default(),
    );
    route.update_waypoints();
    drawing.auto_routes.insert(route);
    // Demo ports: one East-facing input to the left of box1, one West-facing output to the right of box2
    drawing.add_port_box(
        "clk".to_string(),
        PinSide::East,
        Rect::from_center_size(pos2(160.0, 315.0), vec2(4.0 * GRID_SIZE, PORT_HEIGHT)),
    );
    drawing.add_port_box(
        "out".to_string(),
        PinSide::West,
        Rect::from_center_size(pos2(580.0, 15.0), vec2(4.0 * GRID_SIZE, PORT_HEIGHT)),
    );
    drawing
}
