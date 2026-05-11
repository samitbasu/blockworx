use egui::{PointerButton, Pos2, Sense, Stroke, Vec2, pos2};

use crate::grid::GRID_SIZE;
use crate::theme::get_theme;

use super::interaction::{Interaction, compute_interaction};
use super::painter::Painter;

pub struct View {
    pub zoom: f32,
    pub translation: Vec2,
}

impl Default for View {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            translation: Vec2::ZERO,
        }
    }
}

impl View {
    fn world_to_screen(&self, origin: Pos2, world: Pos2) -> Pos2 {
        origin + self.translation + world.to_vec2() * self.zoom
    }

    fn screen_to_world(&self, origin: Pos2, screen: Pos2) -> Pos2 {
        ((screen - origin - self.translation) / self.zoom).to_pos2()
    }

    /// Show the canvas. Draws the grid, then calls `f` with the current interaction
    /// event (positions in world space) and a world-space-aware painter.
    pub fn show<F>(&mut self, ui: &mut egui::Ui, f: F)
    where
        F: FnOnce(Interaction, &mut Painter),
    {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let origin = rect.min;

        // Zoom on scroll (centered on cursor)
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 {
            let new_zoom = (self.zoom * (scroll_delta * 0.002_f32).exp()).clamp(0.1, 10.0);
            let cursor = ui.ctx().pointer_hover_pos().unwrap_or(rect.center());
            let p = cursor.to_vec2() - origin.to_vec2() - self.translation;
            self.translation = cursor.to_vec2() - origin.to_vec2() - p * (new_zoom / self.zoom);
            self.zoom = new_zoom;
        }

        // Pan on right-click or middle-click drag
        if response.dragged_by(PointerButton::Secondary)
            || response.dragged_by(PointerButton::Middle)
        {
            self.translation += response.drag_delta();
        }

        let egui_painter = ui.painter().with_clip_rect(rect);
        let theme = get_theme(ui);

        // Grid — drawn internally so callers only need to draw content
        let world_min = self.screen_to_world(origin, rect.min);
        let world_max = self.screen_to_world(origin, rect.max);

        let major_step: i32 = 4;
        let major_stroke = Stroke::new(1.0, theme.grid_line.gamma_multiply(0.6));
        let minor_stroke = Stroke::new(0.5, theme.grid_line.gamma_multiply(0.6));
        let draw_minor = GRID_SIZE * self.zoom >= 5.0;

        let first_i = (world_min.x / GRID_SIZE).floor() as i32;
        let last_i = (world_max.x / GRID_SIZE).ceil() as i32;
        for i in first_i..=last_i {
            let is_major = i % major_step == 0;
            if !is_major && !draw_minor {
                continue;
            }
            let sx = self
                .world_to_screen(origin, pos2(i as f32 * GRID_SIZE, 0.0))
                .x;
            egui_painter.vline(
                sx,
                rect.y_range(),
                if is_major { major_stroke } else { minor_stroke },
            );
        }

        let first_j = (world_min.y / GRID_SIZE).floor() as i32;
        let last_j = (world_max.y / GRID_SIZE).ceil() as i32;
        for j in first_j..=last_j {
            let is_major = j % major_step == 0;
            if !is_major && !draw_minor {
                continue;
            }
            let sy = self
                .world_to_screen(origin, pos2(0.0, j as f32 * GRID_SIZE))
                .y;
            egui_painter.hline(
                rect.x_range(),
                sy,
                if is_major { major_stroke } else { minor_stroke },
            );
        }

        // Bundle mouse event + keyboard flags into a single Interaction value
        let interaction = compute_interaction(
            &response,
            |p| self.screen_to_world(origin, p),
            self.zoom,
            ui,
        );

        // Hand off to the caller's drawing closure
        let mut painter = Painter::new(egui_painter, origin, self.zoom, self.translation);
        f(interaction, &mut painter);
    }
}
