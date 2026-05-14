use crate::{
    canvas::{Interaction, painter::Painter},
    widget::data::Data,
    widget_ng::{
        names::ToolName,
        tool::{Action, ToolTrait},
    },
};

pub struct MoveTool;

impl ToolTrait for MoveTool {
    fn name(&self) -> ToolName {
        ToolName::Move
    }

    fn widget(
        &mut self,
        data: &mut Data,
        interaction: &Interaction,
        painter: &mut Painter,
    ) -> Option<Action> {
        super::display::widget(data, interaction, painter);
        None
    }
}
