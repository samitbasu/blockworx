use egui::{CursorIcon, Pos2, Vec2};

use crate::{
    store::*,
    widget::{
        auto_route::AddTextButton, direction::RouteDirection, drawing::LineAnchor,
        waypoint::Waypoint,
    },
};

#[derive(Clone, Copy, Debug)]
pub enum RenderMode {
    Normal,
    Moving         { delta: Vec2 },
    Resizing       { mode: ResizeMode, delta: Vec2 },
    Selected,
    PinHeadHovered { pin: PinId },
    PinDragged     { pin: PinId, delta: Vec2 },
    EditingName,
    EditingPinText { pin: PinId },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResizeMode {
    LeftTop,
    RightTop,
    LeftBottom,
    RightBottom,
    CenterTop,
    CenterBottom,
}

#[derive(PartialEq, Default, Debug, derive_more::From)]
pub enum State {
    #[default]
    #[from(skip)]
    Idle,
    #[from(skip)]
    Panning,
    #[from(skip)]
    AddText,
    AddTextHoveredRoute(AddTextHoveredRoute),
    AddingRect(AddingRect),
    MovingRect(MovingRect),
    Selected(Selected),
    PotentialResize(PotentialResize),
    ResizingRect(ResizingRect),
    EditingName(EditingName),
    PinDragged(PinDragged),
    PinLabelHovered(PinLabelHovered),
    PinLabelGripHovered(PinLabelGripHovered),
    EditingPinText(EditingPinText),
    PinHeadHovered(PinHeadHovered),
    InProgressAutoRoute(InProgressAutoRoute),
    ProposedAutoRoute(ProposedAutoRoute),
    RouteHovered(RouteHovered),
    RouteSelected(RouteSelected),
    RouteEdgeHovered(RouteEdgeHovered),
    RouteEdgeDragged(RouteEdgeDragged),
    RouteCornerHovered(RouteCornerHovered),
    WaypointHovered(WaypointHovered),
    WaypointDragged(WaypointDragged),
    RouteLabelHovered(RouteLabelHovered),
    EditingRouteLabelText(EditingRouteLabelText),
    AddTextButtonHovered(AddTextButtonHovered),
    TextAnchorHovered(TextAnchorHovered),
    TextAnchorDragged(TextAnchorDragged),
}

impl State {
    pub fn idle() -> Self {
        State::Idle
    }
    pub fn panning() -> Self {
        State::Panning
    }
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct AddingRect {
    pub start_pos: Pos2,
    pub end_pos: Pos2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MovingRect {
    pub rect: RectId,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Selected {
    pub rect: RectId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PotentialResize {
    pub rect: RectId,
    pub mode: ResizeMode,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EditingName {
    pub rect: RectId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RouteEdgeDragged {
    pub id: RouteId,
    pub direction: RouteDirection,
    pub start_waypoint: WaypointId,
    pub end_waypoint: WaypointId,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PinDragged {
    pub rect: RectId,
    pub pin: PinId,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PinLabelHovered {
    pub rect: RectId,
    pub pin: PinId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RouteControlPointHovered {
    pub id: RouteId,
    pub edge: EdgeId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RouteControlPointDragged {
    pub id: RouteId,
    pub edge: EdgeId,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RouteLabelHovered {
    pub id: RouteId,
    pub edge_index: EdgeId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EditingRouteLabelText {
    pub id: RouteId,
    pub label_id: WireLabelId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PinLabelGripHovered {
    pub rect: RectId,
    pub pin: PinId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EditingPinText {
    pub rect: RectId,
    pub pin: PinId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PinHeadHovered {
    pub rect: RectId,
    pub pin: PinId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResizingRect {
    pub rect: RectId,
    pub mode: ResizeMode,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AddTextButtonHovered {
    pub route: RouteId,
    pub button: AddTextButton,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AddTextHoveredRoute {
    pub route: RouteId,
    pub edge_id: EdgeId,
    pub pos: Pos2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TextAnchorHovered {
    pub route: RouteId,
    pub label_id: WireLabelId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TextAnchorDragged {
    pub route: RouteId,
    pub label_id: WireLabelId,
    pub delta_pos: Vec2,
}

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

#[derive(Clone, PartialEq, Debug)]
pub struct WaypointHovered {
    pub route: RouteId,
    pub waypoint: WaypointId,
}

#[derive(Clone, PartialEq, Debug)]
pub struct WaypointDragged {
    pub route: RouteId,
    pub waypoint: WaypointId,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RouteHovered {
    pub id: RouteId,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RouteSelected {
    pub id: RouteId,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RouteEdgeHovered {
    pub id: RouteId,
    pub edge_index: EdgeId,
    pub direction: RouteDirection,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RouteCornerHovered {
    pub id: RouteId,
    pub edge_1: EdgeId,
    pub edge_2: EdgeId,
}

impl State {
    pub fn render_mode_for_id(&self, id: RectId) -> RenderMode {
        match self {
            State::MovingRect(MovingRect { rect, delta_pos }) if id == *rect
                => RenderMode::Moving { delta: *delta_pos },
            State::Selected(Selected { rect })
            | State::PotentialResize(PotentialResize { rect, .. })
            | State::PinLabelHovered(PinLabelHovered { rect, .. })
            | State::PinLabelGripHovered(PinLabelGripHovered { rect, .. })
                if id == *rect => RenderMode::Selected,
            State::PinHeadHovered(PinHeadHovered { rect, pin }) if id == *rect
                => RenderMode::PinHeadHovered { pin: *pin },
            State::ResizingRect(ResizingRect { rect, mode, delta_pos, .. }) if id == *rect
                => RenderMode::Resizing { mode: *mode, delta: *delta_pos },
            State::PinDragged(PinDragged { rect, pin, delta_pos }) if id == *rect
                => RenderMode::PinDragged { pin: *pin, delta: *delta_pos },
            State::EditingName(EditingName { rect }) if id == *rect
                => RenderMode::EditingName,
            State::EditingPinText(EditingPinText { rect, pin }) if id == *rect
                => RenderMode::EditingPinText { pin: *pin },
            _ => RenderMode::Normal,
        }
    }

    pub fn cursor(&self) -> CursorIcon {
        match self {
            State::PotentialResize(PotentialResize { mode, .. })
            | State::ResizingRect(ResizingRect { mode, .. }) => match mode {
                ResizeMode::LeftTop | ResizeMode::RightBottom => CursorIcon::ResizeNwSe,
                ResizeMode::RightTop | ResizeMode::LeftBottom => CursorIcon::ResizeNeSw,
                ResizeMode::CenterTop | ResizeMode::CenterBottom => CursorIcon::ResizeVertical,
            },
            State::PinLabelHovered { .. }
            | State::RouteLabelHovered { .. }
            | State::AddText
            | State::AddTextHoveredRoute { .. } => CursorIcon::Text,
            State::PinLabelGripHovered { .. } => CursorIcon::Grab,
            State::PinHeadHovered { .. } => CursorIcon::Crosshair,
            State::PinDragged { .. } => CursorIcon::Grabbing,
            State::RouteEdgeHovered(inner) => match inner.direction {
                RouteDirection::Horizontal => CursorIcon::ResizeVertical,
                RouteDirection::Vertical => CursorIcon::ResizeHorizontal,
            },
            _ => CursorIcon::Default,
        }
    }
}
