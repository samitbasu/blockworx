use egui::{
    Color32, Pos2, Rect, Stroke, StrokeKind, Ui, epaint::PathShape, epaint::PathStroke, pos2, vec2,
};

use crate::{
    grid::*,
    state::ResizeMode,
    theme::get_theme,
    widget::{
        block::{Title, TitleSide, control_corner},
        pin::{Pin, PinSide},
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GripState {
    Hidden,
    Drawn,
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
        for s in 1..=4_u8 {
            let t = s as f32 / 4.0;
            let it = 1.0 - t;
            points.push(pos2(
                it * it * p0.x + 2.0 * it * t * curr.x + t * t * p2.x,
                it * it * p0.y + 2.0 * it * t * curr.y + t * t * p2.y,
            ));
        }
    }
    ui.painter().add(egui::Shape::Path(PathShape {
        points,
        closed: true,
        fill,
        stroke: PathStroke::new(stroke.width, stroke.color),
    }));
}

/// If `port_side` is Some, renders a pentagon pointing in that direction; otherwise a rounded rect.
pub fn draw_box_outline(
    bbox: Rect,
    port_side: Option<PinSide>,
    fill: Color32,
    stroke: Stroke,
    ui: &mut Ui,
) {
    if let Some(side) = port_side {
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

pub fn pin_text_location(bbox: Rect, pin: &Pin, offset: f32) -> (Pos2, egui::Align2) {
    let y_coord = bbox.top() + GRID_SIZE + pin.offset + offset;
    match pin.side {
        PinSide::East => (
            pos2(bbox.right() - GRID_SIZE, y_coord),
            egui::Align2::RIGHT_CENTER,
        ),
        PinSide::West => (
            pos2(bbox.left() + GRID_SIZE, y_coord),
            egui::Align2::LEFT_CENTER,
        ),
    }
}

pub fn draw_pin(bbox: Rect, pin: &Pin, grip_state: GripState, offset: f32, ui: &mut Ui) {
    let theme = get_theme(ui);
    let (text_pos, align) = pin_text_location(bbox, pin, offset);
    let stem = match pin.side {
        PinSide::East => vec2(GRID_SIZE, 0.0),
        PinSide::West => vec2(-GRID_SIZE, 0.0),
    };
    ui.painter().line_segment(
        [text_pos + stem, text_pos + 2.0 * stem],
        (0.5, theme.pin_stem),
    );
    ui.painter().text(
        text_pos,
        align,
        &pin.text,
        egui::FontId::monospace(PORT_TEXT_SIZE),
        theme.pin_text,
    );
    let bbox = estimate_bbox_for_pin_text(bbox, pin).expand(3.0);
    ui.painter()
        .rect_stroke(bbox, 1.0, (0.5, Color32::DARK_BLUE), StrokeKind::Middle);
    let hamburger_rect = get_hamburger_rect(bbox.translate(vec2(0.0, offset)), pin);
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
                    theme.hamburger_menu,
                    Stroke::NONE,
                    StrokeKind::Middle,
                );
            }
        }
    }
}

pub fn render_pins_with_box<'a>(
    iter: impl Iterator<Item = &'a Pin>,
    bbox: Rect,
    grip_state: GripState,
    ui: &mut Ui,
) {
    for pin in iter {
        draw_pin(bbox, pin, grip_state, 0.0, ui);
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

pub fn block_title_position(bbox: Rect, title: &Title) -> (Pos2, egui::Align2) {
    match title.side {
        TitleSide::Bottom => (
            bbox.center_bottom() + vec2(title.offset, GRID_SIZE),
            egui::Align2::CENTER_BOTTOM,
        ),
        TitleSide::Top => (
            bbox.center_top() + vec2(title.offset, 0.0),
            egui::Align2::CENTER_BOTTOM,
        ),
    }
}

pub fn draw_control_frame(
    bbox: Rect,
    port_side: Option<PinSide>,
    resize_modes: &[ResizeMode],
    add_pin_east: Option<Pos2>,
    add_pin_west: Option<Pos2>,
    title: Option<&Title>,
    ui: &mut Ui,
) {
    let theme = get_theme(ui);
    draw_box_outline(
        bbox,
        port_side,
        Color32::TRANSPARENT,
        Stroke::new(0.5, theme.selection_frame),
        ui,
    );
    for &mode in resize_modes {
        let pos = control_corner(&bbox, mode);
        ui.painter().rect(
            Rect::from_center_size(pos, vec2(CONTROL_HANDLE_SIZE, CONTROL_HANDLE_SIZE)),
            0.0,
            theme.control_handle_fill,
            (0.5, theme.control_handle_stroke),
            StrokeKind::Middle,
        );
    }
    for &pin_pos in [add_pin_east, add_pin_west].iter().flatten() {
        ui.painter().circle(
            pin_pos,
            PORT_RADIUS,
            theme.control_handle_fill,
            (0.5, theme.control_handle_stroke),
        );
        ui.painter().line_segment(
            [
                pin_pos + vec2(-PORT_RADIUS / 2.0, 0.0),
                pin_pos + vec2(PORT_RADIUS / 2.0, 0.0),
            ],
            (1.0, theme.control_handle_stroke),
        );
        ui.painter().line_segment(
            [
                pin_pos + vec2(0.0, -PORT_RADIUS / 2.0),
                pin_pos + vec2(0.0, PORT_RADIUS / 2.0),
            ],
            (1.0, theme.control_handle_stroke),
        );
    }
    if let Some(title) = title {
        let (title_anchor_pos, _) = block_title_position(bbox, title);
        ui.painter().rect(
            Rect::from_center_size(
                title_anchor_pos,
                vec2(CONTROL_HANDLE_SIZE, CONTROL_HANDLE_SIZE),
            ),
            0.0,
            theme.control_handle_fill,
            (0.5, theme.control_handle_stroke),
            StrokeKind::Middle,
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FocusResult {
    #[default]
    KeptFocus,
    LostFocus,
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
