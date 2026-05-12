use crate::{
    grid::GRID_SIZE,
    widget_ng::{
        move_tool::MoveTool,
        names::{TOOL_NAMES, ToolName},
        new_block::NewBlock,
        new_pin::NewPin,
        tool::{Tool, ToolTrait},
    },
};

pub fn toolbar(tool: &mut Tool, ctx: &egui::Context) {
    egui::Area::new(egui::Id::new("mode_toolbar"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, GRID_SIZE))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    for mode in TOOL_NAMES {
                        let label = mode.to_string();
                        if ui
                            .add(egui::Button::new(label).selected(&tool.name() == mode))
                            .clicked()
                        {
                            *tool = match mode {
                                ToolName::Move => Tool::Move(MoveTool),
                                ToolName::NewBlock => Tool::NewBlock(NewBlock::Idle),
                                ToolName::NewPin => Tool::NewPin(NewPin),
                            };
                        }
                    }
                });
            });
        });
}
