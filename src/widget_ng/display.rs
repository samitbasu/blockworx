use crate::{
    canvas::{Interaction, painter::Painter},
    state::RenderMode,
    widget::{data::Data, shape::BaseShape},
    widget_ng::route::{RouteRenderMode, render_route},
};

pub fn widget(data: &Data, _interaction: &Interaction, painter: &mut Painter) {
    for (_id, rect_box) in data.rect_boxes() {
        rect_box.render_ng(RenderMode::Normal, painter);
    }
    for (_, route) in data.auto_routes() {
        render_route(painter, route, RouteRenderMode::Normal);
    }
}
