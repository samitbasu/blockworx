use egui::{Pos2, Rect, Vec2, pos2, vec2};

use crate::{
    grid::{GRID_SIZE, PORT_HEIGHT, grid_rect, round_to_grid, snap},
    state::ResizeMode,
    store::*,
    widget::pin::{BoxKind, Pin, PinSide},
};

pub struct RectBox {
    name: String,
    inner: Rect,
    pins: Store<PinId, Pin>,
    kind: BoxKind,
}

pub fn resize_rect(rect: &Rect, mode: ResizeMode, delta: Vec2) -> Rect {
    match mode {
        ResizeMode::LeftTop => Rect::from_two_pos(rect.left_top() + delta, rect.right_bottom()),
        ResizeMode::RightTop => Rect::from_two_pos(rect.right_top() + delta, rect.left_bottom()),
        ResizeMode::LeftBottom => Rect::from_two_pos(rect.left_bottom() + delta, rect.right_top()),
        ResizeMode::RightBottom => Rect::from_two_pos(rect.right_bottom() + delta, rect.left_top()),
        ResizeMode::CenterTop => {
            Rect::from_two_pos(rect.left_top() + vec2(0.0, delta.y), rect.right_bottom())
        }
        ResizeMode::CenterBottom => {
            Rect::from_two_pos(rect.left_bottom() + vec2(0.0, delta.y), rect.right_top())
        }
    }
}

pub fn control_corner(rect: &Rect, mode: ResizeMode) -> Pos2 {
    match mode {
        ResizeMode::LeftTop => rect.left_top(),
        ResizeMode::RightTop => rect.right_top(),
        ResizeMode::LeftBottom => rect.left_bottom(),
        ResizeMode::RightBottom => rect.right_bottom(),
        ResizeMode::CenterTop => rect.center_top(),
        ResizeMode::CenterBottom => rect.center_bottom(),
    }
}

impl RectBox {
    pub fn pin(&self, id: PinId) -> Option<&Pin> {
        self.pins.get(id)
    }
    pub fn pins_mut(&mut self, id: PinId) -> Option<&mut Pin> {
        self.pins.get_mut(id)
    }
    pub fn iter_pins(&self) -> impl Iterator<Item = (PinId, &Pin)> + '_ {
        self.pins.iter()
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }
    pub fn kind(&self) -> BoxKind {
        self.kind
    }
    pub fn is_port(&self) -> bool {
        self.kind == BoxKind::Port
    }
    pub fn resize_modes(&self) -> &'static [ResizeMode] {
        if self.is_port() {
            &[
                ResizeMode::LeftTop,
                ResizeMode::RightTop,
                ResizeMode::LeftBottom,
                ResizeMode::RightBottom,
            ]
        } else {
            &[
                ResizeMode::LeftTop,
                ResizeMode::RightTop,
                ResizeMode::LeftBottom,
                ResizeMode::RightBottom,
                ResizeMode::CenterTop,
                ResizeMode::CenterBottom,
            ]
        }
    }
    pub fn new(name: String, inner: Rect) -> Self {
        Self {
            name,
            inner: snap(inner),
            pins: Store::default(),
            kind: BoxKind::Normal,
        }
    }
    pub fn new_port(pin_name: String, side: PinSide, inner: Rect) -> Self {
        let snapped = snap(inner);
        let clamped = Rect::from_min_size(snapped.min, vec2(snapped.width(), PORT_HEIGHT));
        let mut result = Self {
            name: String::new(),
            inner: clamped,
            pins: Store::default(),
            kind: BoxKind::Port,
        };
        result.add_pin(pin_name, side, 0.0);
        result
    }
    pub fn is_pin_offset_available(&self, side: PinSide, offset: f32) -> bool {
        if self.is_port() {
            return false;
        }
        if offset < 0.0 || offset > self.inner.height() {
            return false;
        }
        self.pins
            .values()
            .filter(|l| l.side == side)
            .all(|l| (l.offset - offset).abs() >= GRID_SIZE * 0.2)
    }
    pub fn update_pin_offset(&mut self, pin_id: PinId, delta_y: f32) {
        if self.is_port() {
            return;
        }
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
    pub fn next_pin_offset(&self, side: PinSide) -> Option<f32> {
        if self.is_port() {
            return None;
        }
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
    pub fn control_pin_location_east(&self) -> Option<Pos2> {
        if self.is_port() {
            return None;
        }
        let offset = self.next_pin_offset(PinSide::East)?;
        Some(self.inner.right_top() + vec2(GRID_SIZE, GRID_SIZE + offset))
    }
    pub fn control_pin_location_west(&self) -> Option<Pos2> {
        if self.is_port() {
            return None;
        }
        let offset = self.next_pin_offset(PinSide::West)?;
        Some(self.inner.left_top() + vec2(-GRID_SIZE, GRID_SIZE + offset))
    }
    pub fn pin_head_pos(&self, pin_id: PinId) -> Option<Pos2> {
        self.pins.get(pin_id).map(|pin| match pin.side {
            PinSide::East => self.inner.right_top() + vec2(GRID_SIZE, GRID_SIZE + pin.offset),
            PinSide::West => self.inner.left_top() + vec2(-GRID_SIZE, GRID_SIZE + pin.offset),
        })
    }
    pub fn anchor_point_with_rect(&self, rect: Rect, id: PinId) -> Option<Pos2> {
        self.pins.get(id).map(|pin| match pin.side {
            PinSide::East => pos2(
                rect.right() + GRID_SIZE,
                rect.top() + GRID_SIZE + pin.offset,
            ),
            PinSide::West => pos2(
                rect.left() - GRID_SIZE,
                rect.top() + GRID_SIZE + pin.offset,
            ),
        })
    }
    pub fn anchor_point(&self, id: PinId) -> Option<Pos2> {
        self.anchor_point_with_rect(self.inner, id)
    }
    pub fn add_pin(&mut self, text: String, side: PinSide, offset: f32) -> PinId {
        self.pins.insert(Pin { text, side, offset })
    }
    pub fn predicted_rect(&self) -> Rect {
        grid_rect(self.inner)
    }
    pub fn gui_rect(&self) -> Rect {
        self.inner
    }
    pub fn gui_rect_mut(&mut self) -> &mut Rect {
        &mut self.inner
    }
}
