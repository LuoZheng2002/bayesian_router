use cgmath::{Rotation, Rotation2};

use crate::{
    prim_shape::{PrimShape, RectangleShape},
    vec2::FloatVec2,
};

#[derive(Debug, Clone)]
pub struct CircleCollider {
    pub position: FloatVec2,
    pub diameter: f32,
}

/// polygon is used only for collision detection, not for rendering
/// a line is a special polygon
#[derive(Debug, Clone)]
pub struct PolygonCollider(pub Vec<FloatVec2>);

#[derive(Debug, Clone)]
pub struct BorderCollider {
    pub point_on_border: FloatVec2,
    pub normal: FloatVec2,
}

#[derive(Debug, Clone)]
pub enum Collider {
    Circle(CircleCollider),
    Polygon(PolygonCollider),
    Border(BorderCollider),
}

// polygon with polygon,
// circle with polygon,
// circle with circle,
// circle with border,
// polygon with border,
// border with border (impossible)
impl Collider {
    fn rectangle_to_polygon(rectangle: &RectangleShape) -> PolygonCollider {
        let hw = rectangle.width / 2.0;
        let hh = rectangle.height / 2.0;

        // Corner positions before rotation (relative to center)
        let corners = [
            cgmath::Vector2::new(-hw, -hh),
            cgmath::Vector2::new(hw, -hh),
            cgmath::Vector2::new(hw, hh),
            cgmath::Vector2::new(-hw, hh),
        ];
        // Convert rotation to radians
        let rotation_rad: cgmath::Rad<f32> = rectangle.rotation.into();

        // Create rotation matrix
        let rotation = cgmath::Basis2::from_angle(rotation_rad);

        // Apply rotation and translate to position
        let rotated_corners: Vec<FloatVec2> = corners
            .iter()
            .map(|corner| {
                let rotated_corner = rotation.rotate_vector(*corner);
                // self.position + rotation.rotate_vector(*corner)
                FloatVec2 {
                    x: rectangle.position.x + rotated_corner.x,
                    y: rectangle.position.y + rotated_corner.y,
                }
            })
            .collect();

        PolygonCollider(rotated_corners)
    }
    pub fn from_prim_shape(shape: &PrimShape) -> Self {
        match shape {
            PrimShape::Circle(circle) => Collider::Circle(CircleCollider {
                position: circle.position,
                diameter: circle.diameter,
            }),
            PrimShape::Rectangle(rectangle) => {
                Collider::Polygon(Collider::rectangle_to_polygon(&rectangle))
            }
            PrimShape::Line(line) => Collider::Polygon(PolygonCollider(vec![line.start, line.end])),
        }
    }
    fn circle_circle(circle1: &CircleCollider, circle2: &CircleCollider) -> bool {
        let radius1 = circle1.diameter / 2.0;
        let radius2 = circle2.diameter / 2.0;
        let distance_squared = (circle1.position.x - circle2.position.x).powi(2)
            + (circle1.position.y - circle2.position.y).powi(2);
        distance_squared < (radius1 + radius2).powi(2)
    }
    /// Projects a polygon onto an axis and returns the min and max projection scalars
    fn project_polygon(polygon: &PolygonCollider, axis: FloatVec2) -> (f32, f32) {
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        let verts = &polygon.0;
        for &point in verts {
            let projection = point.dot(axis);
            if projection < min {
                min = projection;
            }
            if projection > max {
                max = projection;
            }
        }
        (min, max)
    }
    /// Project a circle onto an axis (just center projection ± radius)
    fn project_circle(center: FloatVec2, radius: f32, axis: FloatVec2) -> (f32, f32) {
        let center_proj = center.dot(axis);
        (center_proj - radius, center_proj + radius)
    }

