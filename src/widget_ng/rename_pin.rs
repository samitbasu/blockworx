use std::{cell::RefCell, sync::Arc};

use egui::{Rect, vec2};

use crate::{
    canvas::{Event, Interaction, painter::Painter},
    grid::PORT_TEXT_SIZE,
    widget::{data::Data, drawing::LineAnchor, shape::BaseShape},
    widget_ng::{
        names::ToolName,
        render::pin_text_location,
        tool::{Action, EditLine, Tool, ToolTrait},
    },
};

pub enum RenamePin {
    Idle,
    Renaming {
        anchor: LineAnchor,
        label: Arc<RefCell<String>>,
        position: Rect,
    },
}

impl ToolTrait for RenamePin {
    fn name(&self) -> ToolName {
        ToolName::RenamePin
    }

    fn widget(
        &mut self,
        data: &mut Data,
        interaction: &Interaction,
        painter: &mut Painter,
    ) -> Option<Action> {
        eprintln!("interaction {:?}", interaction);
        match self {
            RenamePin::Idle => {
                super::display::widget(data, interaction, painter);
                if let Some(Event::DoubleClicked { pos }) = interaction.event {
                    if let Some((anchor, _)) = data.pin_text_at_pos(pos)
                        && let Some(shape) = data.rect(anchor.rect)
                        && let Some(pin) = shape.pin(anchor.pin)
                    {
                        let editor_width =
                            ((pin.text.len() as f32 * PORT_TEXT_SIZE * 0.6 + 10.0) / 2.0).max(20.0);
                        let (pin_text_pos, pin_text_align) =
                            pin_text_location(shape.gui_rect(), pin, pin.side, 0.0);
                        let editor_position = pin_text_align.anchor_size(
                            pin_text_pos,
                            vec2(editor_width * 2.0, PORT_TEXT_SIZE * 1.5),
                        );
                        *self = RenamePin::Renaming {
                            anchor,
                            label: Arc::new(RefCell::new(pin.text.clone())),
                            position: editor_position,
                        };
                    }
                }
                None
            }
            RenamePin::Renaming {
                anchor,
                label,
                position,
            } => {
                super::display::widget(data, interaction, painter);
                Some(Action::EditLine(EditLine {
                    position: painter.remap_rect(*position),
                    buffer: label.clone(),
                    font: painter.remap_font(egui::FontId::monospace(PORT_TEXT_SIZE)),
                    width: f32::INFINITY,
                    id: "pin_name_edit".into(),
                }))
            }
        }
    }
}
