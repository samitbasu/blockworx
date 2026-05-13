use egui::{Pos2, Rect, Ui, Vec2};
use enum_dispatch::enum_dispatch;

use crate::{
    canvas::painter::Painter,
    grid::grid_rect,
    state::{RenderMode, ResizeMode},
    store::PinId,
    widget::{
        block::{Block, Title},
        pin::{Pin, PinSide},
        port::Port,
        render::FocusResult,
    },
};

#[derive(Copy, Clone, Debug)]
pub struct PinLocation {
    pub side: PinSide,
    pub offset: f32,
}

impl Into<PinLocation> for (PinSide, f32) {
    fn into(self) -> PinLocation {
        PinLocation {
            side: self.0,
            offset: self.1,
        }
    }
}

/// BaseShape provides a name, and can be resized.
/// The BaseShape also provides a Pin management API,
/// but shapes without Pins can simply use the default
/// impl for those methods (which means they have no pins).
#[enum_dispatch(Shape)]
pub trait BaseShape {
    fn title(&self) -> Option<&Title> {
        None
    }
    fn title_mut(&mut self) -> Option<&mut Title> {
        None
    }
    fn name(&self) -> &str;
    fn name_mut(&mut self) -> &mut String;
    fn resize_modes(&self) -> &'static [ResizeMode];
    fn predicted_rect(&self) -> Rect {
        grid_rect(self.gui_rect())
    }
    fn gui_rect(&self) -> Rect;
    fn gui_rect_mut(&mut self) -> &mut Rect;
    fn constrain_resize_delta(&self, delta: Vec2) -> Vec2 {
        delta
    }
    fn apply_resize(&mut self, mode: ResizeMode, new_rect: Rect);
    fn pin(&self, id: PinId) -> Option<&Pin> {
        let _ = id;
        None
    }
    fn pins_mut(&mut self, id: PinId) -> Option<&mut Pin> {
        let _ = id;
        None
    }
    fn with_pin_mut(&mut self, id: PinId, mut f: impl FnMut(&mut Pin)) {
        if let Some(pin) = self.pins_mut(id) {
            f(pin);
        }
    }
    fn with_pins(&self, f: impl FnMut(PinId, &Pin)) {
        let _ = f;
    }
    fn find_pin<T>(&self, mut f: impl FnMut(PinId, &Pin) -> Option<T>) -> Option<T> {
        let mut ret = None;
        self.with_pins(|id, pin| {
            if ret.is_none() {
                ret = f(id, pin);
            }
        });
        ret
    }
    fn pin_head_pos(&self, id: PinId) -> Option<Pos2> {
        let _ = id;
        None
    }
    fn anchor_point_with_rect(&self, rect: Rect, id: PinId) -> Option<Pos2> {
        let _ = (rect, id);
        None
    }
    fn anchor_point(&self, id: PinId) -> Option<Pos2> {
        self.anchor_point_with_rect(self.gui_rect(), id)
    }
    fn new_pin_location(&self, pos: Pos2) -> Option<PinLocation> {
        let _ = pos;
        None
    }
    fn pin_position(&self, location: PinLocation) -> Option<Pos2> {
        let _ = location;
        None
    }
    fn add_pin(&mut self, _text: String, location: impl Into<PinLocation>) -> Option<PinId> {
        let _ = location;
        None
    }
    fn update_pin_location(&mut self, pin_id: PinId, location: PinLocation) {
        let _ = (pin_id, location);
    }
    fn render(&mut self, mode: RenderMode, ui: &mut Ui) -> FocusResult;
    fn render_ng(&self, mode: RenderMode, painter: &mut Painter) {
        let _ = mode;
        let _ = painter;
    }
    fn new_pin_locations(&self) -> Vec<PinLocation> {
        Vec::new()
    }
    fn title_anchor(&self) -> Option<Pos2> {
        None
    }
}

#[enum_dispatch]
pub enum Shape {
    Block(Block),
    Port(Port),
}
