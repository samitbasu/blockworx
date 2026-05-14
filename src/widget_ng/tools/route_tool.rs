use egui::Pos2;

use crate::{
    canvas::{Event, Interaction, painter::Painter},
    grid::{PORT_RADIUS, snap_to_grid},
    router::TaggedPoint,
    store::{Store, WaypointId},
    widget::{auto_route::AutoRoute, data::Data, drawing::LineAnchor, waypoint::Waypoint},
    widget_ng::{
        names::ToolName,
        render::render_path_with_chamfered_corners,
        tool::{Action, ToolTrait},
    },
};

#[derive(Default)]
enum RouteToolState {
    #[default]
    Idle,
    PinHeadHovered {
        anchor: LineAnchor,
    },
    InProgress {
        start: LineAnchor,
        waypoints: Store<WaypointId, Waypoint>,
        head: Pos2,
    },
    Proposed {
        start: LineAnchor,
        waypoints: Store<WaypointId, Waypoint>,
        finish: LineAnchor,
    },
}

#[derive(Default)]
pub struct RouteTool {
    state: RouteToolState,
    preview_path: Vec<TaggedPoint>,
}

impl ToolTrait for RouteTool {
    fn name(&self) -> ToolName {
        ToolName::Route
    }

    fn widget(
        &mut self,
        data: &mut Data,
        interaction: &Interaction,
        painter: &mut Painter,
    ) -> Option<Action> {
        // Take ownership of the current state, leaving Idle in place.
        // This avoids borrow-checker issues when transitioning states.
        let state = std::mem::take(&mut self.state);

        crate::widget_ng::display::widget(data, interaction, painter);

        self.state = match state {
            RouteToolState::Idle => match interaction.event {
                Some(Event::DragStarted { pos } | Event::Clicked { pos }) => {
                    if let Some(anchor) = data.anchor_at_pos(pos) {
                        RouteToolState::InProgress {
                            start: anchor,
                            waypoints: Store::default(),
                            head: pos,
                        }
                    } else {
                        RouteToolState::Idle
                    }
                }
                Some(Event::HoverAt(pos)) => {
                    if let Some(anchor) = data.anchor_at_pos(pos) {
                        RouteToolState::PinHeadHovered { anchor }
                    } else {
                        RouteToolState::Idle
                    }
                }
                _ => RouteToolState::Idle,
            },

            RouteToolState::PinHeadHovered { anchor } => match interaction.event {
                Some(Event::DragStarted { pos } | Event::Clicked { pos }) => {
                    RouteToolState::InProgress {
                        start: anchor,
                        waypoints: Store::default(),
                        head: pos,
                    }
                }
                Some(Event::HoverAt(pos)) => {
                    if let Some(new_anchor) = data.anchor_at_pos(pos) {
                        RouteToolState::PinHeadHovered { anchor: new_anchor }
                    } else {
                        RouteToolState::Idle
                    }
                }
                _ => RouteToolState::PinHeadHovered { anchor },
            },

            RouteToolState::InProgress {
                start,
                mut waypoints,
                mut head,
            } => match interaction.event {
                Some(Event::Clicked { pos }) => {
                    waypoints.insert(Waypoint {
                        pos: snap_to_grid(pos),
                        locked: true,
                    });
                    RouteToolState::InProgress {
                        start,
                        waypoints,
                        head,
                    }
                }
                Some(Event::HoverAt(pos)) => {
                    if let Some(finish) = data.anchor_at_pos(pos) {
                        if finish != start {
                            RouteToolState::Proposed {
                                start,
                                waypoints,
                                finish,
                            }
                        } else {
                            RouteToolState::InProgress {
                                start,
                                waypoints,
                                head,
                            }
                        }
                    } else {
                        head = pos;
                        RouteToolState::InProgress {
                            start,
                            waypoints,
                            head,
                        }
                    }
                }
                _ => RouteToolState::InProgress {
                    start,
                    waypoints,
                    head,
                },
            },

            RouteToolState::Proposed {
                start,
                waypoints,
                mut finish,
            } => match interaction.event {
                Some(Event::Clicked { .. }) => {
                    if let (Some(start_pos), Some(end_pos)) =
                        (data.anchor(start), data.anchor(finish))
                    {
                        if start_pos != end_pos {
                            let auto_route = {
                                let mut router = data.scratch_router();
                                router.waypoint_path(
                                    snap_to_grid(start_pos),
                                    &waypoints,
                                    snap_to_grid(end_pos),
                                )
                            };
                            let route = AutoRoute::build(
                                start,
                                finish,
                                &auto_route,
                                waypoints,
                                Store::default(),
                            );
                            data.add_auto_route(route);
                            self.preview_path.clear();
                            RouteToolState::Idle
                        } else {
                            RouteToolState::Proposed {
                                start,
                                waypoints,
                                finish,
                            }
                        }
                    } else {
                        RouteToolState::Proposed {
                            start,
                            waypoints,
                            finish,
                        }
                    }
                }
                Some(Event::HoverAt(pos)) => {
                    if let Some(anchor) = data.anchor_at_pos(pos) {
                        if anchor != start {
                            finish = anchor;
                        }
                        RouteToolState::Proposed {
                            start,
                            waypoints,
                            finish,
                        }
                    } else {
                        RouteToolState::InProgress {
                            start,
                            waypoints,
                            head: pos,
                        }
                    }
                }
                _ => RouteToolState::Proposed {
                    start,
                    waypoints,
                    finish,
                },
            },
        };

        self.update_preview(data);
        self.render(data, painter);
        None
    }
}

