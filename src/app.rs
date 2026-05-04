use std::path::PathBuf;

use egui::{DragPanButtons, Rect, Scene};

use crate::widget::drawing::Drawing;

enum Pane {
    Drawing,
}

pub struct App {
    filename: PathBuf,
    drawing: Drawing,
    pub scene_rect: Rect,
}

impl App {
    pub fn new(filename: PathBuf) -> Self {
        Self {
            filename,
            drawing: Drawing::demo(),
            scene_rect: Rect::ZERO,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let scene = Scene::new()
                .zoom_range(0.1..=4.0)
                .drag_pan_buttons(DragPanButtons::SECONDARY);
            let response = scene.show(ui, &mut self.scene_rect, |ui| {
                self.drawing.render(ui);
            });
            self.drawing.update_state(response.response);
        });
    }
}
