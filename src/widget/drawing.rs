use egui::{
    Align2, Color32, Pos2, Rect, Response, Stroke, StrokeKind, TextEdit, Ui, Vec2,
    epaint::TextShape, pos2, vec2,
};

use crate::{
    grid::{
        GRID_SIZE, MOVE_HOVER_DISTANCE, PORT_HEIGHT, PORT_RADIUS, ROUTE_TEXT_SIZE, SHIM, grid_rect,
        round_to_grid, snap_to_grid,
    },
    router::{RouterNG, RouterNGBuilder, WIRE_COST, cost::COST_ZERO},
    state::*,
    store::*,
    theme::get_theme,
    turtle::Mark,
    widget::{
        auto_route::{AddTextButton, AutoRoute},
        block::{TitleSide, control_corner, resize_rect},
        data::Data,
        direction::RouteDirection,
        pin::PinSide,
        render::{
            FocusResult, estimate_bbox_for_pin_text, get_control_pin_bbox, get_hamburger_rect,
            render_path_with_chamfered_corners,
        },
        shape::{BaseShape, Shape},
        tool::{
            new_block::NewBlock,
            new_pin::NewPin,
            route::Route,
            select::{SubtoolState, drag_pin::DragPin, rename_pin::RenamePin},
        },
        waypoint::Waypoint,
    },
};

const GRIP_SHIM: f32 = 4.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Mode {
    Move,
    #[default]
    Select,
    Block,
    Pin,
    Route,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LineAnchor {
    pub rect: RectId,
    pub pin: PinId,
}

#[derive(Default)]
pub struct Drawing {
    data: Data,
    state: State,
    debug_marks: Vec<Mark>,
    pub mode: Mode,
    new_block: NewBlock,
    route: Route,
    new_pin: NewPin,
    drag_pin: Option<DragPin>,
    rename_pin: Option<RenamePin>,
}

enum RouteRenderMode {
    Normal,
    Highlighted,
    Selected,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Event {
    HoverAt(Pos2),
    DragStarted { pos: Pos2 },
    Dragging { pos: Pos2, delta: Vec2 },
    DragStopped,
    Clicked { pos: Pos2 },
    DoubleClicked { pos: Pos2 },
}

fn compute_event(response: &Response) -> Option<Event> {
    if response.double_clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        Some(Event::DoubleClicked { pos })
    } else if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        Some(Event::Clicked { pos })
    } else if response.drag_started()
        && let Some(pos) = response.interact_pointer_pos()
    {
        Some(Event::DragStarted { pos })
    } else if response.dragged()
        && let Some(pos) = response.interact_pointer_pos()
    {
        Some(Event::Dragging {
            pos,
            delta: response.drag_delta(),
        })
    } else if response.drag_stopped() {
        Some(Event::DragStopped)
    } else if response.hovered()
        && let Some(pos) = response.hover_pos()
    {
        Some(Event::HoverAt(pos))
    } else {
        None
    }
}

enum Action {
    TransitionTo(State),
    TransitionAndUpdate(State),
    MoveRect {
        inner: MovingRect,
        next: State,
    },
    ResizeRect {
        inner: ResizingRect,
        next: State,
    },
    AllocateLabelAndEdit {
        route: RouteId,
        pos: Pos2,
    },
    StartRouteCornerDrag {
        waypoint_id: WaypointId,
        route: RouteId,
    },
    AddCornerWaypointAndDrag {
        route: RouteId,
        pos: Pos2,
    },
    StartEdgeDrag {
        route: RouteId,
        edge_id: EdgeId,
    },
    AddRouteText {
        route: RouteId,
        button: AddTextButton,
    },
    MoveTitle {
        rect: RectId,
        offset: f32,
        side: TitleSide,
    },
    ShiftRouteLabel {
        route: RouteId,
        label_id: WireLabelId,
        linear_distance_delta: f32,
    },
    DragEdge {
        target: RouteEdgeDragged,
        delta: Vec2,
    },
    FinalizeEdgeDrag {
        route: RouteId,
        start_waypoint: WaypointId,
        end_waypoint: WaypointId,
    },
    FinalizeRouteLabelEdit {
        route: RouteId,
    },
    DragWaypoint {
        inner: WaypointDragged,
        delta: Vec2,
    },
    FinalizeWaypointDrag {
        route: RouteId,
    },
    FinalizeTextAnchorDrag {
        route: RouteId,
    },
}

