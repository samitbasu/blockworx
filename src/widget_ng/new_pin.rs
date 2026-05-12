use crate::{
    canvas::Event,
    grid::PORT_RADIUS,
    widget::shape::BaseShape,
    widget_ng::{names::ToolName, tool::ToolTrait},
};

pub struct NewPin;

impl ToolTrait for NewPin {
    fn name(&self) -> ToolName {
        ToolName::NewPin
    }

    fn widget(
        &mut self,
        data: &mut crate::widget::data::Data,
        interaction: &crate::canvas::Interaction,
        painter: &mut crate::canvas::painter::Painter,
    ) {
        super::display::widget(data, interaction, painter);
        for (_id, rect_box) in data.rect_boxes_mut() {
            for pin_center in rect_box.new_pin_locations() {
                painter.circle_filled(pin_center.pos, PORT_RADIUS, egui::Color32::LIGHT_BLUE);
                if let Some(Event::Clicked { pos }) = interaction.event
                    && pos.distance(pin_center.pos) < PORT_RADIUS
                {
                    let _ =
                        rect_box.add_pin("Port".to_string(), pin_center.side, pin_center.offset);
                } else if let Some(Event::HoverAt(hover_pos)) = interaction.event {
                    if hover_pos.distance(pin_center.pos) < PORT_RADIUS {
                        painter.circle(
                            pin_center.pos,
                            PORT_RADIUS,
                            egui::Color32::LIGHT_BLUE,
                            (2.0, egui::Color32::WHITE),
                        )
                    }
                }
            }
        }
    }
}
