use egui::Pos2;

use crate::widget::linear_distance::LinearDistance;

#[derive(Clone, PartialEq, Debug)]
pub struct WireLabel {
    pub linear_distance: LinearDistance,
    pub position: Pos2,
    pub text: String,
}
