use egui::{Color32, Pos2, Rect, Response, StrokeKind, Ui};

use crate::{
    grid::{GRID_SIZE, snap_to_grid},
    theme::get_theme,
};

pub struct AddingRect {
    pub start_pos: Pos2,
    pub end_pos: Pos2,
}

#[derive(Default)]
enum NewBlockState {
    #[default]
    Idle,
    AddingRect(AddingRect),
}

#[derive(Default)]
pub struct NewBlock {
    state: NewBlockState,
}

impl NewBlock {
    pub(crate) fn update(&mut self, response: &Response) -> Option<(Pos2, Pos2)> {
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let snapped = snap_to_grid(pos);
                self.state = NewBlockState::AddingRect(AddingRect {
                    start_pos: snapped,
                    end_pos: snapped,
                });
            }
            return None;
        }

        match &mut self.state {
            NewBlockState::Idle => None,
            NewBlockState::AddingRect(inner) => {
                if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        inner.start_pos = snap_to_grid(inner.start_pos);
                        inner.end_pos = snap_to_grid(pos);
                    }
                    None
                } else if response.drag_stopped() {
                    let start_pos = inner.start_pos;
                    let end_pos = inner.end_pos;
                    self.state = NewBlockState::Idle;
                    let candidate = Rect::from_two_pos(start_pos, end_pos);
                    if candidate.width() > GRID_SIZE && candidate.height() > GRID_SIZE {
                        Some((start_pos, end_pos))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn render(&self, ui: &mut Ui) {
        if let NewBlockState::AddingRect(AddingRect { start_pos, end_pos }) = &self.state {
            let rect = Rect::from_two_pos(*start_pos, *end_pos);
            let theme = get_theme(ui);
            ui.painter().rect(
                rect,
                3.0,
                Color32::TRANSPARENT,
                (1.0, theme.selection_frame),
                StrokeKind::Middle,
            );
        }
    }
}
