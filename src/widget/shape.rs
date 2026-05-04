use egui::{Pos2, Rect, Ui};

use crate::{
    grid::grid_rect,
    state::{ResizeMode, State},
    store::{PinId, RectId},
    widget::pin::{Pin, PinSide},
};

pub trait Shape {
    fn name(&self) -> &str;
    fn name_mut(&mut self) -> &mut String;
    fn resize_modes(&self) -> impl Iterator<Item = ResizeMode> + '_;
    fn redicted_rect(&self) -> Rect {
        grid_rect(self.gui_rect())
    }
    fn gui_rect(&self) -> Rect;
    fn gui_rect_mut(&self) -> Rect;
    fn render(&mut self, id: RectId, state: &State, ui: &mut Ui);
}

pub trait PinShape: Shape {
    fn pin(&self, id: PinId) -> Option<&Pin>;
    fn pins_mut(&mut self, id: PinId) -> Option<&mut Pin>;
    fn iter_pins(&self) -> impl Iterator<Item = (PinId, &Pin)> + '_;
    fn pin_head_pos(&self, id: PinId) -> Option<Pos2>;
    fn anchor_point_with_rect(&self, rect: Rect, id: PinId) -> Option<Pos2>;
    fn anchor_point(&self, id: PinId) -> Option<Pos2> {
        self.anchor_point_with_rect(self.gui_rect(), id)
    }
}
