use egui::{Response, Ui, Vec2};

use crate::{
    state::RenderMode,
    widget::{
        data::Data, drawing::LineAnchor, pin::PinSide, shape::BaseShape, tool::select::SubtoolState,
    },
};

pub struct DragPin {
    anchor: LineAnchor,
    delta_pos: Vec2,
}

impl From<LineAnchor> for DragPin {
    fn from(anchor: LineAnchor) -> Self {
        Self {
            anchor,
            delta_pos: Vec2::ZERO,
        }
    }
}

impl DragPin {
    pub(crate) fn update(&mut self, data: &mut Data, response: &mut Response) -> SubtoolState {
        if response.dragged_by(egui::PointerButton::Primary)
            && let Some(rect) = data.rect_mut(self.anchor.rect)
            && let Some(pos) = response.interact_pointer_pos()
        {
            self.delta_pos += response.drag_delta();
            let side = if pos.x < rect.gui_rect().center().x {
                PinSide::West
            } else {
                PinSide::East
            };
            rect.with_pin_mut(self.anchor.pin, |p| p.side = side);
            return SubtoolState::Active;
        } else {
            if let Some(rbox) = data.rect_mut(self.anchor.rect) {
                rbox.update_pin_offset(self.anchor.pin, self.delta_pos.y);
            }
            response.mark_changed();
            return SubtoolState::Idle;
        }
    }
    pub fn render(&mut self, data: &mut Data, ui: &mut Ui) {
        for (id, shape) in data.rect_boxes_mut() {
            if id == self.anchor.rect {
                shape.render(
                    RenderMode::PinDragged {
                        pin: self.anchor.pin,
                        delta: self.delta_pos,
                    },
                    ui,
                );
            } else {
                shape.render(RenderMode::Normal, ui);
            }
        }
    }
}
