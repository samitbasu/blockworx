use enum_dispatch::enum_dispatch;

use crate::{
    canvas::{Interaction, painter::Painter},
    widget::data::Data,
    widget_ng::{
        move_pin::MovePin, move_tool::MoveTool, names::ToolName, new_block::NewBlock,
        new_pin::NewPin,
    },
};

#[enum_dispatch(Tool)]
pub trait ToolTrait {
    fn name(&self) -> ToolName;
    fn widget(
        &mut self,
        data: &mut Data,
        interaction: &Interaction,
        painter: &mut Painter,
    ) -> Option<Tool>;
}

#[enum_dispatch]
pub enum Tool {
    Move(MoveTool),
    NewBlock(NewBlock),
    NewPin(NewPin),
    MovePin(MovePin),
}
