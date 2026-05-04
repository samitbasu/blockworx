use std::vec;

use egui::{
    Color32, Pos2, Rect, Shape, Stroke, StrokeKind, TextEdit, Ui, Vec2, epaint::PathShape,
    epaint::PathStroke, pos2, vec2,
};

use crate::{
    grid::*,
    state::*,
    store::{PinId, RectId},
    widget::{
        pin::{Pin, PinSide},
        rect_box::{RectBox, control_corner, resize_rect},
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GripState {
    Hidden,
    Drawn,
}

fn port_pin_side(rect: &RectBox) -> PinSide {
    rect.iter_pins()
        .next()
        .map(|(_, p)| p.side)
        .unwrap_or(PinSide::East)
}

fn rounded_pentagon(vertices: &[Pos2], radius: f32, fill: Color32, stroke: Stroke, ui: &mut Ui) {
    let n = vertices.len();
    let mut points = Vec::with_capacity(n * 5);
    for i in 0..n {
        let prev = vertices[(i + n - 1) % n];
        let curr = vertices[i];
        let next = vertices[(i + 1) % n];
        let dir_in = (curr - prev).normalized();
        let dir_out = (next - curr).normalized();
        let r = radius
            .min((curr - prev).length() * 0.5)
            .min((next - curr).length() * 0.5);
        let p0 = curr - dir_in * r;
        let p2 = curr + dir_out * r;
        points.push(p0);
        // Quadratic Bézier arc with curr as control point, 4 subdivisions
        for s in 1..=4_u8 {
            let t = s as f32 / 4.0;
            let it = 1.0 - t;
            points.push(pos2(
                it * it * p0.x + 2.0 * it * t * curr.x + t * t * p2.x,
                it * it * p0.y + 2.0 * it * t * curr.y + t * t * p2.y,
            ));
        }
    }
    ui.painter().add(Shape::Path(PathShape {
        points,
        closed: true,
        fill,
        stroke: PathStroke::new(stroke.width, stroke.color),
    }));
}

fn draw_box_outline(
    bbox: Rect,
    is_port: bool,
    side: PinSide,
    fill: Color32,
    stroke: Stroke,
    ui: &mut Ui,
) {
    if is_port {
        let render_rect =
            Rect::from_center_size(bbox.center(), vec2(bbox.width(), PORT_RENDER_HEIGHT));
        let center_y = render_rect.center().y;
        let vertices = match side {
            PinSide::East => [
                render_rect.left_top(),
                render_rect.right_top(),
                pos2(render_rect.right() + GRID_SIZE, center_y),
                render_rect.right_bottom(),
                render_rect.left_bottom(),
            ],
            PinSide::West => [
                render_rect.right_top(),
                render_rect.left_top(),
                pos2(render_rect.left() - GRID_SIZE, center_y),
                render_rect.left_bottom(),
                render_rect.right_bottom(),
            ],
        };
        rounded_pentagon(&vertices, 3.0, fill, stroke, ui);
    } else {
        ui.painter()
            .rect(bbox, 3.0, fill, stroke, StrokeKind::Middle);
    }
}

fn draw_port_text_in_rect(rect: &RectBox, bbox: Rect, ui: &mut Ui) {
    if let Some((_, pin)) = rect.iter_pins().next() {
        ui.painter().text(
            bbox.center(),
            egui::Align2::CENTER_CENTER,
            &pin.text,
            egui::FontId::monospace(PORT_TEXT_SIZE),
            Color32::BLACK,
        );
    }
}

fn draw_port_text(rect: &RectBox, ui: &mut Ui) {
    draw_port_text_in_rect(rect, rect.gui_rect(), ui);
}

pub fn draw_resizing_rect(rect: &RectBox, ui: &mut Ui, mode: ResizeMode, delta: Vec2) {
    let resized_rect = resize_rect(&rect.gui_rect(), mode, delta);
    let predicted_rect = grid_rect(resized_rect);
    let is_port = rect.is_port();
    let side = port_pin_side(rect);
    draw_box_outline(
        predicted_rect,
        is_port,
        side,
        Color32::TRANSPARENT,
        Stroke::new(1.0, Color32::DARK_GRAY),
        ui,
    );
    draw_box_outline(
        resized_rect,
        is_port,
        side,
        Color32::LIGHT_GRAY,
        Stroke::new(2.0, Color32::DARK_RED),
        ui,
    );
    if is_port {
        draw_port_text_in_rect(rect, resized_rect, ui);
    } else {
        ui.painter().text(
            resized_rect.center_top() + vec2(0.0, SHIM),
            egui::Align2::CENTER_TOP,
            rect.name(),
            egui::FontId::monospace(TITLE_TEXT_SIZE),
            Color32::BLACK,
        );
        render_pins_with_box(
            rect.iter_pins().map(|(_, pin)| pin),
            resized_rect,
            GripState::Hidden,
            ui,
        );
    }
}

pub fn draw_moving_rect(rect: &RectBox, ui: &mut Ui, delta: Vec2) {
    let shifted_rect = rect.gui_rect().translate(delta);
    let predicted_rect = grid_rect(shifted_rect);
    let is_port = rect.is_port();
    let side = port_pin_side(rect);
    draw_box_outline(
        predicted_rect,
        is_port,
        side,
        Color32::TRANSPARENT,
        Stroke::new(1.0, Color32::DARK_GRAY),
        ui,
    );
    draw_box_outline(
        shifted_rect,
        is_port,
        side,
        Color32::LIGHT_GRAY,
        Stroke::new(2.0, Color32::DARK_RED),
        ui,
    );
    if is_port {
        draw_port_text_in_rect(rect, shifted_rect, ui);
    } else {
        ui.painter().text(
            shifted_rect.center_top() + vec2(0.0, SHIM),
            egui::Align2::CENTER_TOP,
            rect.name(),
            egui::FontId::monospace(TITLE_TEXT_SIZE),
            Color32::BLACK,
        );
        render_pins_with_box(
            rect.iter_pins().map(|(_, pin)| pin),
            shifted_rect,
            GripState::Hidden,
            ui,
        );
    }
}

fn render_pins_with_box<'a>(
    iter: impl Iterator<Item = &'a Pin>,
    bbox: Rect,
    grip_state: GripState,
    ui: &mut Ui,
) {
    for pin in iter {
        draw_pin(bbox, pin, grip_state, 0.0, ui);
    }
}

fn render_frame(rect: &RectBox, ui: &mut Ui) {
    let egui_box = rect.gui_rect();
    let side = port_pin_side(rect);
    draw_box_outline(
        egui_box,
        rect.is_port(),
        side,
        Color32::LIGHT_GRAY,
        Stroke::new(1.0, Color32::BLUE),
        ui,
    );
    if !rect.is_port() {
        ui.painter().text(
            egui_box.center_top() + vec2(0.0, SHIM),
            egui::Align2::CENTER_TOP,
            rect.name(),
            egui::FontId::monospace(TITLE_TEXT_SIZE),
            Color32::BLACK,
        );
    }
}

fn render_with_grip_state(rect: &RectBox, grip_state: GripState, ui: &mut Ui) {
    render_frame(rect, ui);
    if rect.is_port() {
        draw_port_text(rect, ui);
    } else {
        render_pins_with_box(
            rect.iter_pins().map(|(_, pin)| pin),
            rect.gui_rect(),
            grip_state,
            ui,
        );
    }
}

// Draw
//          v anchor point
// port [s] |----
fn draw_pin(bbox: Rect, pin: &Pin, grip_state: GripState, offset: f32, ui: &mut Ui) {
    let y_coord = bbox.top() + GRID_SIZE + pin.offset + offset;
    let (text_pos, stem, align) = match pin.side {
        PinSide::East => (
            pos2(bbox.right() - GRID_SIZE, y_coord),
            vec2(GRID_SIZE, 0.0),
            egui::Align2::RIGHT_CENTER,
        ),
        PinSide::West => (
            pos2(bbox.left() + GRID_SIZE, y_coord),
            vec2(-GRID_SIZE, 0.0),
            egui::Align2::LEFT_CENTER,
        ),
    };
    ui.painter().line_segment(
        [text_pos + stem, text_pos + 2.0 * stem],
        (0.5, Color32::DARK_RED),
    );
    ui.painter().text(
        text_pos,
        align,
        &pin.text,
        egui::FontId::monospace(PORT_TEXT_SIZE),
        Color32::BLACK,
    );
    let hamburger_rect = get_hamburger_rect(bbox.translate(vec2(0.0, offset)), pin);
    // Draw a hamburger grip.
    match grip_state {
        GripState::Hidden => {}
        GripState::Drawn => {
            let bun_height = hamburger_rect.height() / 5.0;
            for i in 0..3 {
                let bun_rect = Rect::from_center_size(
                    pos2(
                        hamburger_rect.center().x,
                        hamburger_rect.top() + bun_height / 2.0 + 2.0 * i as f32 * bun_height,
                    ),
                    vec2(hamburger_rect.width(), bun_height),
                );
                ui.painter().rect(
                    bun_rect,
                    bun_height / 2.0,
                    Color32::DARK_GRAY.gamma_multiply(0.3),
                    Stroke::NONE,
                    StrokeKind::Middle,
                );
            }
        }
    }
}

pub fn estimate_bbox_for_pin_text(bbox: Rect, pin: &Pin) -> Rect {
    let y_coord = bbox.top() + GRID_SIZE + pin.offset;
    let text_width = pin.text.len() as f32 * PORT_TEXT_SIZE * 0.6;
    match pin.side {
        PinSide::East => Rect::from_min_max(
            pos2(
                bbox.right() - GRID_SIZE - text_width,
                y_coord - PORT_TEXT_SIZE / 2.0,
            ),
            pos2(bbox.right() - GRID_SIZE, y_coord + PORT_TEXT_SIZE / 2.0),
        ),
        PinSide::West => Rect::from_min_max(
            pos2(bbox.left() + GRID_SIZE, y_coord - PORT_TEXT_SIZE / 2.0),
            pos2(
                bbox.left() + GRID_SIZE + text_width,
                y_coord + PORT_TEXT_SIZE / 2.0,
            ),
        ),
    }
}

pub fn get_control_pin_bbox(bbox: Rect, pin: &Pin) -> Rect {
    let y_coord = bbox.top() + GRID_SIZE + pin.offset;
    match pin.side {
        PinSide::East => Rect::from_center_size(
            pos2(bbox.right() + GRID_SIZE, y_coord),
            vec2(PORT_RADIUS * 2.0, PORT_RADIUS * 2.0),
        ),
        PinSide::West => Rect::from_center_size(
            pos2(bbox.left() - GRID_SIZE, y_coord),
            vec2(PORT_RADIUS * 2.0, PORT_RADIUS * 2.0),
        ),
    }
}

pub fn get_hamburger_rect(bbox: Rect, pin: &Pin) -> Rect {
    let y_coord = bbox.top() + GRID_SIZE + pin.offset;
    let (text_pos, stem) = match pin.side {
        PinSide::East => (
            pos2(bbox.right() - GRID_SIZE, y_coord),
            vec2(GRID_SIZE, 0.0),
        ),
        PinSide::West => (
            pos2(bbox.left() + GRID_SIZE, y_coord),
            vec2(-GRID_SIZE, 0.0),
        ),
    };
    Rect::from_center_size(text_pos + stem / 2.0, vec2(GRIP_SIZE, GRIP_SIZE))
}

pub fn draw_dragged_pin(rrect: &RectBox, pin: PinId, delta_pos: Vec2, ui: &mut Ui) {
    let Some(pin) = rrect.pin(pin) else {
        return;
    };
    draw_pin(rrect.gui_rect(), pin, GripState::Drawn, delta_pos.y, ui);
}

pub fn draw_control_frame(rrect: &RectBox, ui: &mut Ui) -> Option<()> {
    let bbox = rrect.gui_rect();
    let side = port_pin_side(rrect);
    draw_box_outline(
        bbox,
        rrect.is_port(),
        side,
        Color32::TRANSPARENT,
        Stroke::new(0.5, Color32::DARK_RED),
        ui,
    );
    for mode in rrect.resize_modes() {
        let pos = control_corner(&bbox, *mode);
        ui.painter().rect(
            Rect::from_center_size(pos, vec2(CONTROL_HANDLE_SIZE, CONTROL_HANDLE_SIZE)),
            0.0,
            Color32::WHITE,
            (0.5, Color32::BLACK),
            StrokeKind::Middle,
        );
    }
    [rrect.add_pin_button_east(), rrect.add_pin_button_west()]
        .iter()
        .flatten()
        .for_each(|&pin_pos| {
            ui.painter()
                .circle(pin_pos, PORT_RADIUS, Color32::WHITE, (0.5, Color32::BLACK));
            ui.painter().line_segment(
                [
                    pin_pos + vec2(-PORT_RADIUS / 2.0, 0.0),
                    pin_pos + vec2(PORT_RADIUS / 2.0, 0.0),
                ],
                (1.0, Color32::BLACK),
            );
            ui.painter().line_segment(
                [
                    pin_pos + vec2(0.0, -PORT_RADIUS / 2.0),
                    pin_pos + vec2(0.0, PORT_RADIUS / 2.0),
                ],
                (1.0, Color32::BLACK),
            );
        });
    Some(())
}

fn render_selected(target: &RectBox, ui: &mut Ui) {
    render_with_grip_state(target, GripState::Drawn, ui);
    draw_control_frame(target, ui);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FocusResult {
    #[default]
    KeptFocus,
    LostFocus,
}

pub fn render_rect_box(
    id: RectId,
    target: &mut RectBox,
    state: &State,
    ui: &mut Ui,
) -> FocusResult {
    match state {
        State::MovingRect(MovingRect { rect, delta_pos }) if id == *rect => {
            draw_moving_rect(target, ui, *delta_pos);
        }
        State::Selected(Selected { rect })
        | State::PotentialResize(PotentialResize { rect, .. })
        | State::PinLabelHovered(PinLabelHovered { rect, .. })
        | State::PinLabelGripHovered(PinLabelGripHovered { rect, .. })
            if id == *rect =>
        {
            render_selected(target, ui);
        }
        State::PinHeadHovered(PinHeadHovered { rect, pin }) if id == *rect => {
            render_selected(target, ui);
            if let Some(pin) = target.pin(*pin) {
                let bbox = get_control_pin_bbox(target.gui_rect(), pin);
                ui.painter().circle(
                    bbox.center(),
                    PORT_RADIUS,
                    Color32::GRAY,
                    (1.0, Color32::BLACK),
                );
            }
        }
        State::ResizingRect(ResizingRect {
            rect,
            mode,
            delta_pos,
            ..
        }) if id == *rect => {
            draw_resizing_rect(target, ui, *mode, *delta_pos);
        }
        State::PinDragged(PinDragged {
            rect,
            pin,
            delta_pos,
        }) if id == *rect => {
            if target.is_port() {
                render_frame(target, ui);
                draw_port_text(target, ui);
                ui.painter().line_segment(
                    [
                        target.gui_rect().center_top(),
                        target.gui_rect().center_bottom(),
                    ],
                    (2.0, Color32::DARK_GRAY.gamma_multiply(0.3)),
                );
            } else {
                render_frame(target, ui);
                render_pins_with_box(
                    target
                        .iter_pins()
                        .filter_map(|(lid, l)| if lid != *pin { Some(l) } else { None }),
                    target.gui_rect(),
                    GripState::Drawn,
                    ui,
                );
                draw_control_frame(target, ui);
                ui.painter().line_segment(
                    [
                        target.gui_rect().center_top(),
                        target.gui_rect().center_bottom(),
                    ],
                    (2.0, Color32::DARK_GRAY.gamma_multiply(0.3)),
                );
                draw_dragged_pin(target, *pin, *delta_pos, ui);
            }
        }
        State::EditingName(EditingName { rect }) if id == *rect => {
            render_selected(target, ui);
            let rect_name_width = target.name().len() as f32 * 10.0 + 10.0;
            let editor_position =
                target.gui_rect().center_top() + vec2(-rect_name_width / 2.0, SHIM);
            let editor_rect = Rect::from_min_size(editor_position, vec2(rect_name_width, 20.0));
            let response = ui.place(
                editor_rect,
                TextEdit::singleline(target.name_mut())
                    .font(egui::FontId::monospace(TITLE_TEXT_SIZE))
                    .desired_width(f32::INFINITY),
            );
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                return FocusResult::LostFocus;
            } else {
                response.request_focus();
            }
        }
        State::EditingPinText(EditingPinText { rect, pin }) if id == *rect => {
            render_selected(target, ui);
            let target_inner = target.gui_rect();
            let is_port = target.is_port();
            let Some(pin_ref) = target.pins_mut(*pin) else {
                return FocusResult::KeptFocus;
            };
            let editor_position = if is_port {
                let text_width =
                    (pin_ref.text.len() as f32 * PORT_TEXT_SIZE * 0.6 + 20.0).max(40.0);
                Rect::from_center_size(
                    target_inner.center(),
                    vec2(text_width, PORT_TEXT_SIZE + 4.0),
                )
            } else {
                let editor_width =
                    ((pin_ref.text.len() as f32 * PORT_TEXT_SIZE * 0.6 + 10.0) / 2.0).max(20.0);
                get_hamburger_rect(target_inner, pin_ref).expand2(vec2(editor_width, 0.0))
            };
            let response = ui.place(
                editor_position,
                TextEdit::singleline(&mut pin_ref.text)
                    .font(egui::FontId::monospace(PORT_TEXT_SIZE))
                    .desired_width(f32::INFINITY),
            );
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                return FocusResult::LostFocus;
            } else {
                response.request_focus();
            }
        }
        _ => render_with_grip_state(target, GripState::Hidden, ui),
    }
    FocusResult::KeptFocus
}