impl Drawing {
    pub fn routing_box(&self, id: RectId) -> Option<Rect> {
        let rect = self.data.rect(id)?.gui_rect();
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
        /*         if let State::PinDragged(inner) = &self.state
            && anchor.rect == inner.rect
            && anchor.pin == inner.pin
        {
            let center_line = effective_rect.center().x;
            let rect = self.data.rect(anchor.rect)?;
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
        */
        self.data
            .rect(anchor.rect)
            .and_then(|rect| rect.anchor_point_with_rect(effective_rect, anchor.pin))
        //        }
    }
    pub fn with_anchors(&self, mut f: impl FnMut(LineAnchor)) {
        self.data.rect_boxes().for_each(|(rect_id, rect)| {
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
        let primary_button_down = ui.input(|i| i.pointer.any_down());
        ui.output_mut(|o| {
            o.cursor_icon = if self.mode == Mode::Block || self.mode == Mode::Pin {
                egui::CursorIcon::Crosshair
            } else if self.mode == Mode::Move {
                if matches!(self.state, State::Panning) || primary_button_down {
                    egui::CursorIcon::Grabbing
                } else {
                    egui::CursorIcon::Grab
                }
            } else {
                self.state.cursor()
            };
        });
        (-100..=100).map(|y| y as f32 * GRID_SIZE).for_each(|h| {
            ui.painter()
                .hline(-10_000.0f32..=10_000.0f32, h, (0.15, theme.grid_line));
        });
        (-100..=100).map(|x| x as f32 * GRID_SIZE).for_each(|v| {
            ui.painter()
                .vline(v, -10_000.0f32..=10_000.0f32, (0.15, theme.grid_line));
        });
        //        crate::turtle::draw(&self.debug_marks, ui.painter());
        for (_, route) in self.data.auto_routes() {
            self.render_route(ui, &route, RouteRenderMode::Normal);
        }
        for (id, rect_box) in self.data.rect_boxes_mut() {
            let mode = self.state.render_mode_for_id(id);
            if rect_box.render(mode, ui) == FocusResult::LostFocus {
                self.state = Selected { rect: id }.into();
            }
        }
        self.new_block.render(ui);
        self.route.render(&self.data, ui);
        self.new_pin.render(ui);
        if let Some(pin_edit) = self.drag_pin.as_mut() {
            pin_edit.render(&mut self.data, ui);
        }
        if let Some(rename_pin) = self.rename_pin.as_mut() {
            rename_pin.render(&mut self.data, ui);
        }
        if let State::RouteHovered(target) = &self.state
            && let Some(route) = self.data.auto_route(target.id)
        {
            self.render_route(ui, route, RouteRenderMode::Highlighted);
        }
        if let State::RouteSelected(target) = &self.state
            && let Some(route) = self.data.auto_route(target.id)
        {
            self.render_route(ui, route, RouteRenderMode::Selected);
        }
        if let State::RouteEdgeHovered(target) = &self.state
            && let Some(route) = self.data.auto_route(target.id)
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
            && let Some(route) = self.data.auto_route(target.id)
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
            && let Some(route) = self.data.auto_route(target.id)
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
            && let Some(route) = self.data.auto_route(target.route)
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
            && let Some(route) = self.data.auto_route(target.route)
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
            && let Some(route) = self.data.auto_route(target.route)
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
            && let Some(route) = self.data.auto_route(target.route)
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
            && let Some(route) = self.data.auto_route_mut(target.id)
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
    fn handle_route_hover_check(&self, id: RouteId, event: Event) -> Action {
        if let Event::HoverAt(hover_pos) = event
            && let Some(route) = self.data.auto_route(id)
        {
            for (waypoint_id, waypoint) in route.iter_waypoints() {
                if waypoint.pos.distance(hover_pos) <= PORT_RADIUS * 1.5 {
                    return Action::TransitionTo(
                        WaypointHovered {
                            route: id,
                            waypoint: waypoint_id,
                        }
                        .into(),
                    );
                }
            }
            if let Some((edge_1, edge_2)) = route.hovered_corner(hover_pos) {
                return Action::TransitionTo(RouteCornerHovered { id, edge_1, edge_2 }.into());
            }
            if let Some(edge_id) = route.hovered_edge(hover_pos)
                && let Some(edge) = route.edge(edge_id)
                && hover_pos.distance(edge.center()) <= PORT_RADIUS * 1.5
            {
                return Action::TransitionTo(
                    RouteEdgeHovered {
                        id,
                        edge_index: edge_id,
                        direction: edge.direction(),
                    }
                    .into(),
                );
            }
            if let Some(label_id) = route.hit_text_anchor(hover_pos) {
                return Action::TransitionTo(
                    TextAnchorHovered {
                        route: id,
                        label_id,
                    }
                    .into(),
                );
            }
            if let Some(button) = route.hit_add_text_button(hover_pos) {
                return Action::TransitionTo(
                    AddTextButtonHovered {
                        route: id,
                        button: button.clone(),
                    }
                    .into(),
                );
            }
        }
        Action::TransitionTo(RouteSelected { id }.into())
    }
    fn handle_add_text(&self, event: Event) -> Action {
        if let Event::HoverAt(pos) = event {
            for (id, route) in self.data.auto_routes() {
                if let Some(edge_id) = route.hovered_edge(pos) {
                    return Action::TransitionTo(
                        AddTextHoveredRoute {
                            route: id,
                            edge_id,
                            pos,
                        }
                        .into(),
                    );
                }
            }
        }
        Action::TransitionTo(State::AddText)
    }
    fn handle_add_text_hovered_route(&self, inner: AddTextHoveredRoute, event: Event) -> Action {
        match event {
            Event::Clicked { pos } => Action::AllocateLabelAndEdit {
                route: inner.route,
                pos,
            },
            Event::HoverAt(pos) => {
                if let Some(route) = self.data.auto_route(inner.route)
                    && let Some(edge_id) = route.hovered_edge(pos)
                {
                    Action::TransitionTo(
                        AddTextHoveredRoute {
                            route: inner.route,
                            edge_id,
                            pos,
                        }
                        .into(),
                    )
                } else {
                    Action::TransitionTo(State::AddText)
                }
            }
            _ => Action::TransitionTo(State::AddText),
        }
    }
    fn handle_idle_state(&self, event: Event) -> Action {
        match event {
            Event::DragStarted { pos } => self.drag_start_on_canvas(pos),
            Event::Clicked { pos } => {
                if let Some(id) = self.rect_at(pos) {
                    return Action::TransitionTo(Selected { rect: id }.into());
                }
                if let Some(id) = self.route_hit(pos) {
                    return Action::TransitionTo(RouteSelected { id }.into());
                }
                Action::TransitionTo(State::Idle)
            }
            Event::HoverAt(hover_pos) if self.mode == Mode::Select => {
                if let Some(id) = self.route_hit(hover_pos) {
                    return Action::TransitionTo(RouteHovered { id }.into());
                } else {
                    Action::TransitionTo(State::Idle)
                }
            }
            _ => Action::TransitionTo(State::Idle),
        }
    }
    fn route_hit(&self, pos: Pos2) -> Option<RouteId> {
        for (id, route) in self.data.auto_routes() {
            if route.hovered_edge(pos).is_some() {
                return Some(id);
            }
        }
        None
    }
    fn rect_at(&self, pos: Pos2) -> Option<RectId> {
        self.data
            .rect_boxes()
            .find(|(_, r)| r.gui_rect().contains(pos))
            .map(|(id, _)| id)
    }
    fn drag_start_on_canvas(&self, pos: Pos2) -> Action {
        if self.mode == Mode::Move {
            Action::TransitionTo(State::panning())
        } else if let Some(id) = self.rect_at(pos) {
            Action::TransitionTo(
                MovingRect {
                    rect: id,
                    delta_pos: Vec2::ZERO,
                }
                .into(),
            )
        } else {
            Action::TransitionTo(State::Idle)
        }
    }
    fn handle_selected_state(&mut self, inner: Selected, event: Event) -> Action {
        let rect = inner.rect;
        match event {
            Event::DoubleClicked { pos } => {
                if let Some(rect_box) = self.data.rect(rect) {
                    if let Some(state) = rect_box.find_pin(|pid, pin| {
                        let pin_bbox =
                            estimate_bbox_for_pin_text(rect_box.gui_rect(), pin).expand(3.0);
                        if pin_bbox.contains(pos) {
                            Some(LineAnchor { rect, pin: pid }.into())
                        } else {
                            None
                        }
                    }) {
                        self.rename_pin = Some(state);
                        return Action::TransitionTo(Selected { rect }.into());
                    }
                }

                if let Some(rect_box) = self.data.rect(rect)
                    && let Shape::Block(block) = rect_box
                    && block.title_bbox().contains(pos)
                {
                    return Action::TransitionTo(EditingName { rect }.into());
                }
            }
            Event::Clicked { pos } => {
                if let Some(id) = self.rect_at(pos) {
                    return Action::TransitionTo(Selected { rect: id }.into());
                }
                return Action::TransitionTo(State::idle());
            }
            Event::DragStarted { pos } => {
                eprintln!("Drag started at {pos}");
                if let Some(hbox) = self.data.rect(rect)
                    && let Some(lid) = hbox.find_pin(|lid, pin| {
                        let pin_bbox = estimate_bbox_for_pin_text(hbox.gui_rect(), pin).expand(3.0);
                        eprintln!(
                            "Checking pin {} with bbox {} against pos {pos}",
                            lid, pin_bbox
                        );
                        if pin_bbox.contains(pos) {
                            Some(lid)
                        } else {
                            None
                        }
                    })
                {
                    eprintln!("Starting to drag pin {}", lid);
                    self.drag_pin = Some(LineAnchor { rect, pin: lid }.into());
                    return Action::TransitionTo(Selected { rect }.into());
                }
                return self.drag_start_on_canvas(pos);
            }
            Event::HoverAt(hover_pos) => {
                if let Some(bbox) = self.data.rect(rect) {
                    for &mode in bbox.resize_modes() {
                        if hover_pos.distance(control_corner(&bbox.gui_rect(), mode))
                            < MOVE_HOVER_DISTANCE
                        {
                            return Action::TransitionTo(PotentialResize { rect, mode }.into());
                        }
                    }
                    if let Some(anchor) = bbox.title_anchor()
                        && hover_pos.distance(anchor) < MOVE_HOVER_DISTANCE
                    {
                        return Action::TransitionTo(TitleControlHovered { rect }.into());
                    }
                    if let Shape::Block(block) = &bbox
                        && block.title_bbox().contains(hover_pos)
                    {
                        return Action::TransitionTo(TitleHovered { rect }.into());
                    }
                }
            }
            _ => {}
        }
        Action::TransitionTo(Selected { rect }.into())
    }
    fn handle_potential_resize(&self, inner: PotentialResize, event: Event) -> Action {
        let PotentialResize { rect, mode } = inner;
        match event {
            Event::HoverAt(hover_pos) => {
                if let Some(bbox) = self.data.rect(rect)
                    && hover_pos.distance(control_corner(&bbox.gui_rect(), mode))
                        >= MOVE_HOVER_DISTANCE
                {
                    return Action::TransitionTo(Selected { rect }.into());
                }
            }
            Event::DragStarted { .. } => {
                return Action::TransitionTo(
                    ResizingRect {
                        rect,
                        mode,
                        delta_pos: Vec2::ZERO,
                    }
                    .into(),
                );
            }
            _ => {}
        }
        Action::TransitionTo(PotentialResize { rect, mode }.into())
    }
    fn handle_title_control_hovered(&self, inner: TitleControlHovered, event: Event) -> Action {
        let TitleControlHovered { rect } = inner;
        match event {
            Event::HoverAt(hover_pos) => {
                if let Some(bbox) = self.data.rect(rect)
                    && let Some(title_anchor) = bbox.title_anchor()
                    && hover_pos.distance(title_anchor) >= MOVE_HOVER_DISTANCE
                {
                    return Action::TransitionTo(Selected { rect }.into());
                }
            }
            Event::DragStarted { .. } => {
                return Action::TransitionTo(
                    TitleControlDragged {
                        rect,
                        delta_pos: Vec2::ZERO,
                    }
                    .into(),
                );
            }
            _ => {}
        }
        Action::TransitionTo(TitleControlHovered { rect }.into())
    }

    // fn handle_pin_label_hovered(&self, inner: PinLabelHovered, event: Event) -> Action {
    //     let PinLabelHovered { rect, pin } = inner;
    //     match event {
    //         Event::HoverAt(hover_pos) => {
    //             if let Some(bbox) = self.data.rect(rect)
    //                 && let Some(pin_ref) = bbox.pin(pin)
    //             {
    //                 let pin_bbox = estimate_bbox_for_pin_text(bbox.gui_rect(), pin_ref);
    //                 if !pin_bbox.contains(hover_pos) {
    //                     return Action::TransitionTo(Selected { rect }.into());
    //                 }
    //             }
    //         }
    //         Event::DoubleClicked { .. } => {
    //             return Action::TransitionTo(EditingPinText { rect, pin }.into());
    //         }
    //         _ => {}
    //     }
    //     Action::TransitionTo(PinLabelHovered { rect, pin }.into())
    // }
    fn handle_route_label_hovered(&self, route: RouteLabelHovered, event: Event) -> Action {
        if let Event::HoverAt(hover_pos) = event
            && let Some(auto_route) = self.data.auto_route(route.id)
        {
            let Some(edge_index) = auto_route.hovered_edge(hover_pos) else {
                return Action::TransitionTo(State::Idle);
            };
            if edge_index != route.edge_index {
                return Action::TransitionTo(State::Idle);
            }
        }
        Action::TransitionTo(route.into())
    }
    // fn handle_pin_label_grip_hovered(&self, inner: PinLabelGripHovered, event: Event) -> Action {
    //     let PinLabelGripHovered { rect, pin } = inner;
    //     match event {
    //         Event::DragStarted { .. } | Event::Dragging { .. } => {
    //             eprintln!("Starting to drag port label grip");
    //             return Action::TransitionTo(
    //                 PinDragged {
    //                     rect,
    //                     pin,
    //                     delta_pos: Vec2::ZERO,
    //                 }
    //                 .into(),
    //             );
    //         }
    //         Event::HoverAt(hover_pos) => {
    //             if let Some(bbox) = self.data.rect(rect)
    //                 && let Some(pin_ref) = bbox.pin(pin)
    //             {
    //                 let hamburger_rect =
    //                     get_hamburger_rect(bbox.gui_rect(), pin_ref).expand(GRIP_SHIM);
    //                 if !hamburger_rect.contains(hover_pos) {
    //                     return Action::TransitionTo(Selected { rect }.into());
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }
    //     Action::TransitionTo(PinLabelGripHovered { rect, pin }.into())
    // }
    fn handle_route_corner_hovered(&self, inner: RouteCornerHovered, event: Event) -> Action {
        match event {
            Event::DragStarted { pos } | Event::Dragging { pos, .. } => {
                eprintln!("Starting to drag route corner");
                if let Some(route) = self.data.auto_route(inner.id) {
                    if let Some(waypoint_id) = route.hit_waypoint(pos, PORT_RADIUS) {
                        return Action::StartRouteCornerDrag {
                            waypoint_id,
                            route: inner.id,
                        };
                    }
                    if route.edge(inner.edge_1).is_some() {
                        return Action::AddCornerWaypointAndDrag {
                            route: inner.id,
                            pos,
                        };
                    }
                }
            }
            _ => {}
        }
        self.handle_route_hover_check(inner.id, event)
    }
    fn handle_route_hovered(&self, _inner: RouteHovered, event: Event) -> Action {
        match event {
            Event::Clicked { pos } => {
                if let Some(id) = self.route_hit(pos) {
                    return Action::TransitionTo(RouteSelected { id }.into());
                }
            }
            Event::HoverAt(hover_pos) => {
                if let Some(id) = self.route_hit(hover_pos) {
                    return Action::TransitionTo(RouteHovered { id }.into());
                }
            }
            _ => {}
        }
        Action::TransitionTo(State::Idle)
    }
    fn handle_route_selected(&self, inner: RouteSelected, event: Event) -> Action {
        if matches!(event, Event::Clicked { .. }) {
            return Action::TransitionTo(State::Idle);
        }
        self.handle_route_hover_check(inner.id, event)
    }
    fn handle_route_edge_hovered(&self, target: RouteEdgeHovered, event: Event) -> Action {
        match event {
            Event::DragStarted { .. } | Event::Dragging { .. } => {
                if self
                    .data
                    .auto_route(target.id)
                    .and_then(|r| r.edge(target.edge_index))
                    .is_some()
                {
                    return Action::StartEdgeDrag {
                        route: target.id,
                        edge_id: target.edge_index,
                    };
                }
            }
            _ => {}
        }
        self.handle_route_hover_check(target.id, event)
    }
    fn handle_add_text_button_hovered(&self, inner: AddTextButtonHovered, event: Event) -> Action {
        if matches!(event, Event::Clicked { .. }) {
            return Action::AddRouteText {
                route: inner.route,
                button: inner.button,
            };
        }
        self.handle_route_hover_check(inner.route, event)
    }
    fn handle_text_anchor_hovered(&self, inner: TextAnchorHovered, event: Event) -> Action {
        match event {
            Event::DragStarted { .. } | Event::Dragging { .. } => {
                eprintln!("Starting to drag text anchor");
                return Action::TransitionTo(
                    TextAnchorDragged {
                        route: inner.route,
                        label_id: inner.label_id,
                        delta_pos: Vec2::ZERO,
                    }
                    .into(),
                );
            }
            Event::DoubleClicked { .. } => {
                return Action::TransitionTo(
                    EditingRouteLabelText {
                        id: inner.route,
                        label_id: inner.label_id,
                    }
                    .into(),
                );
            }
            _ => {}
        }
        self.handle_route_hover_check(inner.route, event)
    }
    fn handle_title_hovered(&self, inner: TitleHovered, event: Event) -> Action {
        match event {
            Event::DoubleClicked { .. } => {
                return Action::TransitionTo(EditingName { rect: inner.rect }.into());
            }
            Event::HoverAt(pos) => {
                if let Some(block) = self.data.rect(inner.rect)
                    && let Shape::Block(block) = &block
                    && block.title_bbox().contains(pos)
                {
                    return Action::TransitionTo(inner.into());
                }
            }
            _ => {}
        }
        Action::TransitionTo(Selected { rect: inner.rect }.into())
    }
    fn handle_title_control_dragged(&self, inner: TitleControlDragged, event: Event) -> Action {
        let TitleControlDragged { rect, delta_pos } = inner;
        match event {
            Event::Dragging { delta, .. } => Action::TransitionTo(
                TitleControlDragged {
                    rect,
                    delta_pos: delta_pos + delta,
                }
                .into(),
            ),
            _ => {
                let side = if let Some(Shape::Block(block)) = self.data.rect(rect) {
                    block.title().map(|t| t.side).unwrap_or(TitleSide::Top)
                } else {
                    TitleSide::Top
                };
                Action::MoveTitle {
                    rect,
                    offset: delta_pos.x,
                    side,
                }
            }
        }
    }

    fn handle_text_anchor_dragged(&self, inner: TextAnchorDragged, event: Event) -> Action {
        match event {
            Event::Dragging { delta, .. } => {
                let linear_distance_delta = self
                    .data
                    .auto_route(inner.route)
                    .and_then(|route| {
                        let label_distance = route.label(inner.label_id)?.linear_distance;
                        let (direction, flip_sign) =
                            if let Some((_, edge)) = route.find_edge(label_distance) {
                                let dir = edge.direction();
                                let flip = match dir {
                                    RouteDirection::Horizontal => edge.start.x > edge.end.x,
                                    RouteDirection::Vertical => edge.start.y > edge.end.y,
                                };
                                (dir, flip)
                            } else {
                                (RouteDirection::Horizontal, false)
                            };
                        let d = match direction {
                            RouteDirection::Horizontal => {
                                if flip_sign {
                                    -delta.x
                                } else {
                                    delta.x
                                }
                            }
                            RouteDirection::Vertical => {
                                if flip_sign {
                                    -delta.y
                                } else {
                                    delta.y
                                }
                            }
                        };
                        Some(d)
                    })
                    .unwrap_or(0.0);
                Action::ShiftRouteLabel {
                    route: inner.route,
                    label_id: inner.label_id,
                    linear_distance_delta,
                }
            }
            _ => Action::FinalizeTextAnchorDrag { route: inner.route },
        }
    }
    fn handle_waypoint_hovered(&self, inner: WaypointHovered, event: Event) -> Action {
        match event {
            Event::DragStarted { .. } | Event::Dragging { .. } => {
                eprintln!("Starting to drag waypoint");
                return Action::StartRouteCornerDrag {
                    waypoint_id: inner.waypoint,
                    route: inner.route,
                };
            }
            _ => {}
        }
        self.handle_route_hover_check(inner.route, event)
    }
    fn handle_waypoint_dragged(&self, inner: WaypointDragged, event: Event) -> Action {
        match event {
            Event::Dragging { delta, .. } => Action::DragWaypoint { inner, delta },
            _ => Action::FinalizeWaypointDrag { route: inner.route },
        }
    }
    fn handle_resizing_rect(&self, inner: ResizingRect, event: Event) -> Action {
        let ResizingRect {
            rect,
            mode,
            delta_pos,
        } = inner;
        match event {
            Event::Dragging {
                delta: raw_delta, ..
            } => {
                let constrained_delta = self
                    .data
                    .rect(rect)
                    .map(|b| b.constrain_resize_delta(raw_delta))
                    .unwrap_or(raw_delta);
                Action::TransitionAndUpdate(
                    ResizingRect {
                        rect,
                        mode,
                        delta_pos: delta_pos + constrained_delta,
                    }
                    .into(),
                )
            }
            _ => Action::ResizeRect {
                inner: ResizingRect {
                    rect,
                    mode,
                    delta_pos,
                },
                next: Selected { rect }.into(),
            },
        }
    }
    fn handle_moving_rect(&self, inner: MovingRect, event: Event) -> Action {
        let MovingRect { rect, delta_pos } = inner;
        match event {
            Event::Dragging { delta, .. } => Action::TransitionAndUpdate(
                MovingRect {
                    rect,
                    delta_pos: delta_pos + delta,
                }
                .into(),
            ),
            _ => Action::MoveRect {
                inner: MovingRect { rect, delta_pos },
                next: Selected { rect }.into(),
            },
        }
    }
    fn handle_panning(&self, event: Event) -> Action {
        if matches!(event, Event::DragStopped) {
            return Action::TransitionTo(State::idle());
        }
        Action::TransitionTo(State::panning())
    }
    fn handle_editing_name(&self, inner: EditingName, event: Event) -> Action {
        let rect = inner.rect;
        if matches!(event, Event::Clicked { .. }) {
            return Action::TransitionTo(Selected { rect }.into());
        }
        Action::TransitionTo(EditingName { rect }.into())
    }
    fn handle_editing_route_label_text(
        &self,
        inner: EditingRouteLabelText,
        event: Event,
    ) -> Action {
        if matches!(event, Event::Clicked { .. }) {
            return Action::FinalizeRouteLabelEdit { route: inner.id };
        }
        Action::TransitionTo(inner.into())
    }
    fn handle_route_edge_dragged(&self, target: RouteEdgeDragged, event: Event) -> Action {
        match event {
            Event::Dragging { delta, .. } => {
                let constrained_delta = if target.direction == RouteDirection::Horizontal {
                    vec2(0.0, delta.y)
                } else {
                    vec2(delta.x, 0.0)
                };
                Action::DragEdge {
                    target,
                    delta: constrained_delta,
                }
            }
            _ => Action::FinalizeEdgeDrag {
                route: target.id,
                start_waypoint: target.start_waypoint,
                end_waypoint: target.end_waypoint,
            },
        }
    }
    // fn handle_pin_dragged(&self, inner: PinDragged, event: Event) -> Action {
    //     let PinDragged {
    //         rect,
    //         pin,
    //         delta_pos,
    //     } = inner;
    //     match event {
    //         Event::Dragging { pos, delta } => {
    //             let side = self
    //                 .data
    //                 .rect(rect)
    //                 .map(|rbox| {
    //                     if pos.x < rbox.gui_rect().center().x {
    //                         PinSide::West
    //                     } else {
    //                         PinSide::East
    //                     }
    //                 })
    //                 .unwrap_or(PinSide::East);
    //             Action::MovePin {
    //                 rect,
    //                 pin,
    //                 side,
    //                 delta_pos: delta_pos + delta,
    //             }
    //         }
    //         _ => Action::FinalizePinDrag {
    //             rect,
    //             pin,
    //             delta_y: delta_pos.y,
    //         },
    //     }
    // }
    fn apply_action(&mut self, action: Action) {
        match action {
            Action::TransitionTo(state) => {
                self.state = state;
            }
            Action::TransitionAndUpdate(state) => {
                self.state = state;
                self.update_graph(&[]);
            }
            Action::MoveRect { inner, next } => {
                if let Some(bbox) = self.data.rect_mut(inner.rect) {
                    *bbox.gui_rect_mut() = grid_rect(bbox.gui_rect().translate(inner.delta_pos));
                }
                self.state = next;
                self.update_graph(&[]);
            }
            Action::ResizeRect { inner, next } => {
                if let Some(bbox) = self.data.rect_mut(inner.rect) {
                    let new_rect =
                        grid_rect(resize_rect(&bbox.gui_rect(), inner.mode, inner.delta_pos));
                    bbox.apply_resize(inner.mode, new_rect);
                }
                self.state = next;
                self.update_graph(&[]);
            }
            Action::AddRouteText {
                route: route_id,
                button,
            } => {
                self.state = if let Some(route) = self.data.auto_route_mut(route_id) {
                    let loc = route.map_linear_distance_to_position(button.linear_position);
                    let label_id = route.allocate_label(loc.location);
                    EditingRouteLabelText {
                        id: route_id,
                        label_id,
                    }
                    .into()
                } else {
                    State::Idle
                };
            }
            Action::MoveTitle { rect, offset, side } => {
                if let Some(bbox) = self.data.rect_mut(rect)
                    && let Some(title) = bbox.title_mut()
                {
                    title.offset += offset;
                    title.side = side;
                }
                self.state = Selected { rect }.into();
            }
            Action::StartRouteCornerDrag { waypoint_id, route } => {
                if let Some(r) = self.data.auto_route_mut(route) {
                    r.lock_waypoint(waypoint_id);
                }
                self.state = WaypointDragged {
                    route,
                    waypoint: waypoint_id,
                    delta_pos: Vec2::ZERO,
                }
                .into();
            }
            Action::AddCornerWaypointAndDrag { route, pos } => {
                self.state = if let Some(r) = self.data.auto_route_mut(route) {
                    let waypoint_id = r.add_waypoint(snap_to_grid(pos));
                    r.lock_waypoint(waypoint_id);
                    WaypointDragged {
                        route,
                        waypoint: waypoint_id,
                        delta_pos: Vec2::ZERO,
                    }
                    .into()
                } else {
                    State::Idle
                };
            }
            Action::StartEdgeDrag {
                route: route_id,
                edge_id,
            } => {
                if let Some(route) = self.data.auto_route_mut(route_id)
                    && let Some(edge) = route.edge(edge_id).cloned()
                {
                    eprintln!("Starting to drag route edge");
                    eprintln!("Raw edge: {:?}", edge);
                    let wp1 = route.add_waypoint(edge.waypoint_position_start());
                    let wp2 = route.add_waypoint(edge.waypoint_position_end());
                    route.lock_waypoint(wp1);
                    route.lock_waypoint(wp2);
                    self.state = if wp1 == wp2 {
                        WaypointDragged {
                            route: route_id,
                            waypoint: wp1,
                            delta_pos: Vec2::ZERO,
                        }
                        .into()
                    } else {
                        RouteEdgeDragged {
                            id: route_id,
                            direction: edge.direction(),
                            start_waypoint: wp1,
                            end_waypoint: wp2,
                            delta_pos: Vec2::ZERO,
                        }
                        .into()
                    };
                    self.update_graph(&[route_id]);
                } else {
                    self.state = State::Idle;
                }
            }
            Action::DragEdge { target, delta } => {
                if let Some(route) = self.data.auto_route_mut(target.id) {
                    route.update_waypoint(target.start_waypoint, |wp| wp.pos += delta);
                    route.update_waypoint(target.end_waypoint, |wp| wp.pos += delta);
                    let id = target.id;
                    self.state = State::RouteEdgeDragged(target);
                    self.update_graph(&[id]);
                } else {
                    self.state = RouteSelected { id: target.id }.into();
                }
            }
            Action::FinalizeEdgeDrag {
                route,
                start_waypoint,
                end_waypoint,
            } => {
                if let Some(r) = self.data.auto_route_mut(route) {
                    if let Some(wp) = r.waypoint_mut(start_waypoint) {
                        wp.pos = snap_to_grid(wp.pos);
                        wp.unlock();
                    }
                    if let Some(wp) = r.waypoint_mut(end_waypoint) {
                        wp.pos = snap_to_grid(wp.pos);
                        wp.unlock();
                    }
                }
                self.state = RouteSelected { id: route }.into();
                self.update_graph(&[]);
            }
            Action::FinalizeRouteLabelEdit { route } => {
                if let Some(r) = self.data.auto_route_mut(route) {
                    r.update_waypoints();
                }
                self.state = State::Idle;
            }
            Action::DragWaypoint { inner, delta } => {
                if let Some(route) = self.data.auto_route_mut(inner.route) {
                    if let Some(wp) = route.waypoint_mut(inner.waypoint) {
                        wp.pos += delta;
                    }
                    let route_id = inner.route;
                    self.state = inner.into();
                    self.update_graph(&[route_id]);
                } else {
                    self.state = inner.into();
                }
            }
            Action::FinalizeWaypointDrag { route } => {
                if let Some(r) = self.data.auto_route_mut(route) {
                    r.iter_waypoints_mut().for_each(|(_, wp)| {
                        wp.pos = snap_to_grid(wp.pos);
                        wp.unlock();
                    });
                }
                self.state = RouteSelected { id: route }.into();
                self.update_graph(&[]);
            }
            Action::ShiftRouteLabel {
                route,
                label_id,
                linear_distance_delta,
            } => {
                if let Some(r) = self.data.auto_route_mut(route) {
                    if let Some(label) = r.label_mut(label_id) {
                        label.linear_distance += linear_distance_delta;
                    }
                    r.update_label_positions();
                }
                self.state = TextAnchorDragged {
                    route,
                    label_id,
                    delta_pos: Vec2::ZERO,
                }
                .into();
            }
            Action::FinalizeTextAnchorDrag { route } => {
                if let Some(r) = self.data.auto_route_mut(route) {
                    r.update_waypoints();
                }
                self.state = RouteSelected { id: route }.into();
            }
            Action::AllocateLabelAndEdit { route, pos } => {
                self.state = if let Some(r) = self.data.auto_route_mut(route) {
                    let label_id = r.allocate_label(pos);
                    EditingRouteLabelText {
                        id: route,
                        label_id,
                    }
                    .into()
                } else {
                    State::Idle
                };
            }
        }
    }
    pub fn update_state(&mut self, mut response: Response) {
        if self.mode == Mode::Block {
            if let Some((start, end)) = self.new_block.update(&response) {
                let rect = self.data.add_rect_box(start, end);
                self.state = Selected { rect }.into();
                self.mode = Mode::Select;
                self.update_graph(&[]);
            }
            return;
        }
        if self.mode == Mode::Route {
            if let Some(new_route) = self.route.update(&mut self.data, &mut response) {
                let id = self.data.add_auto_route(new_route);
                self.state = RouteSelected { id }.into();
            }
            self.update_graph(&[]);
            return;
        }
        if self.mode == Mode::Pin {
            self.new_pin.update(&mut self.data, &mut response);
            self.update_graph(&[]);
            return;
        }
        if let Some(edit) = self.drag_pin.as_mut() {
            eprintln!("Drag pin active");
            let subtool_state = edit.update(&mut self.data, &mut response);
            if subtool_state == SubtoolState::Active {
                return;
            }
            eprintln!("Drag pin inactive");
            self.drag_pin = None;
            self.update_graph(&[]);
        }
        if let Some(rename) = self.rename_pin.as_mut() {
            let subtool_state = rename.update(&mut response);
            if subtool_state == SubtoolState::Active {
                return;
            }
            self.rename_pin = None;
            self.update_graph(&[]);
        }
        let Some(event) = compute_event(&response) else {
            return;
        };
        let old_state = std::mem::take(&mut self.state);
        let action = match old_state {
            State::Idle => self.handle_idle_state(event),
            State::AddText => self.handle_add_text(event),
            State::RouteHovered(inner) => self.handle_route_hovered(inner, event),
            State::RouteSelected(inner) => self.handle_route_selected(inner, event),
            State::RouteLabelHovered(inner) => self.handle_route_label_hovered(inner, event),
            State::TextAnchorHovered(inner) => self.handle_text_anchor_hovered(inner, event),
            State::TitleControlHovered(inner) => self.handle_title_control_hovered(inner, event),
            State::TitleHovered(inner) => self.handle_title_hovered(inner, event),
            State::PotentialResize(inner) => self.handle_potential_resize(inner, event),
            //            State::PinLabelHovered(inner) => self.handle_pin_label_hovered(inner, event),
            //            State::PinLabelGripHovered(inner) => self.handle_pin_label_grip_hovered(inner, event),
            State::Panning => self.handle_panning(event),
            State::EditingName(inner) => self.handle_editing_name(inner, event),
            State::AddTextHoveredRoute(inner) => self.handle_add_text_hovered_route(inner, event),
            State::RouteEdgeHovered(inner) => self.handle_route_edge_hovered(inner, event),
            State::RouteCornerHovered(inner) => self.handle_route_corner_hovered(inner, event),
            State::WaypointHovered(inner) => self.handle_waypoint_hovered(inner, event),
            State::TextAnchorDragged(inner) => self.handle_text_anchor_dragged(inner, event),
            State::TitleControlDragged(inner) => self.handle_title_control_dragged(inner, event),
            State::AddTextButtonHovered(inner) => self.handle_add_text_button_hovered(inner, event),
            State::WaypointDragged(inner) => self.handle_waypoint_dragged(inner, event),
            State::RouteEdgeDragged(inner) => self.handle_route_edge_dragged(inner, event),
            State::Selected(inner) => self.handle_selected_state(inner, event),
            State::ResizingRect(inner) => self.handle_resizing_rect(inner, event),
            State::MovingRect(inner) => self.handle_moving_rect(inner, event),
            State::EditingRouteLabelText(inner) => {
                self.handle_editing_route_label_text(inner, event)
            } //            State::PinDragged(inner) => self.handle_pin_dragged(inner, event),
        };
        self.apply_action(action);
    }
    fn build_router(&self) -> RouterNG {
        let mut builder = RouterNGBuilder::default();
        for (id, rect_box) in self.data.rect_boxes() {
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
    fn update_graph(&mut self, ripup: &[RouteId]) {
        let mut router = self.build_router();
        let mut routes = self.data.take_routes();
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
                && !ripup.contains(&id)
            {
                router.add_existing_route(route.iter_edges().map(|(_, edge)| edge), WIRE_COST);
            } else {
                route.rip_and_reroute(anchor_start, anchor_end, &mut router);
            }
        }
        self.data.set_routes(routes);
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
    let box1_id = drawing.data.add_rect_box(origin_1, origin_1 + size);
    let box1_pin1 = drawing
        .data
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
        .data
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
    let box2_id = drawing.data.add_rect_box(origin_2, origin_2 + size);
    let box2_port1 = drawing
        .data
        .rect_mut(box2_id)
        .and_then(|ps| ps.add_pin("o.1.read_logic".to_string(), PinSide::East, GRID_SIZE * 1.0))
        .expect("add_pin");
    let box2_anchor1 = LineAnchor {
        rect: box2_id,
        pin: box2_port1,
    };
    let box2_pin2 = drawing
        .data
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
    drawing.data.add_auto_route(route);
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
    drawing.data.add_auto_route(route);
    drawing.data.add_port_box(
        "clk".to_string(),
        PinSide::East,
        Rect::from_center_size(pos2(160.0, 315.0), vec2(4.0 * GRID_SIZE, PORT_HEIGHT)),
    );
    drawing.data.add_port_box(
        "out".to_string(),
        PinSide::West,
        Rect::from_center_size(pos2(580.0, 15.0), vec2(4.0 * GRID_SIZE, PORT_HEIGHT)),
    );
    drawing
}
