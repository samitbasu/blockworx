use egui::{CursorIcon, Pos2, Vec2};

use crate::{
    store::*,
    widget::{
        auto_route::AddTextButton, direction::RouteDirection, drawing::LineAnchor,
        waypoint::Waypoint,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResizeMode {
    LeftTop,
    RightTop,
    LeftBottom,
    RightBottom,
    CenterTop,
    CenterBottom,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub enum State {
    #[default]
    Idle,
    Panning,
    AddText,
    AddTextHoveredRoute(AddTextHoveredRoute),
    AddingRect(AddingRect),
    MovingRect(MovingRect),
    Selected(Selected),
    PotentialResize(PotentialResize),
    ResizingRect(ResizingRect),
    EditingName(EditingName),
    PortDragged(PortDragged),
    PortLabelHovered(PortLabelHovered),
    PortLabelGripHovered(PortLabelGripHovered),
    EditingLabelText(EditingLabelText),
    PortPinHovered(PortPinHovered),
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

impl From<RouteHovered> for State {
    fn from(value: RouteHovered) -> Self {
        State::RouteHovered(value)
    }
}

impl From<RouteSelected> for State {
    fn from(value: RouteSelected) -> Self {
        State::RouteSelected(value)
    }
}
impl From<RouteEdgeHovered> for State {
    fn from(value: RouteEdgeHovered) -> Self {
        State::RouteEdgeHovered(value)
    }
}
impl From<RouteCornerHovered> for State {
    fn from(value: RouteCornerHovered) -> Self {
        State::RouteCornerHovered(value)
    }
}
impl From<InProgressAutoRoute> for State {
    fn from(value: InProgressAutoRoute) -> Self {
        State::InProgressAutoRoute(value)
    }
}

impl From<ProposedAutoRoute> for State {
    fn from(value: ProposedAutoRoute) -> Self {
        State::ProposedAutoRoute(value)
    }
}

impl From<WaypointHovered> for State {
    fn from(value: WaypointHovered) -> Self {
        State::WaypointHovered(value)
    }
}

impl From<RouteEdgeDragged> for State {
    fn from(value: RouteEdgeDragged) -> Self {
        State::RouteEdgeDragged(value)
    }
}

impl From<RouteLabelHovered> for State {
    fn from(value: RouteLabelHovered) -> Self {
        State::RouteLabelHovered(value)
    }
}

impl From<EditingRouteLabelText> for State {
    fn from(value: EditingRouteLabelText) -> Self {
        State::EditingRouteLabelText(value)
    }
}

impl From<WaypointDragged> for State {
    fn from(value: WaypointDragged) -> Self {
        State::WaypointDragged(value)
    }
}

impl From<AddTextButtonHovered> for State {
    fn from(value: AddTextButtonHovered) -> Self {
        State::AddTextButtonHovered(value)
    }
}

impl From<TextAnchorHovered> for State {
    fn from(value: TextAnchorHovered) -> Self {
        State::TextAnchorHovered(value)
    }
}

impl From<AddTextHoveredRoute> for State {
    fn from(value: AddTextHoveredRoute) -> Self {
        State::AddTextHoveredRoute(value)
    }
}

impl From<TextAnchorDragged> for State {
    fn from(value: TextAnchorDragged) -> Self {
        State::TextAnchorDragged(value)
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
pub struct PortDragged {
    pub rect: RectId,
    pub label: LabelId,
    pub delta_pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PortLabelHovered {
    pub rect: RectId,
    pub label: LabelId,
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
pub struct PortLabelGripHovered {
    pub rect: RectId,
    pub label: LabelId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EditingLabelText {
    pub rect: RectId,
    pub label: LabelId,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PortPinHovered {
    pub rect: RectId,
    pub label: LabelId,
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

#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug, PartialOrd, Ord)]
pub struct WaypointId(usize);

impl From<usize> for WaypointId {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

impl std::fmt::Display for WaypointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "w{}", self.0)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct InProgressAutoRoute {
    pub start: LineAnchor,
    pub waypoints: Vec<Waypoint>,
    pub head: Pos2,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ProposedAutoRoute {
    pub start: LineAnchor,
    pub waypoints: Vec<Waypoint>,
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
    pub fn cursor(&self) -> CursorIcon {
        match self {
            State::PotentialResize(PotentialResize { mode, .. })
            | State::ResizingRect(ResizingRect { mode, .. }) => match mode {
                ResizeMode::LeftTop | ResizeMode::RightBottom => CursorIcon::ResizeNwSe,
                ResizeMode::RightTop | ResizeMode::LeftBottom => CursorIcon::ResizeNeSw,
                ResizeMode::CenterTop | ResizeMode::CenterBottom => CursorIcon::ResizeVertical,
            },
            State::PortLabelHovered { .. }
            | State::RouteLabelHovered { .. }
            | State::AddText
            | State::AddTextHoveredRoute { .. } => CursorIcon::Text,
            State::PortLabelGripHovered { .. } => CursorIcon::Grab,
            State::PortPinHovered { .. } => CursorIcon::Crosshair,
            State::PortDragged { .. } => CursorIcon::Grabbing,
            State::RouteEdgeHovered(inner) => match inner.direction {
                RouteDirection::Horizontal => CursorIcon::ResizeVertical,
                RouteDirection::Vertical => CursorIcon::ResizeHorizontal,
            },
            _ => CursorIcon::Default,
        }
    }
}
