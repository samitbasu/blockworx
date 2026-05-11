use egui::{
    Align2, Color32, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2,
    epaint::{PathShape, PathStroke, TextShape},
};

/// A transform-aware painter that accepts world-space coordinates and converts them
/// to screen space internally. All sizes — font sizes, stroke widths, radii, rounding —
/// scale with zoom so the diagram looks consistent at any zoom level. Unlike egui's
/// Scene (which applies a GPU-level pixel transform), we re-render at the correct
/// size each frame, so text and edges remain sharp.
pub struct Painter {
    inner: egui::Painter,
    origin: Pos2,
    zoom: f32,
    translation: Vec2,
}

impl Painter {
    pub(crate) fn new(inner: egui::Painter, origin: Pos2, zoom: f32, translation: Vec2) -> Self {
        Self {
            inner,
            origin,
            zoom,
            translation,
        }
    }

    /// World-space position → screen-space position.
    fn w2s(&self, world: Pos2) -> Pos2 {
        self.origin + self.translation + world.to_vec2() * self.zoom
    }

    /// Scale a stroke's width by the current zoom level.
    fn scale_stroke(&self, stroke: impl Into<Stroke>) -> Stroke {
        let s = stroke.into();
        Stroke::new(s.width * self.zoom, s.color)
    }

    // ── Rectangles ─────────────────────────────────────────────────────────

    /// Draw a filled and stroked rectangle. All arguments are in world space and scale with zoom.
    pub fn rect(
        &self,
        rect: Rect,
        rounding: f32,
        fill: impl Into<Color32>,
        stroke: impl Into<Stroke>,
    ) {
        let screen = Rect::from_min_max(self.w2s(rect.min), self.w2s(rect.max));
        let screen_rounding = CornerRadius::same((rounding * self.zoom).round().min(255.0) as u8);
        self.inner.rect(
            screen,
            screen_rounding,
            fill.into(),
            self.scale_stroke(stroke),
            StrokeKind::Middle,
        );
    }

    /// Draw a rectangle outline with no fill. All arguments are in world space and scale with zoom.
    pub fn rect_stroke(
        &self,
        rect: Rect,
        rounding: f32,
        stroke: impl Into<Stroke>,
        stroke_kind: StrokeKind,
    ) {
        let screen = Rect::from_min_max(self.w2s(rect.min), self.w2s(rect.max));
        let screen_rounding = CornerRadius::same((rounding * self.zoom).round().min(255.0) as u8);
        self.inner
            .rect_stroke(screen, screen_rounding, self.scale_stroke(stroke), stroke_kind);
    }

    // ── Text ───────────────────────────────────────────────────────────────

    /// Draw text at a world-space position. Font size and position scale with zoom so text
    /// grows and shrinks with the diagram. Because egui re-rasterizes at the correct size
    /// rather than pixel-scaling, the result stays crisp at any zoom level.
    /// Returns the screen-space bounding rect.
    pub fn text(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        font: FontId,
        color: impl Into<Color32>,
    ) -> Rect {
        let scaled = FontId::new(font.size * self.zoom, font.family);
        self.inner.text(self.w2s(pos), anchor, text, scaled, color.into())
    }

    /// Draw text rotated by `angle` radians around `pos` using the given `anchor`.
    /// Font size and position scale with zoom. Use this for vertical labels and similar.
    ///
    /// Note: if additional position adjustments are needed after layout (e.g. centering
    /// corrections), call `text_size` first to measure, then compute the adjusted position
    /// before calling this method.
    pub fn rotated_text(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        font: FontId,
        color: impl Into<Color32>,
        angle: f32,
    ) {
        let color = color.into();
        let scaled = FontId::new(font.size * self.zoom, font.family);
        let galley = self.inner.layout_no_wrap(text.to_string(), scaled, color);
        let shape = TextShape::new(self.w2s(pos), galley, color)
            .with_angle_and_anchor(angle, anchor);
        self.inner.add(shape);
    }

    /// Return the bounding size of `text` at `font` size in world units, without drawing.
    /// The font is scaled by zoom the same way `text()` does, then the screen-pixel size is
    /// divided back by zoom to give world-space dimensions. Use this for layout and hit-testing.
    pub fn text_size(&self, text: impl ToString, font: FontId) -> egui::Vec2 {
        let scaled = FontId::new(font.size * self.zoom, font.family);
        let galley = self.inner.layout_no_wrap(text.to_string(), scaled, Color32::WHITE);
        galley.size() / self.zoom
    }

    // ── Lines ──────────────────────────────────────────────────────────────

    /// Draw a line segment between two world-space points. Stroke width scales with zoom.
    pub fn line_segment(&self, points: [Pos2; 2], stroke: impl Into<Stroke>) {
        self.inner.line_segment(
            [self.w2s(points[0]), self.w2s(points[1])],
            self.scale_stroke(stroke),
        );
    }

    /// Draw a polyline through world-space points. Stroke width scales with zoom.
    pub fn line(&self, points: Vec<Pos2>, stroke: impl Into<Stroke>) {
        let screen_points: Vec<Pos2> = points.into_iter().map(|p| self.w2s(p)).collect();
        self.inner.line(screen_points, self.scale_stroke(stroke));
    }

    // ── Circles ────────────────────────────────────────────────────────────

    /// Draw a filled circle. Center and radius are in world space and scale with zoom.
    pub fn circle_filled(&self, center: Pos2, radius: f32, fill: impl Into<Color32>) {
        self.inner
            .circle_filled(self.w2s(center), radius * self.zoom, fill.into());
    }

    /// Draw a circle outline. Center and radius are in world space. Stroke width scales with zoom.
    pub fn circle_stroke(&self, center: Pos2, radius: f32, stroke: impl Into<Stroke>) {
        self.inner
            .circle_stroke(self.w2s(center), radius * self.zoom, self.scale_stroke(stroke));
    }

    /// Draw a filled and stroked circle. All arguments scale with zoom.
    pub fn circle(
        &self,
        center: Pos2,
        radius: f32,
        fill: impl Into<Color32>,
        stroke: impl Into<Stroke>,
    ) {
        self.inner.circle(
            self.w2s(center),
            radius * self.zoom,
            fill.into(),
            self.scale_stroke(stroke),
        );
    }

    // ── Arbitrary paths and polygons ───────────────────────────────────────

    /// Draw a closed or open path through world-space points with optional fill and stroke.
    /// All coordinates and stroke width scale with zoom.
    pub fn add_path(
        &self,
        points: Vec<Pos2>,
        closed: bool,
        fill: impl Into<Color32>,
        stroke: impl Into<Stroke>,
    ) {
        let screen_points: Vec<Pos2> = points.into_iter().map(|p| self.w2s(p)).collect();
        let s = self.scale_stroke(stroke);
        self.inner.add(egui::Shape::Path(PathShape {
            points: screen_points,
            closed,
            fill: fill.into(),
            stroke: PathStroke::new(s.width, s.color),
        }));
    }

    /// Draw a convex polygon through world-space points. All coordinates and stroke width
    /// scale with zoom.
    pub fn add_convex_polygon(
        &self,
        points: Vec<Pos2>,
        fill: impl Into<Color32>,
        stroke: impl Into<Stroke>,
    ) {
        let screen_points: Vec<Pos2> = points.into_iter().map(|p| self.w2s(p)).collect();
        let s = self.scale_stroke(stroke);
        self.inner
            .add(egui::Shape::convex_polygon(screen_points, fill.into(), s));
    }
}
