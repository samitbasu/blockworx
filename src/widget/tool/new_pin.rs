use egui::{Color32, Response, Ui};

use crate::{
    store::RectId,
    widget::{
        data::{AddPinLocation, Data},
        pin::PinSide,
    },
};

#[derive(Default)]
enum PinState {
    #[default]
    Idle,
    Hovered(AddPinLocation),
}

#[derive(Default)]
pub struct NewPin {
    state: PinState,
}

impl NewPin {
    pub(crate) fn update(&mut self, data: &mut Data, response: &mut Response) {
        match &mut self.state {
            PinState::Idle => {
                if response.clicked()
                    && let Some(pos) = response.interact_pointer_pos()
                    && let Some(pin_loc) = data.new_pin_location(pos)
                {
                    response.mark_changed();
                    data.add_new_pin(pin_loc);
                } else if let Some(pos) = response.hover_pos()
                    && let Some(pin_loc) = data.new_pin_location(pos)
                {
                    response.mark_changed();
                    self.state = PinState::Hovered(pin_loc);
                }
            }
            PinState::Hovered(pin_loc) => {
                if response.clicked()
                    && let Some(pos) = response.interact_pointer_pos()
                    && let Some(new_pin_loc) = data.new_pin_location(pos)
                {
                    response.mark_changed();
                    data.add_new_pin(new_pin_loc);
                } else if let Some(pos) = response.hover_pos()
                    && let Some(new_pin_loc) = data.new_pin_location(pos)
                {
                    self.state = PinState::Hovered(new_pin_loc);
                    response.mark_changed();
                } else {
                    self.state = PinState::Idle;
                }
            }
        }
    }
    pub(crate) fn render(&self, ui: &mut Ui) {
        if let PinState::Hovered(AddPinLocation {
            rect,
            side,
            offset,
            pos,
        }) = &self.state
        {
            ui.painter().circle_filled(*pos, 5.0, Color32::GREEN);
        }
    }
}
