use nalgebra::{Point2, Vector2};
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};

/// Represents the state of a line
#[derive(Debug, Copy, Clone)]
pub struct LineState {
    pub x: f32,
    pub y: f32,
    #[allow(dead_code)]
    pub rotation: f32,
    pub opacity: f32,
    #[allow(dead_code)]
    pub speed: f32,
}

impl LineState {
    /// Returns if the line is visible in the viewport
    pub fn is_visible(&self) -> bool {
        if self.opacity <= 0.0 {
            // opacity <= 0.0 -> not visible
            false
        } else if self.x >= -CANVAS_WIDTH / 2.0
            && self.x <= CANVAS_WIDTH / 2.0
            && self.y >= -CANVAS_HEIGHT / 2.0
            && self.y <= CANVAS_HEIGHT / 2.0
        {
            // opacity > 0.0, origin inside viewport -> visible
            true
        } else {
            // opacity > 0.0, origin outside viewport, but still may visible because of rotation

            let half_length = 1.5 * CANVAS_WIDTH;

            let center = Point2::new(self.x, self.y);
            let direction = Vector2::new(
                self.rotation.to_radians().cos(),
                self.rotation.to_radians().sin(),
            ) * half_length;

            let p1 = center + direction;
            let p2 = center - direction;

            // viewport boundaries
            let viewport_min = Point2::new(-CANVAS_WIDTH / 2.0, -CANVAS_HEIGHT / 2.0);
            let viewport_max = Point2::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0);

            // check if either endpoint of the line is inside the viewport
            if is_point_in_rect(p1, viewport_min, viewport_max)
                || is_point_in_rect(p2, viewport_min, viewport_max)
            {
                return true;
            }

            let edges = [
                // left
                (
                    Point2::new(viewport_min.x, viewport_min.y),
                    Point2::new(viewport_min.x, viewport_max.y),
                ),
                // right
                (
                    Point2::new(viewport_max.x, viewport_min.y),
                    Point2::new(viewport_max.x, viewport_max.y),
                ),
                // top
                (
                    Point2::new(viewport_min.x, viewport_max.y),
                    Point2::new(viewport_max.x, viewport_max.y),
                ),
                // bottom
                (
                    Point2::new(viewport_min.x, viewport_min.y),
                    Point2::new(viewport_max.x, viewport_min.y),
                ),
            ];

            // check if the line intersects with any of the viewport edges
            for (edge_start, edge_end) in edges {
                if line_segments_intersect(p1, p2, edge_start, edge_end) {
                    return true;
                }
            }

            // both endpoints are outside the viewport and the line doesn't intersect with any viewport edge, it's not visible
            false
        }
    }
}

/// Check if a point is inside a rectangle
fn is_point_in_rect(point: Point2<f32>, rect_min: Point2<f32>, rect_max: Point2<f32>) -> bool {
    point.x >= rect_min.x && point.x <= rect_max.x && point.y >= rect_min.y && point.y <= rect_max.y
}

/// Check if two line segments intersect
fn line_segments_intersect(
    p1: Point2<f32>,
    p2: Point2<f32>, // endpoints of the first line segment
    p3: Point2<f32>,
    p4: Point2<f32>, // endpoints of the second line segment
) -> bool {
    // use cross products to detect intersection between two line segments
    let d1 = direction(p3, p4, p1);
    let d2 = direction(p3, p4, p2);
    let d3 = direction(p1, p2, p3);
    let d4 = direction(p1, p2, p4);

    // line segments intersect if d1 and d2 have opposite signs, and d3 and d4 have opposite signs
    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }

    // special case: collinear segments that overlap
    if d1 == 0.0 && on_segment(p3, p4, p1) {
        return true;
    }
    if d2 == 0.0 && on_segment(p3, p4, p2) {
        return true;
    }
    if d3 == 0.0 && on_segment(p1, p2, p3) {
        return true;
    }
    if d4 == 0.0 && on_segment(p1, p2, p4) {
        return true;
    }

    false
}

/// Calculate the direction (cross product) of point p3 relative to line segment p1-p2
fn direction(p1: Point2<f32>, p2: Point2<f32>, p3: Point2<f32>) -> f32 {
    (p3.x - p1.x) * (p2.y - p1.y) - (p3.y - p1.y) * (p2.x - p1.x)
}

/// Check if point p is on line segment p1-p2
fn on_segment(p1: Point2<f32>, p2: Point2<f32>, p: Point2<f32>) -> bool {
    p.x >= p1.x.min(p2.x) && p.x <= p1.x.max(p2.x) && p.y >= p1.y.min(p2.y) && p.y <= p1.y.max(p2.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_visible() {
        let state = LineState {
            x: -1958.3334,
            y: 979.1667,
            rotation: 106.40625,
            opacity: 255.0,
            speed: 10.0,
        };

        assert!(!state.is_visible());
    }
}
