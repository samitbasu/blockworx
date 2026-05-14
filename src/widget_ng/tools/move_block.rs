use egui::{CursorIcon, Vec2};

use crate::{
    canvas::{Event, Interaction, painter::Painter},
    state::RenderMode,
    store::RectId,
    widget::{data::Data, shape::BaseShape},
    widget_ng::{
        names::ToolName,
        route::{RouteRenderMode, render_route},
        tool::{Action, ToolTrait},
    },
};

pub enum MoveBlock {
    Idle,
    Dragging { rect: RectId, delta_pos: Vec2 },
}

impl ToolTrait for MoveBlock {
    fn name(&self) -> ToolName {
        ToolName::MoveBlock
    }

    fn widget(
        &mut self,
        data: &mut Data,
        interaction: &Interaction,
        painter: &mut Painter,
    ) -> Option<Action> {
        self.render(data, interaction, painter);
        match self {
            MoveBlock::Idle => {
                if let Some(Event::HoverAt(hover_pos)) = interaction.event {
                    if data.block_at_pos(hover_pos).is_some() {
                        painter.set_cursor(CursorIcon::Move);
                    }
                }
                if let Some(Event::DragStarted { pos }) = interaction.event {
                    if let Some(block) = data.block_at_pos(pos) {
                        *self = MoveBlock::Dragging {
                            rect: block,
                            delta_pos: Vec2::ZERO,
                        };
                    }
                }
            }
            MoveBlock::Dragging { rect, delta_pos } => {
                if let Some(Event::Dragging { delta, .. }) = interaction.event {
                    *delta_pos += delta;
                } else if let Some(Event::DragStopped { .. }) = interaction.event {
                    data.move_block(*rect, *delta_pos);
                    *self = MoveBlock::Idle;
                }
                data.update_routes(&[]);
            }
        }
        None
    }
}

impl MoveBlock {
    fn render(&self, data: &Data, interaction: &Interaction, painter: &mut Painter) {
        match self {
            MoveBlock::Idle => {
                crate::widget_ng::display::widget(data, interaction, painter);
            }
            MoveBlock::Dragging { rect, delta_pos } => {
                for (id, rect_box) in data.rect_boxes() {
                    if id != *rect {
                        rect_box.render_ng(RenderMode::Normal, painter);
                    } else {
                        rect_box.render_ng(RenderMode::Moving { delta: *delta_pos }, painter);
                    }
                }
                for (_, route) in data.auto_routes() {
                    render_route(painter, route, RouteRenderMode::Normal);
                }
            }
        }
    }
}
