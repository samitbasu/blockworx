use egui::{Pos2, Response};

use crate::{
    grid::{PORT_RADIUS, snap_to_grid},
    router::TaggedPoint,
    store::{Store, WaypointId},
    theme::get_theme,
    widget::{
        auto_route::AutoRoute, data::Data, drawing::LineAnchor,
        render::render_path_with_chamfered_corners, waypoint::Waypoint,
    },
};

#[derive(PartialEq, Debug)]
pub struct InProgressAutoRoute {
    pub start: LineAnchor,
    pub waypoints: Store<WaypointId, Waypoint>,
    pub head: Pos2,
}

#[derive(PartialEq, Debug)]
pub struct ProposedAutoRoute {
    pub start: LineAnchor,
    pub waypoints: Store<WaypointId, Waypoint>,
    pub finish: LineAnchor,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PinHeadHovered {
    pub anchor: LineAnchor,
}

#[derive(Default)]
enum RouteState {
    #[default]
    Idle,
    PinHeadHovered(PinHeadHovered),
    InProgressAutoRoute(InProgressAutoRoute),
    ProposedAutoRoute(ProposedAutoRoute),
}

#[derive(Default)]
pub struct Route {
    state: RouteState,
    preview_path: Vec<TaggedPoint>,
}

impl Route {
    pub(crate) fn update(&mut self, data: &mut Data, response: &mut Response) -> Option<AutoRoute> {
        match &mut self.state {
            RouteState::Idle => {
                if (response.drag_started() || response.clicked())
                    && let Some(pos) = response.interact_pointer_pos()
                    && let Some(anchor) = data.anchor_at_pos(pos)
                {
                    response.mark_changed();
                    self.state = RouteState::InProgressAutoRoute(InProgressAutoRoute {
                        start: anchor,
                        waypoints: Store::default(),
                        head: pos,
                    });
                } else if let Some(pos) = response.hover_pos()
                    && let Some(anchor) = data.anchor_at_pos(pos)
                {
                    response.mark_changed();
                    self.state = RouteState::PinHeadHovered(PinHeadHovered { anchor });
                } else {
                }
            }
            RouteState::PinHeadHovered(inner) => {
                if (response.drag_started() || response.clicked())
                    && let Some(pos) = response.interact_pointer_pos()
                {
                    response.mark_changed();
                    self.state = RouteState::InProgressAutoRoute(InProgressAutoRoute {
                        start: inner.anchor,
                        waypoints: Store::default(),
                        head: pos,
                    });
                } else if let Some(pos) = response.hover_pos()
                    && let Some(anchor) = data.anchor_at_pos(pos)
                {
                    response.mark_changed();
                    self.state = RouteState::PinHeadHovered(PinHeadHovered { anchor });
                } else {
                    response.mark_changed();
                    self.state = RouteState::Idle;
                }
            }
            RouteState::InProgressAutoRoute(inner) => {
                if response.clicked()
                    && let Some(pos) = response.interact_pointer_pos()
                {
                    response.mark_changed();
                    inner.waypoints.insert(Waypoint {
                        pos: snap_to_grid(pos),
                        locked: true,
                    });
                } else if let Some(pos) = response.hover_pos() {
                    response.mark_changed();
                    if let Some(anchor) = data.anchor_at_pos(pos)
                        && anchor != inner.start
                    {
                        self.state = RouteState::ProposedAutoRoute(ProposedAutoRoute {
                            start: inner.start,
                            waypoints: std::mem::take(&mut inner.waypoints),
                            finish: anchor,
                        });
                    } else {
                        inner.head = pos;
                    }
                }
            }
            RouteState::ProposedAutoRoute(inner) => {
                if response.clicked()
                    && let Some(pos) = response.interact_pointer_pos()
                    && let Some(start_pos) = data.anchor(inner.start)
                    && let Some(end_pos) = data.anchor(inner.finish)
                    && start_pos != end_pos
                {
                    let mut scratch_router = data.scratch_router();
                    let auto_route = scratch_router.waypoint_path(
                        snap_to_grid(start_pos),
                        &inner.waypoints,
                        snap_to_grid(end_pos),
                    );
                    let route = AutoRoute::build(
                        inner.start,
                        inner.finish,
                        &auto_route,
                        std::mem::take(&mut inner.waypoints),
                        Store::default(),
                    );
                    response.mark_changed();
                    self.state = RouteState::Idle;
                    return Some(route);
                } else if let Some(pos) = response.hover_pos()
                    && let Some(anchor) = data.anchor_at_pos(pos)
                    && anchor != inner.start
                {
                    response.mark_changed();
                    inner.finish = anchor;
                } else if let Some(pos) = response.hover_pos() {
                    response.mark_changed();
                    self.state = RouteState::InProgressAutoRoute(InProgressAutoRoute {
                        start: inner.start,
                        waypoints: std::mem::take(&mut inner.waypoints),
                        head: pos,
                    });
                }
            }
        }
        self.update_preview(data);
        None
    }
    fn update_preview(&mut self, data: &mut Data) {
        let mut router = data.scratch_router();
        let new_path = match &self.state {
            RouteState::InProgressAutoRoute(inner) => data.anchor(inner.start).map(|start_pos| {
                router.waypoint_path(start_pos, &inner.waypoints, snap_to_grid(inner.head))
            }),
            RouteState::ProposedAutoRoute(inner) => {
                if let (Some(start_pos), Some(end_pos)) =
                    (data.anchor(inner.start), data.anchor(inner.finish))
                {
                    Some(router.waypoint_path(
                        snap_to_grid(start_pos),
                        &inner.waypoints,
                        snap_to_grid(end_pos),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        };
        match new_path {
            Some(path) => self.preview_path = path,
            None => self.preview_path.clear(),
        }
    }

    pub fn render(&self, data: &Data, ui: &mut egui::Ui) {
        let theme = get_theme(ui);
        match &self.state {
            RouteState::PinHeadHovered(inner) => {
                if let Some(pos) = data.anchor(inner.anchor) {
                    ui.painter().circle(
                        pos,
                        PORT_RADIUS,
                        theme.route_proposed_endpoint,
                        (0.5, theme.route_proposed_endpoint),
                    );
                }
            }
            RouteState::InProgressAutoRoute(inner) => {
                let pts: Vec<Pos2> = self.preview_path.iter().map(|p| p.pos.into()).collect();
                render_path_with_chamfered_corners(&pts).render(ui, (0.5, theme.route_in_progress));
                for (_, wp) in inner.waypoints.iter() {
                    ui.painter()
                        .circle_filled(wp.pos, PORT_RADIUS, theme.route_in_progress);
                }
            }
            RouteState::ProposedAutoRoute(inner) => {
                let pts: Vec<Pos2> = self.preview_path.iter().map(|p| p.pos.into()).collect();
                render_path_with_chamfered_corners(&pts).render(ui, (1.5, theme.route_in_progress));
                for anchor in [inner.start, inner.finish] {
                    if let Some(pos) = data.anchor(anchor) {
                        ui.painter().circle(
                            pos,
                            PORT_RADIUS,
                            theme.route_proposed_endpoint,
                            (0.5, theme.route_proposed_endpoint),
                        );
                    }
                }
            }
            _ => {}
        }
    }
}
