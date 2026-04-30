use egui::{Pos2, Rect, Vec2, pos2, vec2};

use crate::{
    grid::{GRID_SIZE, grid_rect, round_to_grid, snap},
    state::ResizeMode,
    store::*,
    widget::label::{Label, LabelSide},
};

pub struct RectBox {
    name: String,
    inner: Rect,
    labels: Store<LabelId, Label>,
}

pub fn resize_rect(rect: &Rect, mode: ResizeMode, delta: Vec2) -> Rect {
    match mode {
        ResizeMode::LeftTop => Rect::from_two_pos(rect.left_top() + delta, rect.right_bottom()),
        ResizeMode::RightTop => Rect::from_two_pos(rect.right_top() + delta, rect.left_bottom()),
        ResizeMode::LeftBottom => Rect::from_two_pos(rect.left_bottom() + delta, rect.right_top()),
        ResizeMode::RightBottom => Rect::from_two_pos(rect.right_bottom() + delta, rect.left_top()),
        ResizeMode::CenterTop => {
            Rect::from_two_pos(rect.left_top() + vec2(0.0, delta.y), rect.right_bottom())
        }
        ResizeMode::CenterBottom => {
            Rect::from_two_pos(rect.left_bottom() + vec2(0.0, delta.y), rect.right_top())
        }
    }
}

pub fn control_corner(rect: &Rect, mode: ResizeMode) -> Pos2 {
    match mode {
        ResizeMode::LeftTop => rect.left_top(),
        ResizeMode::RightTop => rect.right_top(),
        ResizeMode::LeftBottom => rect.left_bottom(),
        ResizeMode::RightBottom => rect.right_bottom(),
        ResizeMode::CenterTop => rect.center_top(),
        ResizeMode::CenterBottom => rect.center_bottom(),
    }
}

impl RectBox {
    pub fn label(&self, id: LabelId) -> Option<&Label> {
        self.labels.get(id)
    }
    pub fn label_mut(&mut self, id: LabelId) -> Option<&mut Label> {
        self.labels.get_mut(id)
    }
    pub fn iter_labels(&self) -> impl Iterator<Item = (LabelId, &Label)> + '_ {
        self.labels.iter()
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }
    pub fn new(name: String, inner: Rect) -> Self {
        Self {
            name,
            inner: snap(inner),
            labels: Store::default(),
        }
    }
    pub fn is_port_offset_available(&self, side: LabelSide, offset: f32) -> bool {
        if offset < 0.0 || offset > self.inner.height() {
            return false;
        }
        self.labels
            .values()
            .filter(|l| l.side == side)
            .all(|l| (l.offset - offset).abs() >= GRID_SIZE * 0.2)
    }
    pub fn update_label_offset(&mut self, label_id: LabelId, delta_y: f32) {
        let Some(label_ref) = self.label(label_id) else {
            return;
        };
        let label_offset = round_to_grid(label_ref.offset + delta_y);
        if !self.is_port_offset_available(label_ref.side, label_offset) {
            return;
        }
        let Some(label_ref) = self.label_mut(label_id) else {
            return;
        };
        label_ref.offset = label_offset;
    }
    pub fn next_port_offset(&self, side: LabelSide) -> Option<f32> {
        let max_pos = (self.inner.height() / GRID_SIZE) as i32 - 1;
        if max_pos <= 0 {
            return None;
        }
        (0_u32..max_pos as u32).find_map(|ndx| {
            let offset = ndx as f32 * GRID_SIZE;
            if self
                .labels
                .values()
                .any(|l| l.side == side && (l.offset - offset).abs() < GRID_SIZE * 0.6)
            {
                None
            } else {
                Some(offset)
            }
        })
    }
    pub fn control_pin_location_east(&self) -> Option<Pos2> {
        // Find the first free offset
        // We want to check 0, -1, 1, -2, 2,..
        let offset = self.next_port_offset(LabelSide::East)?;
        Some(self.inner.right_top() + vec2(GRID_SIZE, GRID_SIZE + offset))
    }
    pub fn control_pin_location_west(&self) -> Option<Pos2> {
        let offset = self.next_port_offset(LabelSide::West)?;
        Some(self.inner.left_top() + vec2(-GRID_SIZE, GRID_SIZE + offset))
    }
    pub fn control_pin_for_label(&self, label_id: LabelId) -> Option<Pos2> {
        self.labels.get(label_id).map(|label| match label.side {
            LabelSide::East => self.inner.right_top() + vec2(GRID_SIZE, GRID_SIZE + label.offset),
            LabelSide::West => self.inner.left_top() + vec2(-GRID_SIZE, GRID_SIZE + label.offset),
        })
    }
    pub fn anchor_point_with_rect(&self, rect: Rect, id: LabelId) -> Option<Pos2> {
        self.labels.get(id).map(|label| match label.side {
            LabelSide::East => pos2(
                rect.right() + GRID_SIZE,
                rect.top() + GRID_SIZE + label.offset,
            ),
            LabelSide::West => pos2(
                rect.left() - GRID_SIZE,
                rect.top() + GRID_SIZE + label.offset,
            ),
        })
    }
    pub fn anchor_point(&self, id: LabelId) -> Option<Pos2> {
        self.anchor_point_with_rect(self.inner, id)
    }
    pub fn add_label(&mut self, text: String, side: LabelSide, offset: f32) -> LabelId {
        self.labels.insert(Label { text, side, offset })
    }
    pub fn predicted_rect(&self) -> Rect {
        grid_rect(self.inner)
    }
    pub fn gui_rect(&self) -> Rect {
        self.inner
    }
    pub fn gui_rect_mut(&mut self) -> &mut Rect {
        &mut self.inner
    }
}
