use egui::{Pos2, Rect, vec2};

use crate::{
    grid::{GRID_SIZE, snap_to_grid},
    router::{RouterNG, RouterNGBuilder, WIRE_COST, cost::COST_ZERO},
    store::{RectId, RouteId, Store},
    widget::{
        auto_route::AutoRoute,
        block::Block,
        drawing::LineAnchor,
        pin::PinSide,
        port::Port,
        render::{estimate_bbox_for_pin_text, get_control_pin_bbox},
        shape::{BaseShape, PinLocation, Shape},
    },
};

#[derive(Default)]
pub struct Data {
    rect_boxes: Store<RectId, Shape>,
    auto_routes: Store<RouteId, AutoRoute>,
    router: Option<RouterNG>,
}

impl Data {
    pub fn take_routes(&mut self) -> Store<RouteId, AutoRoute> {
        std::mem::take(&mut self.auto_routes)
    }
    pub fn set_routes(&mut self, routes: Store<RouteId, AutoRoute>) {
        self.auto_routes = routes;
    }
    pub fn rect(&self, id: RectId) -> Option<&Shape> {
        self.rect_boxes.get(id)
    }
    pub fn rect_mut(&mut self, id: RectId) -> Option<&mut Shape> {
        self.rect_boxes.get_mut(id)
    }
    pub fn add_rect_box(&mut self, start: Pos2, end: Pos2) -> RectId {
        self.rect_boxes
            .insert(Block::new("Untitled".to_string(), Rect::from_two_pos(start, end)).into())
    }
    pub fn add_port_box(&mut self, pin_name: String, side: PinSide, inner: Rect) -> RectId {
        self.rect_boxes
            .insert(Port::new(pin_name, side, inner).into())
    }
    pub fn rect_boxes(&self) -> impl Iterator<Item = (RectId, &Shape)> {
        self.rect_boxes.iter()
    }
    pub fn rect_boxes_mut(&mut self) -> impl Iterator<Item = (RectId, &mut Shape)> {
        self.rect_boxes.iter_mut()
    }
    pub fn auto_routes(&self) -> impl Iterator<Item = (RouteId, &AutoRoute)> {
        self.auto_routes.iter()
    }
    pub fn auto_route(&self, id: RouteId) -> Option<&AutoRoute> {
        self.auto_routes.get(id)
    }
    pub fn auto_route_mut(&mut self, id: RouteId) -> Option<&mut AutoRoute> {
        self.auto_routes.get_mut(id)
    }
    pub fn add_auto_route(&mut self, route: AutoRoute) -> RouteId {
        self.auto_routes.insert(route)
    }
    pub fn anchor_at_pos(&self, pos: Pos2) -> Option<LineAnchor> {
        for (rect_id, shape) in self.rect_boxes.iter() {
            let rect = shape.gui_rect();
            if let Some(anchor) = shape.find_pin(|pin_id, pin| {
                let bbox = get_control_pin_bbox(rect, pin);
                if bbox.contains(pos) {
                    Some(LineAnchor {
                        rect: rect_id,
                        pin: pin_id,
                    })
                } else {
                    None
                }
            }) {
                return Some(anchor);
            }
        }
        None
    }
    pub fn pin_text_at_pos(&self, pos: Pos2) -> Option<(LineAnchor, PinLocation)> {
        for (id, rect_box) in self.rect_boxes() {
            if let Some(pin) = rect_box.find_pin(|pid, pin| {
                let text_box = estimate_bbox_for_pin_text(rect_box.gui_rect(), pin).expand(4.0);
                if text_box.contains(pos) {
                    Some((
                        LineAnchor { rect: id, pin: pid },
                        PinLocation {
                            side: pin.side,
                            offset: pin.offset,
                        },
                    ))
                } else {
                    None
                }
            }) {
                return Some(pin);
            }
        }
        None
    }
    pub fn anchor(&self, anchor: LineAnchor) -> Option<Pos2> {
        let shape = self.rect(anchor.rect)?;
        shape.anchor_point_with_rect(shape.gui_rect(), anchor.pin)
    }
    fn build_router(&self) -> RouterNG {
        let mut builder = RouterNGBuilder::default();
        for (_, rect_box) in self.rect_boxes() {
            let rect = rect_box.gui_rect();
            builder.add_block(rect.left_top(), rect.right_bottom());
            rect_box.with_pins(|pid, pin| {
                let Some(anchor) = rect_box.anchor_point_with_rect(rect, pid) else {
                    return;
                };
                let anchor_pos = match pin.side {
                    PinSide::East => anchor + vec2(GRID_SIZE, 0.0),
                    PinSide::West => anchor - vec2(GRID_SIZE, 0.0),
                };
                builder.add_h_channel(anchor_pos, COST_ZERO);
            });
        }
        builder.build()
    }
    pub fn update_routes(&mut self, ripup: &[RouteId]) {
        let mut router = self.build_router();
        let mut routes = self.take_routes();
        for (id, route) in routes.iter_mut() {
            let Some(anchor_start) = self.anchor(route.start()) else {
                continue;
            };
            let Some(anchor_end) = self.anchor(route.finish()) else {
                continue;
            };
            let anchor_start = snap_to_grid(anchor_start);
            let anchor_end = snap_to_grid(anchor_end);
            if Some(route.start_pos()) == self.anchor(route.start())
                && Some(route.end_pos()) == self.anchor(route.finish())
                && !router.is_route_blocked(route.iter_edges().map(|(_, edge)| edge))
                && !ripup.contains(&id)
            {
                router.add_existing_route(route.iter_edges().map(|(_, edge)| edge), WIRE_COST);
            } else {
                route.rip_and_reroute(anchor_start, anchor_end, &mut router);
            }
        }
        self.set_routes(routes);
        self.router = Some(router);
    }
    pub fn new_pin_location(&self, pos: Pos2) -> Option<(RectId, PinLocation)> {
        for (rect_id, shape) in self.rect_boxes.iter() {
            if let Some(location) = shape.new_pin_location(pos) {
                return Some((rect_id, location));
            }
        }
        None
    }
    pub fn add_new_pin(&mut self, rect: RectId, loc: PinLocation) {
        if let Some(shape) = self.rect_boxes.get_mut(rect) {
            shape.add_pin("Pin".into(), loc);
        }
    }
    pub fn scratch_router(&mut self) -> RouterNG {
        // TODO - Not sure if this is efficient or not.
        // if let Some(router) = &mut self.router {
        //     router.clone()
        // } else {
        self.update_routes(&[]);
        self.router.clone().unwrap()
        //        }
    }
}
