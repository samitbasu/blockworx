use egui::{
    Align2, Color32, PointerButton, Pos2, Rect, Response, Stroke, StrokeKind, TextEdit, Ui,
    epaint::TextShape, pos2, vec2,
};

use crate::{
    grid::{
        GRID_SIZE, MOVE_HOVER_DISTANCE, PORT_HEIGHT, PORT_RADIUS, ROUTE_TEXT_SIZE, SHIM, grid_rect,
        round_to_grid, snap_to_grid,
    },
    router::{RouterNG, RouterNGBuilder, TaggedPoint, WIRE_COST, cost::COST_ZERO},
    state::*,
    store::*,
    theme::get_theme,
    turtle::Mark,
    widget::{
        auto_route::AutoRoute,
        block::{Block, TitleSide, control_corner, resize_rect},
        direction::RouteDirection,
        pin::PinSide,
        port::Port,
        render::{
            FocusResult, estimate_bbox_for_pin_text, get_control_pin_bbox, get_hamburger_rect,
            render_path_with_chamfered_corners,
        },
        shape::{BaseShape, Shape},
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
    rect_boxes: Store<RectId, Shape>,
    auto_routes: Store<RouteId, AutoRoute>,
    state: State,
    auto_route: Vec<TaggedPoint>,
    reroute: bool,
    ripup_set: Vec<RouteId>,
    debug_marks: Vec<Mark>,
}

enum RouteRenderMode {
    Normal,
    Highlighted,
    Selected,
}

impl Drawing {
    pub fn rect(&self, id: RectId) -> Option<&Shape> {
        self.rect_boxes.get(id)
    }
    pub fn rect_mut(&mut self, id: RectId) -> Option<&mut Shape> {
        self.rect_boxes.get_mut(id)
    }
    pub fn add_rect_box(&mut self, start: Pos2, end: Pos2) -> RectId {
        self.rect_boxes
            .insert(Block::new("Untitled".to_string(), Rect::from_two_pos(start, end)).into())
    }
    pub fn add_port_box(&mut self, pin_name: String, side: PinSide, inner: Rect) -> RectId {
        self.rect_boxes
            .insert(Port::new(pin_name, side, inner).into())
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
    pub fn with_anchors(&self, mut f: impl FnMut(LineAnchor)) {
        self.rect_boxes.iter().for_each(|(rect_id, rect)| {
            rect.with_pins(|pin_id, _| {
                f(LineAnchor {
                    rect: rect_id,
                    pin: pin_id,
                })
            })
        });
    }
    pub fn with_anchors_and_positions(&self, mut f: impl FnMut(LineAnchor, Pos2)) {
        self.with_anchors(|anchor| {
            if let Some(pos) = self.anchor(anchor) {
                f(anchor, pos);
            }
        });
    }
    pub fn find_anchor<T>(&self, mut f: impl FnMut(LineAnchor, Pos2) -> Option<T>) -> Option<T> {
        let mut ret = None;
        self.with_anchors_and_positions(|anchor, pos| {
            if ret.is_none() {
                ret = f(anchor, pos);
            }
        });
        ret
    }
    fn render_route(&self, ui: &mut Ui, route: &AutoRoute, mode: RouteRenderMode) {
        let theme = get_theme(ui);
        let route_stroke = match mode {
            RouteRenderMode::Normal => (1.7, theme.route_normal),
            RouteRenderMode::Highlighted => (2.5, theme.route_highlighted),
            RouteRenderMode::Selected => (2.5, theme.route_selected),
        };
        let points = render_path_with_chamfered_corners(&route.points());
        points.render(ui, route_stroke);
        let text_color = match mode {
            RouteRenderMode::Normal => theme.route_normal,
            RouteRenderMode::Highlighted => theme.route_highlighted,
            RouteRenderMode::Selected => theme.route_selected,
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
                    let galley = ui.ctx().fonts_mut(|fv| {
                        fv.layout_no_wrap(
                            label.text.clone(),
                            egui::FontId::monospace(ROUTE_TEXT_SIZE),
                            text_color,
                        )
                    });
                    let mut text = TextShape::new(pos, galley, text_color)
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
                    theme.waypoint_fill,
                    (0.5, theme.control_handle_stroke),
                );
            }
            for dh in route.drag_handles() {
                ui.painter().rect(
                    Rect::from_center_size(dh, vec2(PORT_RADIUS * 2.0, PORT_RADIUS * 2.0)),
                    PORT_RADIUS / 4.0,
                    theme.waypoint_fill,
                    (0.5, theme.control_handle_stroke),
                    StrokeKind::Middle,
                );
            }
            for ta in route.text_anchors() {
                Self::draw_text_anchor(
                    ui,
                    ta,
                    theme.waypoint_fill,
                    (0.5, theme.control_handle_stroke),
                );
            }
            for at in route.all_add_text_buttons() {
                Self::draw_add_text_button(
                    ui,
                    at.pos,
                    theme.add_button_fill,
                    (0.5, theme.control_handle_stroke),
                );
            }
        }
    }
    fn draw_text_anchor(ui: &mut Ui, ta: Pos2, fill: Color32, stroke: impl Into<Stroke>) {
        let stroke: Stroke = stroke.into();
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
        let theme = get_theme(ui);
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
            theme.route_in_progress,
        );
    }
    pub fn render(&mut self, ui: &mut Ui) {
        let theme = get_theme(ui);
        ui.output_mut(|o| o.cursor_icon = self.state.cursor());
        (-100..=100).map(|y| y as f32 * GRID_SIZE).for_each(|h| {
            ui.painter()
                .hline(-10_000.0f32..=10_000.0f32, h, (0.15, theme.grid_line));
        });
        (-100..=100).map(|x| x as f32 * GRID_SIZE).for_each(|v| {
            ui.painter()
                .vline(v, -10_000.0f32..=10_000.0f32, (0.15, theme.grid_line));
        });
        crate::turtle::draw(&self.debug_marks, ui.painter());
        for route in self.auto_routes.values() {
            self.render_route(ui, &route, RouteRenderMode::Normal);
        }
        for (id, rect_box) in self.rect_boxes.iter_mut() {
            let mode = self.state.render_mode_for_id(id);
            if rect_box.render(mode, ui) == FocusResult::LostFocus {
                self.state = Selected { rect: id }.into();
            }
        }
        if let State::AddingRect(AddingRect { start_pos, end_pos }) = &self.state {
            let rect = Rect::from_two_pos(*start_pos, *end_pos);
            ui.painter().rect(
                rect,
                3.0,
                Color32::TRANSPARENT,
                (1.0, theme.selection_frame),
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
            points.render(ui, (0.5, theme.route_in_progress));
            inner.waypoints.iter().for_each(|(_, wp)| {
                ui.painter()
                    .circle_filled(wp.pos, PORT_RADIUS, theme.route_in_progress);
            });
        }
        if let State::ProposedAutoRoute(inner) = &self.state {
            let points = self
                .auto_route
                .iter()
                .map(|p| p.pos.into())
                .collect::<Vec<Pos2>>();
            let points = render_path_with_chamfered_corners(&points);
            points.render(ui, (1.5, theme.route_in_progress));
            if let Some(start_pos) = self.anchor(inner.start) {
                ui.painter().circle(
                    start_pos,
                    PORT_RADIUS,
                    theme.route_proposed_endpoint,
                    (0.5, theme.route_proposed_endpoint),
                );
            }
            if let Some(end_pos) = self.anchor(inner.finish) {
                ui.painter().circle(
                    end_pos,
                    PORT_RADIUS,
                    theme.route_proposed_endpoint,
                    (0.5, theme.route_proposed_endpoint),
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
                ui.painter()
                    .line_segment([edge_start, edge_end], (2.5, theme.route_edge_highlight));
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
                    theme.corner_highlight_fill,
                    (0.5, theme.control_handle_stroke),
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
            points.render(ui, (1.5, theme.edge_drag_preview));
        }
        if let State::WaypointHovered(target) = &self.state
            && let Some(route) = self.auto_routes.get(target.route)
        {
            self.render_route(ui, route, RouteRenderMode::Highlighted);
            if let Some(wp) = route.waypoint(target.waypoint) {
                ui.painter().circle(
                    wp.pos,
                    PORT_RADIUS,
                    theme.waypoint_fill,
                    (0.5, theme.control_handle_fill),
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
                    theme.waypoint_fill,
                    (1.0, theme.control_handle_fill),
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
                Self::draw_text_anchor(ui, pos, theme.waypoint_fill, (0.5, theme.hover_fill));
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
                    theme.waypoint_fill,
                    (1.0, theme.control_handle_fill),
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
                theme.route_in_progress,
                (0.5, theme.control_handle_fill),
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
        if response.double_clicked_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
            && let Some(rect_box) = self.rect(rect)
            && let Shape::Block(block) = rect_box
            && block.title_bbox().contains(pos)
        {
            return EditingName { rect }.into();
        }
        if response.clicked_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            if let Some(ps) = self.rect_mut(rect)
                && let Some(pin_pos) = ps.add_pin_button_east()
                && pos.distance(pin_pos) <= PORT_RADIUS
                && let Some(next_offset) = ps.next_pin_offset(PinSide::East)
            {
                ps.add_pin("port".into(), PinSide::East, next_offset);
                self.reroute = true;
                return Selected { rect }.into();
            }
            if let Some(ps) = self.rect_mut(rect)
                && let Some(pin_pos) = ps.add_pin_button_west()
                && pos.distance(pin_pos) <= PORT_RADIUS
                && let Some(next_offset) = ps.next_pin_offset(PinSide::West)
            {
                ps.add_pin("port".into(), PinSide::West, next_offset);
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
            && let Some(lid) = hbox.find_pin(|lid, _pin| {
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
            for &mode in bbox.resize_modes() {
                if hover_pos.distance(control_corner(&bbox.gui_rect(), mode)) < MOVE_HOVER_DISTANCE
                {
                    return PotentialResize { rect, mode }.into();
                }
            }
            if let Some(title_position_anchor) = bbox.title_anchor() {
                if hover_pos.distance(title_position_anchor) < MOVE_HOVER_DISTANCE {
                    return TitleControlHovered { rect }.into();
                }
            }
            if let Shape::Block(block) = &bbox
                && block.title_bbox().contains(hover_pos)
            {
                return TitleHovered { rect }.into();
            }
            if let Some(state) = bbox.find_pin(|pid, pin| {
                let pin_bbox = estimate_bbox_for_pin_text(bbox.gui_rect(), pin);
                if pin_bbox.contains(hover_pos) {
                    eprintln!("Hovering over label {}", pin.text);
                    return Some(PinLabelHovered { rect, pin: pid }.into());
                }
                let hamburger_rect = get_hamburger_rect(bbox.gui_rect(), pin).expand(GRIP_SHIM);
                if hamburger_rect.contains(hover_pos) {
                    eprintln!("Hovering over grip for label {}", pin.text);
                    return Some(PinLabelGripHovered { rect, pin: pid }.into());
                }
                let pin_location = get_control_pin_bbox(bbox.gui_rect(), pin);
                if pin_location.contains(hover_pos) {
                    eprintln!("Hovering over pin for label {}", pin.text);
                    return Some(PinHeadHovered { rect, pin: pid }.into());
                }
                None
            }) {
                return state;
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
    fn handle_title_control_hovered(
        &self,
        inner: TitleControlHovered,
        response: Response,
    ) -> State {
        let TitleControlHovered { rect } = inner;
        if let Some(hover_pos) = response.hover_pos()
            && let Some(bbox) = self.rect(rect)
            && let Some(title_anchor) = bbox.title_anchor()
            && hover_pos.distance(title_anchor) >= MOVE_HOVER_DISTANCE
        {
            return Selected { rect }.into();
        }
        if response.drag_started_by(egui::PointerButton::Primary) {
            return TitleControlDragged {
                rect,
                delta_pos: vec2(0.0, 0.0),
            }
            .into();
        }
        TitleControlHovered { rect }.into()
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
    fn handle_title_hovered(&mut self, inner: TitleHovered, response: Response) -> State {
        if response.double_clicked_by(egui::PointerButton::Primary) {
            return EditingName { rect: inner.rect }.into();
        }
        if let Some(block) = self.rect(inner.rect)
            && let Some(pos) = response.hover_pos()
            && let Shape::Block(block) = &block
            && block.title_bbox().contains(pos)
        {
            return inner.into();
        }
        Selected { rect: inner.rect }.into()
    }
    fn handle_title_control_dragged(
        &mut self,
        inner: TitleControlDragged,
        response: Response,
    ) -> State {
        let TitleControlDragged { rect, delta_pos } = inner;
        if let Some(pos) = response.interact_pointer_pos()
            && let Some(rbox) = self.rect_mut(rect)
        {
            let center_line = rbox.gui_rect().center().y;
            if let Some(title_ref) = rbox.title_mut() {
                if pos.y < center_line {
                    title_ref.side = TitleSide::Top;
                } else {
                    title_ref.side = TitleSide::Bottom;
                }
            }
        }
        if response.dragged_by(egui::PointerButton::Primary) {
            let delta = response.drag_delta();
            return TitleControlDragged {
                rect,
                delta_pos: delta_pos + delta,
            }
            .into();
        } else if (response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged())
            && let Some(bbox) = self.rect_mut(inner.rect)
            && let Some(title) = bbox.title_mut()
        {
            title.offset += delta_pos.x;
            return Selected { rect: inner.rect }.into();
        }
        inner.into()
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
            let raw_delta = response.drag_delta();
            let constrained_delta = self
                .rect(rect)
                .map(|b| b.constrain_resize_delta(raw_delta))
                .unwrap_or(raw_delta);
            return ResizingRect {
                rect,
                mode,
                delta_pos: delta_pos + constrained_delta,
            }
            .into();
        } else if (response.drag_stopped_by(egui::PointerButton::Primary) || !response.dragged())
            && let Some(bbox) = self.rect_mut(rect)
        {
            let new_rect = grid_rect(resize_rect(&bbox.gui_rect(), mode, delta_pos));
            bbox.apply_resize(mode, new_rect);
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
            if let Some(route) = self.auto_routes.get_mut(target.id) {
                route.update_waypoint(target.start_waypoint, |wp| wp.pos += delta);
                route.update_waypoint(target.end_waypoint, |wp| wp.pos += delta);
                self.reroute = true;
                self.ripup_set.push(target.id);
                return target.into();
            }
            return self.handle_route_hover_check(target.id, response);
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
            if let Some(pin_ref) = rbox.pins_mut(pin) {
                if pos.x < center_line {
                    pin_ref.side = PinSide::West;
                } else {
                    pin_ref.side = PinSide::East;
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
            if let Some(tail) = self.find_anchor(|anchor, anchor_pos| {
                (anchor_pos.distance(pos) < PORT_RADIUS).then_some(anchor)
            }) && tail != auto_route.start
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
            if let Some(tail) = self.find_anchor(|anchor, anchor_pos| {
                (anchor_pos.distance(pos) < PORT_RADIUS).then_some(anchor)
            }) && tail != proposed_route.start
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
            State::TitleControlHovered(inner) => self.handle_title_control_hovered(inner, response),
            State::TitleControlDragged(inner) => self.handle_title_control_dragged(inner, response),
            State::TitleHovered(inner) => self.handle_title_hovered(inner, response),
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
            rect_box.with_pins(|pid, pin| {
                let Some(anchor_pos) = rect_box.anchor_point_with_rect(effective_rect, pid) else {
                    return;
                };
                let anchor_pos = match pin.side {
                    PinSide::East => anchor_pos + vec2(GRID_SIZE, 0.0),
                    PinSide::West => anchor_pos - vec2(GRID_SIZE, 0.0),
                };
                builder.add_h_channel(anchor_pos, COST_ZERO);
            });
        }
        builder.build()
    }
    fn update_graph(&mut self) {
        let mut router = self.build_router();
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
        self.debug_marks = router.debug_marks();
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
    let box1_pin1 = drawing
        .rect_mut(box1_id)
        .and_then(|ps| {
            ps.add_pin(
                "i.1.write_logic".to_string(),
                PinSide::West,
                GRID_SIZE * 1.0,
            )
        })
        .expect("add_pin");
    let box1_anchor1 = LineAnchor {
        rect: box1_id,
        pin: box1_pin1,
    };
    let box1_pin2 = drawing
        .rect_mut(box1_id)
        .and_then(|ps| {
            ps.add_pin(
                "i.0.write_logic".to_string(),
                PinSide::West,
                GRID_SIZE * 2.0,
            )
        })
        .expect("add_pin");
    let box1_anchor2 = LineAnchor {
        rect: box1_id,
        pin: box1_pin2,
    };
    let origin_2 = pos2(0.0, 0.0);
    let box2_id = drawing.add_rect_box(origin_2, origin_2 + size);
    let box2_port1 = drawing
        .rect_mut(box2_id)
        .and_then(|ps| ps.add_pin("o.1.read_logic".to_string(), PinSide::East, GRID_SIZE * 1.0))
        .expect("add_pin");
    let box2_anchor1 = LineAnchor {
        rect: box2_id,
        pin: box2_port1,
    };
    let box2_pin2 = drawing
        .rect_mut(box2_id)
        .and_then(|ps| ps.add_pin("o.0.read_logic".to_string(), PinSide::East, GRID_SIZE * 2.0))
        .expect("add_pin");
    let box2_anchor2 = LineAnchor {
        rect: box2_id,
        pin: box2_pin2,
    };
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
