use egui::{Color32, Pos2, Rect, Stroke, TextEdit, Ui, Vec2, pos2, vec2};

use crate::{
    grid::{
        GRID_SIZE, PORT_RADIUS, PORT_TEXT_SIZE, TITLE_TEXT_SIZE, grid_rect, round_to_grid, snap,
    },
    state::{RenderMode, ResizeMode},
    store::*,
    theme::get_theme,
    widget::{
        pin::{Pin, PinSide},
        render::{
            FocusResult, GripState, block_title_position, draw_box_outline, draw_control_frame,
            draw_pin, get_control_pin_bbox, pin_text_location, render_pins_with_box,
        },
        shape::BaseShape,
    },
};

fn draw_block_title(bbox: Rect, title: &Title, ui: &mut Ui) {
    let theme = get_theme(ui);
    let (pos, align) = block_title_position(bbox, title);
    ui.painter().text(
        pos,
        align,
        &title.name,
        egui::FontId::monospace(TITLE_TEXT_SIZE),
        theme.shape_title,
    );
}

fn draw_block_frame(bbox: Rect, title: &Title, ui: &mut Ui) {
    let theme = get_theme(ui);
    draw_box_outline(
        bbox,
        None,
        theme.shape_fill,
        Stroke::new(1.0, theme.shape_stroke),
        ui,
    );
    draw_block_title(bbox, title, ui);
}

pub fn resize_rect(rect: &Rect, mode: ResizeMode, delta: Vec2) -> Rect {
    match mode {
        ResizeMode::LeftTop => Rect::from_two_pos(rect.left_top() + delta, rect.right_bottom()),
        ResizeMode::RightTop => Rect::from_two_pos(rect.right_top() + delta, rect.left_bottom()),
        ResizeMode::LeftBottom => Rect::from_two_pos(rect.left_bottom() + delta, rect.right_top()),
        ResizeMode::RightBottom => Rect::from_two_pos(rect.right_bottom() + delta, rect.left_top()),
    }
}

pub fn control_corner(rect: &Rect, mode: ResizeMode) -> Pos2 {
    match mode {
        ResizeMode::LeftTop => rect.left_top(),
        ResizeMode::RightTop => rect.right_top(),
        ResizeMode::LeftBottom => rect.left_bottom(),
        ResizeMode::RightBottom => rect.right_bottom(),
    }
}

const NORMAL_RESIZE_MODES: &[ResizeMode] = &[
    ResizeMode::LeftTop,
    ResizeMode::RightTop,
    ResizeMode::LeftBottom,
    ResizeMode::RightBottom,
];

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub enum TitleSide {
    Top,
    #[default]
    Bottom,
}

#[derive(Clone, PartialEq, Default)]
pub struct Title {
    pub name: String,
    pub side: TitleSide,
    pub offset: f32,
}

pub struct Block {
    title: Title,
    inner: Rect,
    pins: Store<PinId, Pin>,
}

impl Block {
    pub fn new(name: String, inner: Rect) -> Self {
        Self {
            title: Title {
                name,
                ..Title::default()
            },
            inner: snap(inner),
            pins: Store::default(),
        }
    }

    pub fn is_pin_offset_available(&self, side: PinSide, offset: f32) -> bool {
        if offset < 0.0 || offset > self.inner.height() {
            return false;
        }
        self.pins
            .values()
            .filter(|l| l.side == side)
            .all(|l| (l.offset - offset).abs() >= GRID_SIZE * 0.2)
    }

    pub fn update_pin_offset_inner(&mut self, pin_id: PinId, delta_y: f32) {
        let Some(pin_ref) = self.pin(pin_id) else {
            return;
        };
        let pin_offset = round_to_grid(pin_ref.offset + delta_y);
        if !self.is_pin_offset_available(pin_ref.side, pin_offset) {
            return;
        }
        let Some(pin_ref) = self.pins_mut(pin_id) else {
            return;
        };
        pin_ref.offset = pin_offset;
    }

    pub fn next_pin_offset_inner(&self, side: PinSide) -> Option<f32> {
        let max_pos = (self.inner.height() / GRID_SIZE) as i32 - 1;
        if max_pos <= 0 {
            return None;
        }
        (0_u32..max_pos as u32).find_map(|ndx| {
            let offset = ndx as f32 * GRID_SIZE;
            if self
                .pins
                .values()
                .any(|l| l.side == side && (l.offset - offset).abs() < GRID_SIZE * 0.6)
            {
                None
            } else {
                Some(offset)
            }
        })
    }

