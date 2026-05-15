use egui::Vec2;

use crate::{
    canvas::{Event, Interaction, painter::Painter},
    grid::{PORT_RADIUS, RESIZE_SHIM, grid_rect},
    state::{RenderMode, ResizeMode},
    store::RectId,
    widget::{block::resize_rect, data::Data, shape::BaseShape},
    widget_ng::{
        names::ToolName,
        route::render_route,
        tool::{Action, ToolTrait},
    },
};

#[derive(Default)]
pub enum ResizeBlock {
    #[default]
    Idle,
    Selected {
        rect: RectId,
    },
    ResizeRect {
        rect: RectId,
        mode: ResizeMode,
        delta_pos: Vec2,
    },
}

impl ToolTrait for ResizeBlock {
    fn name(&self) -> ToolName {
        ToolName::ResizeBlock
    }

    fn widget(
        &mut self,
        data: &mut Data,
        interaction: &Interaction,
        painter: &mut Painter,
    ) -> Option<Action> {
        self.render(data, interaction, painter);
        let state = std::mem::take(self);
        match state {
            ResizeBlock::Idle => {
                if let Some(Event::Clicked { pos }) = interaction.event
                    && let Some(block) = data.block_at_pos(pos)
                {
                    eprintln!("Selected block {block}");
                    *self = ResizeBlock::Selected { rect: block };
                    return None;
                }
            }
            ResizeBlock::Selected { rect } => {
                if let Some(Event::Clicked { pos }) = interaction.event {
                    if let Some(block) = data.block_at_pos(pos) {
                        if block != rect {
                            eprintln!("Selected block {block}");
                            *self = ResizeBlock::Selected { rect: block };
                            return None;
                        }
                    } else {
                        *self = ResizeBlock::Idle;
                        return None;
                    }
                }
                if let Some(Event::DragStarted { pos }) = interaction.event
                    && let Some(block) = data.rect(rect)
                    && block.resizable()
                {
                    let selection_rect = block.gui_rect().expand(RESIZE_SHIM);
                    let resize_mode = if pos.distance(selection_rect.left_top()) < PORT_RADIUS {
                        Some(ResizeMode::LeftTop)
                    } else if pos.distance(selection_rect.right_top()) < PORT_RADIUS {
                        Some(ResizeMode::RightTop)
                    } else if pos.distance(selection_rect.left_bottom()) < PORT_RADIUS {
                        Some(ResizeMode::LeftBottom)
                    } else if pos.distance(selection_rect.right_bottom()) < PORT_RADIUS {
                        Some(ResizeMode::RightBottom)
                    } else {
                        None
                    };
                    if let Some(mode) = resize_mode {
                        *self = ResizeBlock::ResizeRect {
                            rect,
                            mode,
                            delta_pos: Vec2::ZERO,
                        };
                        return None;
                    }
                }
            }
            ResizeBlock::ResizeRect {
                rect,
                mode,
                delta_pos,
            } => {
                if let Some(Event::Dragging { delta, .. }) = interaction.event {
                    *self = ResizeBlock::ResizeRect {
                        rect,
                        mode,
                        delta_pos: delta_pos + delta,
                    };
                    data.update_routes(&[]);
                    return None;
                }
                if let Some(Event::DragStopped { .. }) = interaction.event {
                    if let Some(block) = data.rect_mut(rect) {
                        let bbox = block.gui_rect();
                        let resized = resize_rect(&bbox, mode, delta_pos);
                        let predicted = grid_rect(resized);
                        block.apply_resize(mode, predicted);
                    }
                    *self = ResizeBlock::Selected { rect };
                    data.update_routes(&[]);
                    return None;
                }
            }
        }
        *self = state;
        None
    }
}

impl ResizeBlock {
    fn render(&self, data: &Data, interaction: &Interaction, painter: &mut Painter) {
        match self {
            ResizeBlock::Idle => {
                crate::widget_ng::display::widget(data, interaction, painter);
            }
            ResizeBlock::Selected { rect } => {
                for (id, rect_box) in data.rect_boxes() {
                    if id != *rect {
                        rect_box.render_ng(RenderMode::Normal, painter);
                    } else {
                        rect_box.render_ng(RenderMode::Selected, painter);
                    }
                }
                for (_, route) in data.auto_routes() {
                    render_route(
                        painter,
                        route,
                        crate::widget_ng::route::RouteRenderMode::Normal,
                    );
                }
            }
            ResizeBlock::ResizeRect {
                rect,
                mode,
                delta_pos,
            } => {
                for (id, rect_box) in data.rect_boxes() {
                    if id != *rect {
                        rect_box.render_ng(RenderMode::Normal, painter);
                    } else {
                        rect_box.render_ng(
                            RenderMode::Resizing {
                                mode: *mode,
                                delta: *delta_pos,
                            },
                            painter,
                        );
                    }
                }
                for (_, route) in data.auto_routes() {
                    render_route(
                        painter,
                        route,
                        crate::widget_ng::route::RouteRenderMode::Normal,
                    );
                }
            }
        }
    }
}
