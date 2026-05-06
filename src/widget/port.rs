use egui::{Pos2, Rect, Vec2, pos2, vec2};

use crate::{
    grid::{GRID_SIZE, PORT_HEIGHT, snap},
    state::ResizeMode,
    store::PinId,
    widget::{
        pin::{Pin, PinSide},
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
}