    pub fn add_pin_inner(&mut self, text: String, side: PinSide, offset: f32) -> PinId {
        self.pins.insert(Pin { text, side, offset })
    }

    pub fn add_pin_button_east_inner(&self) -> Option<Pos2> {
        let offset = self.next_pin_offset_inner(PinSide::East)?;
        Some(self.inner.right_top() + vec2(GRID_SIZE, GRID_SIZE + offset))
    }

    pub fn add_pin_button_west_inner(&self) -> Option<Pos2> {
        let offset = self.next_pin_offset_inner(PinSide::West)?;
        Some(self.inner.left_top() + vec2(-GRID_SIZE, GRID_SIZE + offset))
    }
    pub fn title_bbox(&self) -> Rect {
        let title_width = (self.title.name.len() as f32 * TITLE_TEXT_SIZE * 0.6 + 10.0).max(20.0);
        let (title_pos, title_align) = block_title_position(self.inner, &self.title);
        title_align.anchor_size(title_pos, vec2(title_width, TITLE_TEXT_SIZE))
    }
}

impl BaseShape for Block {
    fn name(&self) -> &str {
        &self.title.name
    }
    fn name_mut(&mut self) -> &mut String {
        &mut self.title.name
    }
    fn title(&self) -> Option<&Title> {
        Some(&self.title)
    }
    fn title_mut(&mut self) -> Option<&mut Title> {
        Some(&mut self.title)
    }
    fn resize_modes(&self) -> &'static [ResizeMode] {
        NORMAL_RESIZE_MODES
    }
    fn gui_rect(&self) -> Rect {
        self.inner
    }
    fn gui_rect_mut(&mut self) -> &mut Rect {
        &mut self.inner
    }
    fn apply_resize(&mut self, _mode: ResizeMode, new_rect: Rect) {
        self.inner = new_rect;
    }
    fn pin(&self, id: PinId) -> Option<&Pin> {
        self.pins.get(id)
    }
    fn pins_mut(&mut self, id: PinId) -> Option<&mut Pin> {
        self.pins.get_mut(id)
    }
    fn with_pins(&self, mut f: impl FnMut(PinId, &Pin)) {
        self.pins.iter().for_each(|(id, pin)| f(id, pin));
    }
    fn pin_head_pos(&self, pin_id: PinId) -> Option<Pos2> {
        self.pins.get(pin_id).map(|pin| match pin.side {
            PinSide::East => self.inner.right_top() + vec2(GRID_SIZE, GRID_SIZE + pin.offset),
            PinSide::West => self.inner.left_top() + vec2(-GRID_SIZE, GRID_SIZE + pin.offset),
        })
    }
    fn anchor_point_with_rect(&self, rect: Rect, id: PinId) -> Option<Pos2> {
        self.pins.get(id).map(|pin| match pin.side {
            PinSide::East => pos2(
                rect.right() + GRID_SIZE,
                rect.top() + GRID_SIZE + pin.offset,
            ),
            PinSide::West => pos2(rect.left() - GRID_SIZE, rect.top() + GRID_SIZE + pin.offset),
        })
    }
    fn add_pin_button_east(&self) -> Option<Pos2> {
        self.add_pin_button_east_inner()
    }
    fn add_pin_button_west(&self) -> Option<Pos2> {
        self.add_pin_button_west_inner()
    }
    fn next_pin_offset(&self, side: PinSide) -> Option<f32> {
        self.next_pin_offset_inner(side)
    }
    fn add_pin(&mut self, text: String, side: PinSide, offset: f32) -> Option<PinId> {
        Some(self.add_pin_inner(text, side, offset))
    }
    fn update_pin_offset(&mut self, pin_id: PinId, delta_y: f32) {
        self.update_pin_offset_inner(pin_id, delta_y);
    }
    fn new_pin_location(&self, pos: Pos2) -> Option<(PinSide, f32)> {
        let left_top = self.inner.left_top();
        let offset = round_to_grid(pos.y - left_top.y);
        if offset < 0.0 || offset > self.inner.height() {
            return None;
        }
        if (pos.x - left_top.x).abs() < GRID_SIZE / 2.0 {
            if self.pins.iter().any(|(_, p)| {
                p.side == PinSide::West && (p.offset - offset).abs() < GRID_SIZE * 0.2
            }) {
                return None;
            }
            return Some((PinSide::West, offset));
        }
        let right_top = self.inner.right_top();
        if (pos.x - right_top.x).abs() < GRID_SIZE / 2.0 {
            if self.pins.iter().any(|(_, p)| {
                p.side == PinSide::East && (p.offset - offset).abs() < GRID_SIZE * 0.2
            }) {
                return None;
            }
            return Some((PinSide::East, offset));
        }
        None
    }
    fn title_anchor(&self) -> Option<Pos2> {
        let (pos, _) = block_title_position(self.inner, &self.title);
        Some(pos)
    }
    fn render(&mut self, mode: RenderMode, ui: &mut Ui) -> FocusResult {
        let bbox = self.inner;
        match mode {
            RenderMode::Normal => {
                draw_block_frame(bbox, &self.title, ui);
                render_pins_with_box(self.pins.values(), bbox, GripState::Hidden, ui);
            }
            RenderMode::PinAddHovered { side, offset } => {
                draw_block_frame(bbox, &self.title, ui);
                render_pins_with_box(self.pins.values(), bbox, GripState::Hidden, ui);
                let theme = get_theme(ui);
                let button_pos = match side {
                    PinSide::East => bbox.right_top() + vec2(0.0, offset),
                    PinSide::West => bbox.left_top() + vec2(0.0, offset),
                };
                ui.painter().circle(
                    button_pos,
                    PORT_RADIUS,
                    theme.add_button_fill,
                    (1.0, theme.shape_stroke),
                );
            }
            RenderMode::Selected => {
                draw_block_frame(bbox, &self.title, ui);
                render_pins_with_box(self.pins.values(), bbox, GripState::Drawn, ui);
                draw_control_frame(
                    bbox,
                    None,
                    NORMAL_RESIZE_MODES,
                    self.add_pin_button_east_inner(),
                    self.add_pin_button_west_inner(),
                    Some(&self.title),
                    ui,
                );
            }
            RenderMode::PinHeadHovered { pin } => {
                draw_block_frame(bbox, &self.title, ui);
                render_pins_with_box(self.pins.values(), bbox, GripState::Drawn, ui);
                draw_control_frame(
                    bbox,
                    None,
                    NORMAL_RESIZE_MODES,
                    self.add_pin_button_east_inner(),
                    self.add_pin_button_west_inner(),
                    Some(&self.title),
                    ui,
                );
                if let Some(p) = self.pins.get(pin) {
                    let theme = get_theme(ui);
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
                let theme = get_theme(ui);
                let shifted = bbox.translate(delta);
                let predicted = grid_rect(shifted);
                draw_box_outline(
                    predicted,
                    None,
                    Color32::TRANSPARENT,
                    Stroke::new(1.0, theme.drag_preview_stroke),
                    ui,
                );
                draw_box_outline(
                    shifted,
                    None,
                    theme.drag_active_fill,
                    Stroke::new(2.0, theme.drag_active_stroke),
                    ui,
                );
                draw_block_title(shifted, &self.title, ui);
                render_pins_with_box(self.pins.values(), shifted, GripState::Hidden, ui);
            }
            RenderMode::Resizing { mode, delta } => {
                let theme = get_theme(ui);
                let resized = resize_rect(&bbox, mode, delta);
                let predicted = grid_rect(resized);
                draw_box_outline(
                    predicted,
                    None,
                    Color32::TRANSPARENT,
                    Stroke::new(1.0, theme.drag_preview_stroke),
                    ui,
                );
                draw_box_outline(
                    resized,
                    None,
                    theme.drag_active_fill,
                    Stroke::new(2.0, theme.drag_active_stroke),
                    ui,
                );
                draw_block_title(resized, &self.title, ui);
                render_pins_with_box(self.pins.values(), resized, GripState::Hidden, ui);
            }
            RenderMode::PinDragged {
                pin: dragged_pin,
                delta,
            } => {
                let theme = get_theme(ui);
                draw_block_frame(bbox, &self.title, ui);
                render_pins_with_box(
                    self.pins
                        .iter()
                        .filter_map(|(id, p)| if id != dragged_pin { Some(p) } else { None }),
                    bbox,
                    GripState::Drawn,
                    ui,
                );
                draw_control_frame(
                    bbox,
                    None,
                    NORMAL_RESIZE_MODES,
                    self.add_pin_button_east_inner(),
                    self.add_pin_button_west_inner(),
                    Some(&self.title),
                    ui,
                );
                ui.painter().line_segment(
                    [bbox.center_top(), bbox.center_bottom()],
                    (2.0, theme.pin_drag_indicator),
                );
                if let Some(pin) = self.pins.get(dragged_pin) {
                    draw_pin(bbox, pin, GripState::Drawn, delta.y, ui);
                }
            }
            RenderMode::TitleDragged { delta } => {
                let mut shifted = self.title.clone();
                shifted.offset += delta.x;
                draw_block_frame(bbox, &shifted, ui);
                // Special case - draw a partial mid-line and put instructions above
                // and below the line
                let theme = get_theme(ui);
                ui.painter().line_segment(
                    [bbox.left_center(), bbox.right_center()],
                    (2.0, theme.pin_drag_indicator),
                );
                if shifted.side == TitleSide::Bottom {
                    ui.painter().text(
                        bbox.center(),
                        egui::Align2::CENTER_BOTTOM,
                        "↑ Drag to top",
                        egui::FontId::monospace(8.0),
                        theme.text_hint_text,
                    );
                } else {
                    ui.painter().text(
                        bbox.center(),
                        egui::Align2::CENTER_TOP,
                        "↓ Drag to bottom",
                        egui::FontId::monospace(8.0),
                        theme.text_hint_text,
                    );
                }
            }
            RenderMode::EditingName => {
                draw_block_frame(bbox, &self.title, ui);
                render_pins_with_box(self.pins.values(), bbox, GripState::Drawn, ui);
                draw_control_frame(
                    bbox,
                    None,
                    NORMAL_RESIZE_MODES,
                    self.add_pin_button_east_inner(),
                    self.add_pin_button_west_inner(),
                    Some(&self.title),
                    ui,
                );
                let rect_name_width =
                    (self.title.name.len() as f32 * TITLE_TEXT_SIZE * 0.6 + 10.0).max(20.0);
                let (title_pos, title_align) = block_title_position(bbox, &self.title);
                let editor_rect =
                    title_align.anchor_size(title_pos, vec2(rect_name_width, TITLE_TEXT_SIZE));
                let response = ui.place(
                    editor_rect,
                    TextEdit::singleline(&mut self.title.name)
                        .font(egui::FontId::monospace(TITLE_TEXT_SIZE))
                        .desired_width(f32::INFINITY),
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    return FocusResult::LostFocus;
                } else {
                    response.request_focus();
                }
            }
            RenderMode::EditingPinText { pin: pin_id } => {
                draw_block_frame(bbox, &self.title, ui);
                render_pins_with_box(self.pins.values(), bbox, GripState::Drawn, ui);
                draw_control_frame(
                    bbox,
                    None,
                    NORMAL_RESIZE_MODES,
                    self.add_pin_button_east_inner(),
                    self.add_pin_button_west_inner(),
                    Some(&self.title),
                    ui,
                );
                let Some(pin_ref) = self.pins.get_mut(pin_id) else {
                    return FocusResult::KeptFocus;
                };
                let editor_width =
                    ((pin_ref.text.len() as f32 * PORT_TEXT_SIZE * 0.6 + 10.0) / 2.0).max(20.0);
                let (pin_text_pos, pin_text_align) = pin_text_location(bbox, pin_ref, 0.0);
                let editor_position = pin_text_align
                    .anchor_size(pin_text_pos, vec2(editor_width * 2.0, PORT_TEXT_SIZE * 1.5));
                let theme = get_theme(ui);
                let response = ui.place(
                    editor_position,
                    TextEdit::singleline(&mut pin_ref.text)
                        .font(egui::FontId::monospace(PORT_TEXT_SIZE))
                        .background_color(theme.text_edit_background)
                        .text_color(theme.text_edit_text)
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
