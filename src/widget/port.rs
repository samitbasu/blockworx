use egui::{Color32, Pos2, Rect, Stroke, TextEdit, Ui, Vec2, pos2, vec2};

use crate::{
    grid::{GRID_SIZE, PORT_HEIGHT, PORT_RADIUS, PORT_TEXT_SIZE, grid_rect, snap},
    state::{RenderMode, ResizeMode},
    store::PinId,
    theme::get_theme,
    widget::{
        block::resize_rect,
        pin::{Pin, PinSide},
        render::{FocusResult, draw_box_outline, draw_control_frame, get_control_pin_bbox},
        shape::BaseShape,
    },
};

const PORT_PIN_ID: PinId = PinId::from_usize(0);

const PORT_RESIZE_MODES: &[ResizeMode] = &[
    ResizeMode::LeftTop,
    ResizeMode::RightTop,
    ResizeMode::LeftBottom,
    ResizeMode::RightBottom,
];

pub struct Port {
    inner: Rect,
    pin: Pin,
}

impl Port {
    pub fn new(pin_name: String, side: PinSide, inner: Rect) -> Self {
        let snapped = snap(inner);
        let clamped = Rect::from_min_size(snapped.min, vec2(snapped.width(), PORT_HEIGHT));
        Self {
            inner: clamped,
            pin: Pin {
                text: pin_name,
                side,
                offset: 0.0,
            },
        }
    }
    pub fn port_side(&self) -> PinSide {
        self.pin.side
    }
}

