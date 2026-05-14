use egui::{CursorIcon, Vec2};

use crate::{
    canvas::{Event, Interaction, painter::Painter},
    grid::round_to_grid,
    state::RenderMode,
    widget::{
        data::Data,
        drawing::LineAnchor,
        pin::PinSide,
        shape::{BaseShape, PinLocation},
    },
    widget_ng::{
        names::ToolName,
        tool::{Action, Tool, ToolTrait},
    },
};

pub enum MovePin {
    Idle,
    Dragging {
        anchor: LineAnchor,
        location: PinLocation,
        delta_pos: Vec2,
    },
}

impl ToolTrait for MovePin {
    fn name(&self) -> ToolName {
        ToolName::MovePin
    }

    fn widget(
        &mut self,
        data: &mut Data,
        interaction: &Interaction,
        painter: &mut Painter,
    ) -> Option<Action> {
        match self {
            MovePin::Idle => {
                crate::widget_ng::display::widget(data, interaction, painter);
                if let Some(Event::HoverAt(hover_pos)) = interaction.event {
                    if data.pin_text_at_pos(hover_pos).is_some() {
                        painter.set_cursor(CursorIcon::PointingHand);
                    }
                } else if let Some(Event::DragStarted { pos }) = interaction.event {
                    if let Some((anchor, location)) = data.pin_text_at_pos(pos) {
                        *self = MovePin::Dragging {
                            anchor,
                            location,
                            delta_pos: Vec2::ZERO,
                        };
                    }
                }
            }
            MovePin::Dragging {
                anchor,
                location,
                delta_pos,
            } => {
                if let Some(Event::Dragging { pos, delta }) = interaction.event {
                    *delta_pos += delta;
                    if pos.x < data.rect(anchor.rect)?.gui_rect().center().x {
                        location.side = PinSide::West;
                    } else {
                        location.side = PinSide::East;
                    }
                }
                if let Some(Event::DragStopped { pos }) = interaction.event {
                    if pos.x < data.rect(anchor.rect)?.gui_rect().center().x {
                        location.side = PinSide::West;
                    } else {
                        location.side = PinSide::East;
                    }
                    location.offset = round_to_grid(location.offset + delta_pos.y);
                    if let Some(rbox) = data.rect_mut(anchor.rect) {
                        rbox.update_pin_location(anchor.pin, *location);
                    }
                    return Some(Action::SwitchTool(Tool::MovePin(MovePin::Idle)));
                }
                for (id, shape) in data.rect_boxes() {
                    if id == anchor.rect {
                        shape.render_ng(
                            RenderMode::PinDragged {
                                pin: anchor.pin,
                                delta: delta_pos.y,
                                side: location.side,
                            },
                            painter,
                        );
                    } else {
                        shape.render_ng(RenderMode::Normal, painter);
                    }
                }
            }
        }
        None
    }
}