pub enum RenderSegment {
    Edge { from: Pos2, to: Pos2 },
    Chamfer { from: Pos2, to: Pos2, center: Pos2 },
}

pub struct RenderedPath {
    pub segments: Vec<RenderSegment>,
}

impl From<Vec<RenderSegment>> for RenderedPath {
    fn from(segments: Vec<RenderSegment>) -> Self {
        Self { segments }
    }
}

impl RenderedPath {
    pub fn render(&self, ui: &mut Ui, stroke: impl Into<Stroke>) {
        let stroke = stroke.into();
        let mut points = vec![];
        for segment in &self.segments {
            let (from, to) = match segment {
                RenderSegment::Edge { from, to } => (*from, *to),
                RenderSegment::Chamfer { from, to, .. } => (*from, *to),
            };
            if points.last() != Some(&from) {
                points.push(from);
            }
            points.push(to);
        }
        ui.painter().line(points, stroke);
    }
}

pub fn render_path_with_chamfered_corners(points: &[Pos2]) -> RenderedPath {
    let start = points.first().cloned().unwrap_or_default();
    let end = points.last().cloned().unwrap_or_default();
    let mut rendered_segments: Vec<RenderSegment> = Vec::new();
    let mut last = start;
    for window in points.windows(3) {
        let [prev, current, next] = [window[0], window[1], window[2]];
        let v1 = (current - prev).normalized();
        let v2 = (next - current).normalized();
        let angle = v1.dot(v2);
        if angle.abs() < 0.1 {
            let chamfer_length = GRID_SIZE / 4.0;
            let chamfer_point1 = current - v1 * chamfer_length;
            let chamfer_point2 = current + v2 * chamfer_length;
            rendered_segments.push(RenderSegment::Edge {
                from: last,
                to: chamfer_point1,
            });
            rendered_segments.push(RenderSegment::Chamfer {
                from: chamfer_point1,
                to: chamfer_point2,
                center: current,
            });
            last = chamfer_point2;
        } else {
            rendered_segments.push(RenderSegment::Edge {
                from: last,
                to: current,
            });
            last = current;
        }
    }
    rendered_segments.push(RenderSegment::Edge {
        from: last,
        to: end,
    });
    rendered_segments.into()
}
