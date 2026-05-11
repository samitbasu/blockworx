use egui::{Response, Ui};

use crate::{
    state::RenderMode,
    widget::{
        data::Data, drawing::LineAnchor, render::FocusResult, shape::BaseShape,
        tool::select::SubtoolState,
    },
};

pub struct RenamePin {
    anchor: LineAnchor,
    lost_focus: bool,
}

impl From<LineAnchor> for RenamePin {
    fn from(anchor: LineAnchor) -> Self {
        Self {
            anchor,
            lost_focus: false,
        }
    }
}

impl RenamePin {
    pub(crate) fn update(&mut self, response: &mut Response) -> SubtoolState {
        if self.lost_focus || response.clicked() || response.double_clicked() {
            response.mark_changed();
            return SubtoolState::Idle;
        }
        return SubtoolState::Active;
    }
    pub fn render(&mut self, data: &mut Data, ui: &mut Ui) {
        for (id, shape) in data.rect_boxes_mut() {
            if id == self.anchor.rect {
                self.lost_focus = shape.render(
                    RenderMode::EditingPinText {
                        pin: self.anchor.pin,
                    },
                    ui,
                ) == FocusResult::LostFocus;
            } else {
                shape.render(RenderMode::Normal, ui);
            }
        }
    }
}