impl RouteTool {
    fn update_preview(&mut self, data: &mut Data) {
        let mut router = data.scratch_router();
        let new_path = match &self.state {
            RouteToolState::InProgress {
                start,
                waypoints,
                head,
            } => data
                .anchor(*start)
                .map(|start_pos| router.waypoint_path(start_pos, waypoints, snap_to_grid(*head))),
            RouteToolState::Proposed {
                start,
                waypoints,
                finish,
            } => match (data.anchor(*start), data.anchor(*finish)) {
                (Some(start_pos), Some(end_pos)) => Some(router.waypoint_path(
                    snap_to_grid(start_pos),
                    waypoints,
                    snap_to_grid(end_pos),
                )),
                _ => None,
            },
            _ => None,
        };
        match new_path {
            Some(path) => self.preview_path = path,
            None => self.preview_path.clear(),
        }
    }

    fn render(&self, data: &Data, painter: &mut Painter) {
        let theme = painter.theme().clone();
        match &self.state {
            RouteToolState::PinHeadHovered { anchor } => {
                if let Some(pos) = data.anchor(*anchor) {
                    painter.circle(
                        pos,
                        PORT_RADIUS,
                        theme.route_proposed_endpoint,
                        (0.5, theme.route_proposed_endpoint),
                    );
                }
            }
            RouteToolState::InProgress { waypoints, .. } => {
                let pts: Vec<Pos2> = self.preview_path.iter().map(|p| p.pos.into()).collect();
                render_path_with_chamfered_corners(&pts)
                    .render(painter, (0.5, theme.route_in_progress));
                for (_, wp) in waypoints.iter() {
                    painter.circle_filled(wp.pos, PORT_RADIUS, theme.route_in_progress);
                }
            }
            RouteToolState::Proposed {
                start,
                finish,
                waypoints,
            } => {
                let pts: Vec<Pos2> = self.preview_path.iter().map(|p| p.pos.into()).collect();
                render_path_with_chamfered_corners(&pts)
                    .render(painter, (1.5, theme.route_in_progress));
                for anchor in [*start, *finish] {
                    if let Some(pos) = data.anchor(anchor) {
                        painter.circle(
                            pos,
                            PORT_RADIUS,
                            theme.route_proposed_endpoint,
                            (0.5, theme.route_proposed_endpoint),
                        );
                    }
                }
                for (_, wp) in waypoints.iter() {
                    painter.circle_filled(wp.pos, PORT_RADIUS, theme.route_in_progress);
                }
            }
            _ => {}
        }
    }
}
