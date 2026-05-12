use egui::{Align2, Color32, Pos2, Rect, Stroke, vec2};

use crate::{
    canvas::painter::Painter,
    grid::{PORT_RADIUS, ROUTE_TEXT_SIZE, SHIM},
    widget::{auto_route::AutoRoute, direction::RouteDirection},
    widget_ng::render::render_path_with_chamfered_corners,
};

pub enum RouteRenderMode {
    Normal,
    Highlighted,
    Selected,
}

fn draw_text_anchor(painter: &mut Painter, ta: Pos2, fill: Color32, stroke: impl Into<Stroke>) {
    let stroke: Stroke = stroke.into();
    painter.add_convex_polygon(
        [
            ta + vec2(0.0, -PORT_RADIUS),
            ta + vec2(PORT_RADIUS, 0.0),
            ta + vec2(0.0, PORT_RADIUS),
            ta + vec2(-PORT_RADIUS, 0.0),
        ]
        .into(),
        fill,
        stroke,
    );
}

fn draw_add_text_button(painter: &mut Painter, at: Pos2, fill: Color32, stroke: impl Into<Stroke>) {
    let stroke: Stroke = stroke.into();
    painter.rect(
        Rect::from_center_size(at, vec2(PORT_RADIUS * 2.0, PORT_RADIUS * 2.0)),
        PORT_RADIUS / 4.0,
        fill,
        stroke,
    );
    painter.text(
        at,
        Align2::CENTER_CENTER,
        "T",
        egui::FontId::monospace(ROUTE_TEXT_SIZE * 0.8),
        painter.theme().route_in_progress,
    );
}

pub(crate) fn render_route(painter: &mut Painter, route: &AutoRoute, mode: RouteRenderMode) {
    let route_stroke = match mode {
        RouteRenderMode::Normal => (1.7, painter.theme().route_normal),
        RouteRenderMode::Highlighted => (2.5, painter.theme().route_highlighted),
        RouteRenderMode::Selected => (2.5, painter.theme().route_selected),
    };
    let points = render_path_with_chamfered_corners(&route.points());
    points.render(painter, route_stroke);
    let text_color = match mode {
        RouteRenderMode::Normal => painter.theme().route_normal,
        RouteRenderMode::Highlighted => painter.theme().route_highlighted,
        RouteRenderMode::Selected => painter.theme().route_selected,
    };
    for (_, label) in route.iter_labels() {
        let loc_and_direction = route.map_linear_distance_to_position(label.linear_distance);
        let pos = loc_and_direction.location;
        match loc_and_direction.direction {
            RouteDirection::Horizontal => {
                painter.text(
                    pos + vec2(0.0, -SHIM / 4.0),
                    egui::Align2::CENTER_BOTTOM,
                    &label.text,
                    egui::FontId::monospace(ROUTE_TEXT_SIZE),
                    text_color,
                );
            }
            RouteDirection::Vertical => {
                painter.rotated_text(
                    pos,
                    Align2::LEFT_BOTTOM,
                    &label.text,
                    egui::FontId::monospace(ROUTE_TEXT_SIZE),
                    text_color,
                    std::f32::consts::FRAC_PI_2,
                );
            }
        }
    }
    if matches!(mode, RouteRenderMode::Selected) {
        for (_, wp) in route.iter_waypoints() {
            painter.circle(
                wp.pos,
                PORT_RADIUS,
                painter.theme().waypoint_fill,
                (0.5, painter.theme().control_handle_stroke),
            );
        }
        for dh in route.drag_handles() {
            painter.rect(
                Rect::from_center_size(dh, vec2(PORT_RADIUS * 2.0, PORT_RADIUS * 2.0)),
                PORT_RADIUS / 4.0,
                painter.theme().waypoint_fill,
                (0.5, painter.theme().control_handle_stroke),
            );
        }
        for ta in route.text_anchors() {
            draw_text_anchor(
                painter,
                ta,
                painter.theme().waypoint_fill,
                (0.5, painter.theme().control_handle_stroke),
            );
        }
        for at in route.all_add_text_buttons() {
            draw_add_text_button(
                painter,
                at.pos,
                painter.theme().add_button_fill,
                (0.5, painter.theme().control_handle_stroke),
            );
        }
    }
}
