use std::path::PathBuf;

use egui::{DragPanButtons, Rect, Scene};

use crate::{
    grid::GRID_SIZE,
    theme::Theme,
    widget::drawing::{Drawing, Mode},
};

enum Pane {
    Drawing,
}

pub struct App {
    filename: PathBuf,
    drawing: Drawing,
    pub scene_rect: Rect,
    pub theme: Theme,
}

impl App {
    pub fn new(filename: PathBuf) -> Self {
        Self {
            filename,
            drawing: Drawing::demo(),
            scene_rect: Rect::ZERO,
            theme: Theme::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, self.theme));
        egui::CentralPanel::default().show(ctx, |ui| {
            let scene = if self.drawing.mode == Mode::Move {
                Scene::new()
                    .zoom_range(0.1..=4.0)
                    .drag_pan_buttons(DragPanButtons::all())
            } else {
                Scene::new()
                    .zoom_range(0.1..=4.0)
                    .drag_pan_buttons(DragPanButtons::SECONDARY)
            };
            let response = scene.show(ui, &mut self.scene_rect, |ui| {
                self.drawing.render(ui);
            });
            self.drawing.update_state(response.response);
            egui::Area::new(egui::Id::new("mode_toolbar"))
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, GRID_SIZE))
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for mode in [
                                Mode::Move,
                                Mode::Select,
                                Mode::Block,
                                Mode::Pin,
                                Mode::Route,
                            ] {
                                let label = match mode {
                                    Mode::Move => "Move",
                                    Mode::Select => "Select",
                                    Mode::Block => "Block",
                                    Mode::Pin => "Pin",
                                    Mode::Route => "Route",
                                };
                                if ui
                                    .add(
                                        egui::Button::new(label)
                                            .selected(self.drawing.mode == mode),
                                    )
                                    .clicked()
                                {
                                    self.drawing.mode = mode;
                                }
                            }
                        });
                    });
                });
        });
    }
}
