use std::path::PathBuf;

// use egui::{DragPanButtons, Rect, Scene};
use egui::{Align2, FontId, Rect, Stroke, pos2};

use crate::{
    canvas::{Event, Interaction, View},
    theme::{Theme, get_theme},
    widget::drawing::Drawing, // Mode used in commented-out toolbar below
};

// enum Pane {
//     Drawing,
// }

pub struct App {
    #[allow(dead_code)]
    filename: PathBuf,
    #[allow(dead_code)]
    drawing: Drawing,
    #[allow(dead_code)]
    pub scene_rect: Rect,
    pub theme: Theme,
    canvas: View,
}

impl App {
    pub fn new(filename: PathBuf) -> Self {
        Self {
            filename,
            drawing: Drawing::demo(),
            scene_rect: Rect::ZERO,
            theme: Theme::default(),
            canvas: View::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, self.theme));
        egui::CentralPanel::default().show(ctx, |ui| {
            let theme = get_theme(ui);
            self.canvas.show(ui, |interaction, painter| {
                let demo_world = egui::Rect::from_min_max(pos2(60.0, 60.0), pos2(240.0, 150.0));
                let hovered =
                    matches!(interaction.event, Some(Event::HoverAt(p)) if demo_world.contains(p));
                let fill = if hovered {
                    theme.hover_fill
                } else {
                    theme.shape_fill
                };
                painter.rect(demo_world, 4.0, fill, Stroke::new(1.0, theme.shape_stroke));
                painter.text(
                    demo_world.center(),
                    Align2::CENTER_CENTER,
                    "Demo Block",
                    FontId::proportional(14.0),
                    theme.shape_title,
                );
            });

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