impl BaseShape for Port {
    fn name(&self) -> &str {
        &self.pin.text
    }
    fn name_mut(&mut self) -> &mut String {
        &mut self.pin.text
    }
    fn resize_modes(&self) -> &'static [ResizeMode] {
        PORT_RESIZE_MODES
    }
    fn gui_rect(&self) -> Rect {
        self.inner
    }
    fn gui_rect_mut(&mut self) -> &mut Rect {
        &mut self.inner
    }
    fn constrain_resize_delta(&self, mut delta: Vec2) -> Vec2 {
        delta.y = 0.0;
        delta
    }
    fn apply_resize(&mut self, _mode: ResizeMode, new_rect: Rect) {
        self.inner = Rect::from_min_size(
            pos2(new_rect.min.x, self.inner.min.y),
            vec2(new_rect.width(), PORT_HEIGHT),
        );
    }
    fn pin(&self, id: PinId) -> Option<&Pin> {
        (id == PORT_PIN_ID).then_some(&self.pin)
    }
    fn pins_mut(&mut self, id: PinId) -> Option<&mut Pin> {
        (id == PORT_PIN_ID).then_some(&mut self.pin)
    }
    fn with_pins(&self, mut f: impl FnMut(PinId, &Pin)) {
        f(PORT_PIN_ID, &self.pin);
    }
    fn pin_head_pos(&self, id: PinId) -> Option<Pos2> {
        if id != PORT_PIN_ID {
            return None;
        }
        Some(match self.pin.side {
            PinSide::East => self.inner.right_top() + vec2(GRID_SIZE, GRID_SIZE + self.pin.offset),
            PinSide::West => self.inner.left_top() + vec2(-GRID_SIZE, GRID_SIZE + self.pin.offset),
        })
    }
    fn anchor_point_with_rect(&self, rect: Rect, id: PinId) -> Option<Pos2> {
        if id != PORT_PIN_ID {
            return None;
        }
        Some(match self.pin.side {
            PinSide::East => pos2(
                rect.right() + GRID_SIZE,
                rect.top() + GRID_SIZE + self.pin.offset,
            ),
            PinSide::West => pos2(
                rect.left() - GRID_SIZE,
                rect.top() + GRID_SIZE + self.pin.offset,
            ),
        })
    }

    fn render(&mut self, mode: RenderMode, ui: &mut Ui) -> FocusResult {
        let bbox = self.inner;
        let side = Some(self.pin.side);
        let theme = get_theme(ui);

        let draw_normal = |bbox: Rect, name: &str, side: Option<PinSide>, ui: &mut Ui| {
            draw_box_outline(
                bbox,
                side,
                theme.shape_fill,
                Stroke::new(1.0, theme.shape_stroke),
                ui,
            );
            ui.painter().text(
                bbox.center(),
                egui::Align2::CENTER_CENTER,
                name,
                egui::FontId::monospace(PORT_TEXT_SIZE),
                theme.pin_text,
            );
        };

        match mode {
            RenderMode::Normal | RenderMode::TitleDragged { .. } => {
                draw_normal(bbox, &self.pin.text, side, ui);
            }
            RenderMode::Selected => {
                draw_normal(bbox, &self.pin.text, side, ui);
                draw_control_frame(bbox, side, PORT_RESIZE_MODES, None, None, None, ui);
            }
            RenderMode::PinHeadHovered { pin } => {
                draw_normal(bbox, &self.pin.text, side, ui);
                draw_control_frame(bbox, side, PORT_RESIZE_MODES, None, None, None, ui);
                if let Some(p) = self.pin(pin) {
                    let cb = get_control_pin_bbox(bbox, p);
                    ui.painter().circle(
                        cb.center(),
                        PORT_RADIUS,
                        theme.hover_fill,
                        (1.0, theme.control_handle_stroke),
                    );
                }
            }
            RenderMode::Moving { delta } => {
                let shifted = bbox.translate(delta);
                let predicted = grid_rect(shifted);
                draw_box_outline(
                    predicted,
                    side,
                    Color32::TRANSPARENT,
                    Stroke::new(1.0, theme.drag_preview_stroke),
                    ui,
                );
                draw_box_outline(
                    shifted,
                    side,
                    theme.drag_active_fill,
                    Stroke::new(2.0, theme.drag_active_stroke),
                    ui,
                );
                ui.painter().text(
                    shifted.center(),
                    egui::Align2::CENTER_CENTER,
                    &self.pin.text,
                    egui::FontId::monospace(PORT_TEXT_SIZE),
                    theme.pin_text,
                );
            }
            RenderMode::Resizing { mode, delta } => {
                let resized = resize_rect(&bbox, mode, delta);
                let predicted = grid_rect(resized);
                draw_box_outline(
                    predicted,
                    side,
                    Color32::TRANSPARENT,
                    Stroke::new(1.0, theme.drag_preview_stroke),
                    ui,
                );
                draw_box_outline(
                    resized,
                    side,
                    theme.drag_active_fill,
                    Stroke::new(2.0, theme.drag_active_stroke),
                    ui,
                );
                ui.painter().text(
                    resized.center(),
                    egui::Align2::CENTER_CENTER,
                    &self.pin.text,
                    egui::FontId::monospace(PORT_TEXT_SIZE),
                    theme.pin_text,
                );
            }
            RenderMode::PinDragged { .. } => {
                draw_normal(bbox, &self.pin.text, side, ui);
                ui.painter().line_segment(
                    [bbox.center_top(), bbox.center_bottom()],
                    (2.0, theme.pin_drag_indicator),
                );
            }
            RenderMode::EditingName | RenderMode::EditingPinText { .. } => {
                draw_normal(bbox, &self.pin.text, side, ui);
                draw_control_frame(bbox, side, PORT_RESIZE_MODES, None, None, None, ui);
                let text_width =
                    (self.pin.text.len() as f32 * PORT_TEXT_SIZE * 0.6 + 20.0).max(40.0);
                let editor_position =
                    Rect::from_center_size(bbox.center(), vec2(text_width, PORT_TEXT_SIZE + 4.0));
                let response = ui.place(
                    editor_position,
                    TextEdit::singleline(&mut self.pin.text)
                        .font(egui::FontId::monospace(PORT_TEXT_SIZE))
                        .desired_width(f32::INFINITY),
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    return FocusResult::LostFocus;
                } else {
                    response.request_focus();
                }
            }
        }
        FocusResult::KeptFocus
    }
}
