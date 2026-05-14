use enum_dispatch::enum_dispatch;

use crate::{
    canvas::{Interaction, painter::Painter},
    widget::data::Data,
    widget_ng::{
        names::ToolName,
        tools::{MoveBlock, MovePin, MoveTool, NewBlock, NewPin, RenamePin, RouteTool},
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
    ) -> Option<Action>;
}

#[enum_dispatch]
pub enum Tool {
    Move(MoveTool),
    NewBlock(NewBlock),
    NewPin(NewPin),
    MovePin(MovePin),
    RenamePin(RenamePin),
    Route(RouteTool),
    MoveBlock(MoveBlock),
}

pub enum Action {
    SwitchTool(Tool),
}
