use egui::Color32;

#[derive(Clone, Copy)]
pub struct Theme {
    // Shapes (blocks and ports)
    pub shape_fill: Color32,
    pub shape_stroke: Color32,
    pub shape_title: Color32,

    // Routes / wires
    pub route_normal: Color32,
    pub route_selected: Color32,
    pub route_highlighted: Color32,
    pub route_edge_highlight: Color32,
    pub route_in_progress: Color32,
    pub route_proposed_endpoint: Color32,

    // Pins
    pub pin_stem: Color32,
    pub pin_text: Color32,

    // Drag / move states
    pub drag_preview_stroke: Color32,
    pub drag_active_fill: Color32,
    pub drag_active_stroke: Color32,
    pub hover_fill: Color32,
    pub corner_highlight_fill: Color32,
    pub edge_drag_preview: Color32,

    // Selection & editing controls
    pub selection_frame: Color32,
    pub control_handle_fill: Color32,
    pub control_handle_stroke: Color32,
    pub waypoint_fill: Color32,
    pub add_button_fill: Color32,

    // UI chrome
    pub grid_line: Color32,
    pub pin_drag_indicator: Color32,
    pub hamburger_menu: Color32,
    pub text_edit_background: Color32,
    pub text_edit_text: Color32,

    // Debug
    pub debug_graph: Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            shape_fill: Color32::LIGHT_GRAY,
            shape_stroke: Color32::BLUE,
            shape_title: Color32::DARK_GREEN,
            route_normal: Color32::DARK_GREEN,
            route_selected: Color32::LIGHT_GREEN,
            route_highlighted: Color32::LIGHT_GREEN.gamma_multiply(0.3),
            route_edge_highlight: Color32::LIGHT_GREEN.gamma_multiply(0.7),
            route_in_progress: Color32::LIGHT_YELLOW,
            route_proposed_endpoint: Color32::DARK_RED,
            pin_stem: Color32::DARK_RED,
            pin_text: Color32::BLACK,
            drag_preview_stroke: Color32::DARK_GRAY,
            drag_active_fill: Color32::LIGHT_GRAY,
            drag_active_stroke: Color32::DARK_RED,
            hover_fill: Color32::GRAY,
            corner_highlight_fill: Color32::LIGHT_RED.linear_multiply(0.5),
            edge_drag_preview: Color32::GRAY.gamma_multiply(0.2),
            selection_frame: Color32::DARK_RED,
            control_handle_fill: Color32::WHITE,
            control_handle_stroke: Color32::BLACK,
            waypoint_fill: Color32::LIGHT_GREEN.linear_multiply(0.5),
            add_button_fill: Color32::LIGHT_YELLOW.linear_multiply(0.5),
            grid_line: Color32::LIGHT_GRAY.linear_multiply(0.3),
            pin_drag_indicator: Color32::DARK_GRAY.gamma_multiply(0.3),
            hamburger_menu: Color32::DARK_GRAY.gamma_multiply(0.3),
            text_edit_background: Color32::WHITE,
            text_edit_text: Color32::BLACK,
            debug_graph: Color32::RED.gamma_multiply(0.2),
        }
    }
}

pub fn get_theme(ui: &egui::Ui) -> Theme {
    ui.data(|d| d.get_temp::<Theme>(egui::Id::NULL))
        .unwrap_or_default()
}
