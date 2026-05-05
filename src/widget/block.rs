use egui::{Pos2, Rect, Vec2, pos2, vec2};

use crate::{
    grid::{GRID_SIZE, round_to_grid, snap},
    state::ResizeMode,
    store::*,
    widget::{
        pin::{Pin, PinSide},
        shape::{PinShape, Shape},
    },
};

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

const NORMAL_RESIZE_MODES: &[ResizeMode] = &[
    ResizeMode::LeftTop,
    ResizeMode::RightTop,
    ResizeMode::LeftBottom,
    ResizeMode::RightBottom,
    ResizeMode::CenterTop,
    ResizeMode::CenterBottom,
];

// ── RectBoxNormal ─────────────────────────────────────────────────────────────

pub struct Block {
    name: String,
    inner: Rect,
    pins: Store<PinId, Pin>,
}

impl Block {
    pub fn new(name: String, inner: Rect) -> Self {
        Self {
            name,
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
}

impl Shape for Block {
    fn name(&self) -> &str {
        &self.name
    }
    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }
    fn resize_modes(&self) -> Box<dyn Iterator<Item = ResizeMode> + '_> {
        Box::new(NORMAL_RESIZE_MODES.iter().copied())
    }
    fn gui_rect(&self) -> Rect {
        self.inner
    }
    fn gui_rect_mut(&mut self) -> &mut Rect {
        &mut self.inner
    }
    fn as_pin_shape(&self) -> Option<&dyn PinShape> {
        Some(self)
    }
    fn as_pin_shape_mut(&mut self) -> Option<&mut dyn PinShape> {
        Some(self)
    }
    fn apply_resize(&mut self, _mode: ResizeMode, new_rect: Rect) {
        self.inner = new_rect;
    }
}

impl PinShape for Block {
    fn pin(&self, id: PinId) -> Option<&Pin> {
        self.pins.get(id)
    }
    fn pins_mut(&mut self, id: PinId) -> Option<&mut Pin> {
        self.pins.get_mut(id)
    }
    fn iter_pins(&self) -> Box<dyn Iterator<Item = (PinId, &Pin)> + '_> {
        Box::new(self.pins.iter())
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
}

// ── RectBoxPort ───────────────────────────────────────────────────────────────