    /// Checks if two projections overlap
    fn projections_overlap(min_a: f32, max_a: f32, min_b: f32, max_b: f32) -> bool {
        !(max_a < min_b || max_b < min_a)
    }
    fn polygon_circle(polygon: &PolygonCollider, circle: &CircleCollider) -> bool {
        let verts = &polygon.0;
        let radius = circle.diameter / 2.0;

        // 1. Check all polygon edge normals
        for i in 0..verts.len() {
            if verts.len() == 2 && i == 1 {
                break; // Skip the second vertex for lines
            }
            let a = verts[i];
            let b = verts[(i + 1) % verts.len()];
            let edge = b - a;
            // let normal = Vector2::new(-edge.y, edge.x).normalize();
            let normal = edge.perp().normalize();

            let (min_poly, max_poly) = Self::project_polygon(polygon, normal);
            let (min_circ, max_circ) = Self::project_circle(circle.position, radius, normal);

            if !Self::projections_overlap(min_poly, max_poly, min_circ, max_circ) {
                return false; // Separating axis found
            }
        }

        // 2. Check axis from circle center to closest polygon vertex
        let mut min_distance_sq = f32::MAX;
        let mut closest_vertex = verts[0];

        for &v in verts {
            let dist_sq = (v - circle.position).magnitude2();
            if dist_sq < min_distance_sq {
                min_distance_sq = dist_sq;
                closest_vertex = v;
            }
        }

        let axis_to_vertex = (closest_vertex - circle.position).normalize();

        let (min_poly, max_poly) = Self::project_polygon(polygon, axis_to_vertex);
        let (min_circ, max_circ) = Self::project_circle(circle.position, radius, axis_to_vertex);

        if !Self::projections_overlap(min_poly, max_poly, min_circ, max_circ) {
            return false; // Separating axis found
        }
        true // No separating axis found → collision
    }

    /// Main SAT collision detection function
    fn polygons_collide(poly1: &PolygonCollider, poly2: &PolygonCollider) -> bool {
        // Check axes from polygon 1
        let poly1verts = &poly1.0;
        for i in 0..poly1verts.len() {
            if poly1verts.len() == 2 && i == 1 {
                break; // Skip the second vertex for lines
            }
            let edge = poly1verts[(i + 1) % poly1verts.len()] - poly1verts[i];
            let axis = edge.perp().normalize();

            let (min_a, max_a) = Self::project_polygon(poly1, axis);
            let (min_b, max_b) = Self::project_polygon(poly2, axis);

            if !Self::projections_overlap(min_a, max_a, min_b, max_b) {
                return false; // Found separating axis
            }
        }
        // Check axes from polygon 2
        let poly2verts = &poly2.0;
        for i in 0..poly2verts.len() {
            if poly2verts.len() == 2 && i == 1 {
                break; // Skip the second vertex for lines
            }
            let edge = poly2verts[(i + 1) % poly2verts.len()] - poly2verts[i];
            let axis = edge.perp().normalize();

            let (min_a, max_a) = Self::project_polygon(poly1, axis);
            let (min_b, max_b) = Self::project_polygon(poly2, axis);

            if !Self::projections_overlap(min_a, max_a, min_b, max_b) {
                return false; // Found separating axis
            }
        }
        true // No separating axis found
    }
    fn polygon_border(polygon: &PolygonCollider, border: &BorderCollider) -> bool {
        // the only axis to check is the border normal
        let axis = border.normal.normalize();
        let (_poly_min, poly_max) = Self::project_polygon(polygon, axis);
        // project the border point onto the axis
        let border_projection = border.point_on_border.dot(axis);
        poly_max > border_projection
    }
    fn circle_border(circle: &CircleCollider, border: &BorderCollider) -> bool {
        // project the circle center onto the border normal
        let radius = circle.diameter / 2.0;
        let (_circle_min, circle_max) =
            Self::project_circle(circle.position, radius, border.normal);
        let border_projection = border.point_on_border.dot(border.normal.normalize());
        // check if the distance from the border to the circle center is less than the radius
        circle_max > border_projection
    }
    pub fn collides_with(&self, other: &Collider) -> bool {
        match (self, other) {
            (Collider::Circle(c1), Collider::Circle(c2)) => Self::circle_circle(c1, c2),
            (Collider::Circle(c), Collider::Polygon(p)) => Self::polygon_circle(p, c),
            (Collider::Polygon(p), Collider::Circle(c)) => Self::polygon_circle(p, c),
            (Collider::Polygon(p1), Collider::Polygon(p2)) => Self::polygons_collide(p1, p2),
            (Collider::Polygon(p), Collider::Border(b)) => Self::polygon_border(p, b),
            (Collider::Circle(c), Collider::Border(b)) => Self::circle_border(c, b),
            (Collider::Border(b), Collider::Polygon(p)) => Self::polygon_border(p, b),
            (Collider::Border(b), Collider::Circle(c)) => Self::circle_border(c, b),
            (Collider::Border(_), Collider::Border(_)) => {
                panic!("Border with border collision is not defined")
            }
        }
    }
}
