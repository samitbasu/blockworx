use std::{cell::RefCell, sync::Arc};

use egui::Rect;
use enum_dispatch::enum_dispatch;

use crate::{
    canvas::{Interaction, painter::Painter},
    widget::data::Data,
    widget_ng::{
        move_pin::MovePin, move_tool::MoveTool, names::ToolName, new_block::NewBlock,
        new_pin::NewPin, rename_pin::RenamePin,
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
}

pub struct EditLine {
    pub position: Rect,
    pub buffer: Arc<RefCell<String>>,
    pub font: egui::FontId,
    pub width: f32,
    pub id: egui::Id,
}

pub enum Action {
    SwitchTool(Tool),
    EditLine(EditLine),
}
