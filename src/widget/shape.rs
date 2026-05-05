use egui::{Pos2, Rect, Vec2};

use crate::{
    grid::grid_rect,
    state::ResizeMode,
    store::PinId,
    widget::pin::{Pin, PinSide},
};

pub trait Shape {
    fn name(&self) -> &str;
    fn name_mut(&mut self) -> &mut String;
    fn resize_modes(&self) -> Box<dyn Iterator<Item = ResizeMode> + '_>;
    fn predicted_rect(&self) -> Rect {
        grid_rect(self.gui_rect())
    }
    fn gui_rect(&self) -> Rect;
    fn gui_rect_mut(&mut self) -> &mut Rect;
    fn as_pin_shape(&self) -> Option<&dyn PinShape> {
        None
    }
    fn as_pin_shape_mut(&mut self) -> Option<&mut dyn PinShape> {
        None
    }
    fn constrain_resize_delta(&self, delta: Vec2) -> Vec2 {
        delta
    }
    fn apply_resize(&mut self, mode: ResizeMode, new_rect: Rect);
    /// Returns the port facing direction for shapes that render as a pentagon.
    /// Returns None for shapes that render as a normal rounded rectangle.
    fn port_side(&self) -> Option<PinSide> {
        None
    }
}

pub trait PinShape: Shape {
    fn pin(&self, id: PinId) -> Option<&Pin>;
    fn pins_mut(&mut self, id: PinId) -> Option<&mut Pin>;
    fn iter_pins(&self) -> Box<dyn Iterator<Item = (PinId, &Pin)> + '_>;
    fn pin_head_pos(&self, id: PinId) -> Option<Pos2>;
    fn anchor_point_with_rect(&self, rect: Rect, id: PinId) -> Option<Pos2>;
    fn anchor_point(&self, id: PinId) -> Option<Pos2> {
        self.anchor_point_with_rect(self.gui_rect(), id)
    }
    // Pin management — default no-ops, overridden by shapes that support it.
    fn add_pin_button_east(&self) -> Option<Pos2> {
        None
    }
    fn add_pin_button_west(&self) -> Option<Pos2> {
        None
    }
    fn next_pin_offset(&self, _side: PinSide) -> Option<f32> {
        None
    }
    fn add_pin(&mut self, _text: String, _side: PinSide, _offset: f32) -> Option<PinId> {
        None
    }
    fn update_pin_offset(&mut self, _pin_id: PinId, _delta_y: f32) {}
}
