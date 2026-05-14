use egui::{Pos2, Response, Vec2};

/// Per-frame input state passed to the canvas drawing closure.
///
/// `event` holds the mouse/pointer interaction (if any) with all positions
/// already converted to world space. The boolean flags fire independently —
/// a key press and a mouse event can both be set in the same frame.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Interaction {
    pub event: Option<Event>,
    /// The canvas widget lost keyboard focus this frame.
    pub lost_focus: bool,
    /// Enter was pressed while the canvas held keyboard focus.
    pub enter_pressed: bool,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Event {
    HoverAt(Pos2),
    DragStarted { pos: Pos2 },
    Dragging { pos: Pos2, delta: Vec2 },
    DragStopped { pos: Pos2 },
    Clicked { pos: Pos2 },
    DoubleClicked { pos: Pos2 },
}

/// Build the full `Interaction` for one frame.
/// Positions in `event` are world-space; flags are read directly from the response and
/// global input. Both can be set simultaneously — they are independent sources.
pub(crate) fn compute_interaction(
    response: &Response,
    screen_to_world: impl Fn(Pos2) -> Pos2,
    zoom: f32,
    ui: &egui::Ui,
) -> Interaction {
    let event = compute_event(response, screen_to_world, zoom);
    let lost_focus = response.lost_focus();
    // Scope enter to when the canvas has keyboard focus so it doesn't fire
    // while a text field elsewhere is active.
    let enter_pressed = response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    Interaction {
        event,
        lost_focus,
        enter_pressed,
    }
}

pub(crate) fn compute_event(
    response: &Response,
    screen_to_world: impl Fn(Pos2) -> Pos2,
    zoom: f32,
) -> Option<Event> {
    if response.double_clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        Some(Event::DoubleClicked {
            pos: screen_to_world(pos),
        })
    } else if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        Some(Event::Clicked {
            pos: screen_to_world(pos),
        })
    } else if response.drag_started()
        && let Some(pos) = response.interact_pointer_pos()
    {
        Some(Event::DragStarted {
            pos: screen_to_world(pos),
        })
    } else if response.dragged()
        && let Some(pos) = response.interact_pointer_pos()
    {
        Some(Event::Dragging {
            pos: screen_to_world(pos),
            delta: response.drag_delta() / zoom,
        })
    } else if response.drag_stopped()
        && let Some(pos) = response.interact_pointer_pos()
    {
        Some(Event::DragStopped {
            pos: screen_to_world(pos),
        })
    } else if response.hovered()
        && let Some(pos) = response.hover_pos()
    {
        Some(Event::HoverAt(screen_to_world(pos)))
    } else {
        None
    }
}
