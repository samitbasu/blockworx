use std::path::PathBuf;

// use egui::{DragPanButtons, Rect, Scene};
use egui::{Rect, TextEdit};

use crate::{
    canvas::View,
    theme::Theme,
    widget::drawing::Drawing,
    widget_ng::{
        move_tool::MoveTool,
        tool::{Action, Tool, ToolTrait},
        toolbar::toolbar,
    }, // Mode used in commented-out toolbar below
};

pub struct App {
    #[allow(dead_code)]
    filename: PathBuf,
    #[allow(dead_code)]
    drawing: Drawing,
    #[allow(dead_code)]
    pub scene_rect: Rect,
    pub theme: Theme,
    canvas: View,
    tool: Tool,
    focus_lost: bool,
}

impl App {
    pub fn new(filename: PathBuf) -> Self {
        Self {
            filename,
            drawing: Drawing::demo(),
            scene_rect: Rect::ZERO,
            theme: Theme::default(),
            canvas: View::new(Theme::default()),
            tool: Tool::Move(MoveTool),
            focus_lost: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, self.theme));
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut ui_cursor = None;
            let mut editor = None;
            self.canvas.show(ui, |mut interaction, painter| {
                interaction.lost_focus |= self.focus_lost;
                let action = self
                    .tool
                    .widget(self.drawing.data_mut(), &interaction, painter);
                match action {
                    Some(Action::SwitchTool(next_tool)) => {
                        self.drawing.data_mut().update_routes(&[]);
                        self.tool = next_tool;
                    }
                    Some(Action::EditLine(edit_line)) => {
                        editor = Some(edit_line);
                    }
                    None => { /* No action */ }
                }
                ui_cursor = painter.cursor();
            });
            self.focus_lost = false;
            if let Some(editor) = editor {
                let mut buffer = editor.buffer.borrow_mut();
                let response = ui.place(
                    editor.position,
                    TextEdit::singleline(&mut *buffer)
                        .id(editor.id)
                        .desired_width(editor.width)
                        .font(editor.font),
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.focus_lost = true;
                }
            }
            if let Some(cursor) = ui_cursor {
                ui.output_mut(|o| {
                    o.cursor_icon = cursor;
                });
            }
            toolbar(&mut self.tool, ctx);

            // Old Scene-based drawing (preserved for incremental migration):
            // let scene = if self.drawing.mode == Mode::Move {
            //     Scene::new()
            //         .zoom_range(0.1..=4.0)
            //         .drag_pan_buttons(DragPanButtons::all())
            // } else {
            //     Scene::new()
            //         .zoom_range(0.1..=4.0)
            //         .drag_pan_buttons(DragPanButtons::SECONDARY)
            // };
            // let response = scene.show(ui, &mut self.scene_rect, |ui| {
            //     self.drawing.render(ui);
            // });
            // self.drawing.update_state(response.response);
            // egui::Area::new(egui::Id::new("mode_toolbar"))
            //     .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, GRID_SIZE))
            //     .show(ctx, |ui| {
            //         egui::Frame::popup(ui.style()).show(ui, |ui| {
            //             ui.horizontal(|ui| {
            //                 for mode in [
            //                     Mode::Move,
            //                     Mode::Select,
            //                     Mode::Block,
            //                     Mode::Pin,
            //                     Mode::Route,
            //                 ] {
            //                     let label = match mode {
            //                         Mode::Move => "Move",
            //                         Mode::Select => "Select",
            //                         Mode::Block => "Block",
            //                         Mode::Pin => "Pin",
            //                         Mode::Route => "Route",
            //                     };
            //                     if ui
            //                         .add(
            //                             egui::Button::new(label)
            //                                 .selected(self.drawing.mode == mode),
            //                         )
            //                         .clicked()
            //                     {
            //                         self.drawing.mode = mode;
            //                     }
            //                 }
            //             });
            //         });
            //     });
        });
    }
}
