use crate::router::{
    coord::{CoordX, CoordY},
    interval_overlap,
    point::{Point, point},
};

// A blocked rectangle - inclusive of the edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl Block {
    pub fn expand_x(&self, delta_x: impl Into<CoordX>) -> Self {
        let delta_x: CoordX = delta_x.into();
        Block {
            top_left: point(self.top_left.x - delta_x, self.top_left.y),
            bottom_right: point(self.bottom_right.x + delta_x, self.bottom_right.y),
        }
    }
    pub fn spans_y(&self, y: CoordY) -> bool {
        self.top_left.y <= y && self.bottom_right.y >= y
    }
    pub fn spans_x(&self, x: CoordX) -> bool {
        self.top_left.x <= x && self.bottom_right.x >= x
    }
    pub fn is_left_of(&self, x: CoordX) -> bool {
        self.bottom_right.x < x
    }
    pub fn is_right_of(&self, x: CoordX) -> bool {
        self.top_left.x > x
    }
    pub fn is_above(&self, y: CoordY) -> bool {
        self.bottom_right.y < y
    }
    pub fn is_below(&self, y: CoordY) -> bool {
        self.top_left.y > y
    }
    pub fn contains(&self, point: Point) -> bool {
        self.spans_x(point.x) && self.spans_y(point.y)
    }
    pub fn intersects_edge(
        &self,
        start_point: impl Into<Point>,
        end_point: impl Into<Point>,
    ) -> bool {
        // Check for intersection between the edge and the block.  The edge is
        // either horizontal or vertical, so we can check for intersection by comparing the coordinates.
        let start_point: Point = start_point.into();
        let end_point: Point = end_point.into();
        if start_point.y == end_point.y {
            let min_x = start_point.x.min(end_point.x);
            let max_x = start_point.x.max(end_point.x);
            // The edge goes from [min_x,max_x], and we have the interval
            // [self.top_left.x, self.bottom_right.x] - the edge intersects the block if the intervals overlap.
            self.spans_y(start_point.y)
                && interval_overlap(min_x, max_x, self.top_left.x, self.bottom_right.x)
        } else {
            let min_y = start_point.y.min(end_point.y);
            let max_y = start_point.y.max(end_point.y);
            self.spans_x(start_point.x)
                && interval_overlap(min_y, max_y, self.top_left.y, self.bottom_right.y)
        }
    }
}
