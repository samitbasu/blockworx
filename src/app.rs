use std::path::PathBuf;

use egui::{DragPanButtons, Rect, Scene};

use crate::{theme::Theme, widget::drawing::Drawing};

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
