use egui::Pos2;

use crate::{
    canvas::{Event, Interaction, painter::Painter},
    grid::GRID_SIZE,
    widget::data::Data,
    widget_ng::{
        names::ToolName,
        tool::{Action, ToolTrait},
    },
};

pub enum NewBlock {
    Idle,
    Dragging { start: Pos2 },
}

impl ToolTrait for NewBlock {
    fn name(&self) -> ToolName {
        ToolName::NewBlock
    }
    fn widget(
        &mut self,
        data: &mut Data,
        interaction: &Interaction,
        painter: &mut Painter,
    ) -> Option<Action> {
        // Draw the background
        super::display::widget(data, interaction, painter);
        match self {
            NewBlock::Idle => {
                if let Some(Event::DragStarted { pos }) = interaction.event {
                    *self = NewBlock::Dragging { start: pos };
                }
            }
            NewBlock::Dragging { start } => {
                if let Some(Event::Dragging { pos, .. }) = interaction.event {
                    let rect = egui::Rect::from_two_pos(*start, pos);
                    painter.rect(
                        rect,
                        0.0,
                        egui::Color32::TRANSPARENT,
                        egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
                    );
                } else if let Some(Event::DragStopped { pos }) = interaction.event {
                    let start_pos = *start;
                    let end_pos = pos;
                    let candidate = egui::Rect::from_two_pos(start_pos, end_pos);
                    if candidate.width() > GRID_SIZE && candidate.height() > GRID_SIZE {
                        data.add_rect_box(start_pos, end_pos);
                    }
                    *self = NewBlock::Idle;
                }
            }
        }
        None
    }
}
