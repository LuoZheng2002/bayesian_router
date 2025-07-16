use cgmath::{Rotation, Rotation2};

use crate::vec2::FloatVec2;


#[derive(Debug, Clone)]
pub struct CircleShape{
    pub position: FloatVec2,
    pub diameter: f32,
}
#[derive(Debug, Clone)]
pub struct RectangleShape {
    pub position: FloatVec2, // center position of the rectangle
    pub width: f32,
    pub height: f32,
    pub rotation: cgmath::Deg<f32>, // Rotation counterclockwise in degrees
}

impl RectangleShape {
    pub fn to_polygon(&self) -> Polygon {
        let hw = self.width / 2.0;
        let hh = self.height / 2.0;

        // Corner positions before rotation (relative to center)
        let corners = [
            cgmath::Vector2::new(-hw, -hh),
            cgmath::Vector2::new(hw, -hh),
            cgmath::Vector2::new(hw, hh),
            cgmath::Vector2::new(-hw, hh),
        ];
        // Convert rotation to radians
        let rotation_rad: cgmath::Rad<f32> = self.rotation.into();

        // Create rotation matrix
        let rotation = cgmath::Basis2::from_angle(rotation_rad);

        // Apply rotation and translate to position
        let rotated_corners: Vec<FloatVec2> = corners
            .iter()
            .map(|corner| {
                let rotated_corner = rotation.rotate_vector(*corner);

                // self.position + rotation.rotate_vector(*corner)
                FloatVec2 {
                    x: self.position.x + rotated_corner.x,
                    y: self.position.y + rotated_corner.y,
                }
            })
            .collect();

        Polygon(rotated_corners)
    }
}

#[derive(Debug, Clone)]
pub enum PrimShape {
    Circle(CircleShape),
    Rectangle(RectangleShape),
}

#[derive(Debug, Clone)]
pub struct Polygon(pub Vec<FloatVec2>);

impl PrimShape {

    fn circle_collides_with_circle(circle1: &CircleShape, circle2: &CircleShape) -> bool {
        let radius1 = circle1.diameter / 2.0;
        let radius2 = circle2.diameter / 2.0;
        let distance_squared = (circle1.position.x - circle2.position.x).powi(2)
            + (circle1.position.y - circle2.position.y).powi(2);
        distance_squared < (radius1 + radius2).powi(2)
    }
    fn rectangle_collides_with_rectangle(rect1: &RectangleShape, rect2: &RectangleShape) -> bool {
        let rect1_polygon = rect1.to_polygon();
        let rect2_polygon = rect2.to_polygon();
        Self::polygons_collide(&rect1_polygon, &rect2_polygon)
    }
    /// Projects a polygon onto an axis and returns the min and max projection scalars
    fn project_polygon(polygon: &Polygon, axis: FloatVec2) -> (f32, f32) {
        let mut min = polygon.0[0].dot(axis);
        let mut max = min;

        for &point in polygon.0.iter().skip(1) {
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

    /// Main SAT collision detection function
    fn polygons_collide(poly1: &Polygon, poly2: &Polygon) -> bool {
        // Check axes from polygon 1
        for i in 0..poly1.0.len() {
            let edge = poly1.0[(i + 1) % poly1.0.len()].sub(poly1.0[i]);
            let axis = edge.perp().normalize();

            let (min_a, max_a) = Self::project_polygon(poly1, axis);
            let (min_b, max_b) = Self::project_polygon(poly2, axis);

            if !Self::projections_overlap(min_a, max_a, min_b, max_b) {
                return false; // Found separating axis
            }
        }

        // Check axes from polygon 2
        for i in 0..poly2.0.len() {
            let edge = poly2.0[(i + 1) % poly2.0.len()].sub(poly2.0[i]);
            let axis = edge.perp().normalize();

            let (min_a, max_a) = Self::project_polygon(poly1, axis);
            let (min_b, max_b) = Self::project_polygon(poly2, axis);

            if !Self::projections_overlap(min_a, max_a, min_b, max_b) {
                return false; // Found separating axis
            }
        }
        true // No separating axis found
    }

    /// Main function: Polygon vs Circle collision
    pub fn polygon_circle_collide(polygon: &Polygon, circle: &CircleShape) -> bool {
        let Polygon(ref verts) = *polygon;
        let radius = circle.diameter / 2.0;

        // 1. Check all polygon edge normals
        for i in 0..verts.len() {
            let a = verts[i];
            let b = verts[(i + 1) % verts.len()];
            let edge = b.sub(a);
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
            let dist_sq = v.sub(circle.position).magnitude2();
            if dist_sq < min_distance_sq {
                min_distance_sq = dist_sq;
                closest_vertex = v;
            }
        }

        let axis_to_vertex = (closest_vertex.sub(circle.position)).normalize();

        let (min_poly, max_poly) = Self::project_polygon(polygon, axis_to_vertex);
        let (min_circ, max_circ) = Self::project_circle(circle.position, radius, axis_to_vertex);

        if !Self::projections_overlap(min_poly, max_poly, min_circ, max_circ) {
            return false; // Separating axis found
        }

        true // No separating axis found → collision
    }
    fn circle_collides_with_rectangle(circle: &CircleShape, rectangle: &RectangleShape) -> bool {
        let polygon = rectangle.to_polygon();
        Self::polygon_circle_collide(&polygon, circle)
    }
    pub fn collides_with(&self, other: &PrimShape) -> bool {
        match (self, other) {
            (PrimShape::Circle(circle1), PrimShape::Circle(circle2)) => {
                Self::circle_collides_with_circle(circle1, circle2)
            }
            (PrimShape::Circle(circle), PrimShape::Rectangle(rectangle)) => {
                Self::circle_collides_with_rectangle(circle, rectangle)
            }
            (PrimShape::Rectangle(rectangle), PrimShape::Circle(circle)) => {
                Self::circle_collides_with_rectangle(circle, rectangle)
            }
            (PrimShape::Rectangle(rect1), PrimShape::Rectangle(rect2)) => {
                Self::rectangle_collides_with_rectangle(rect1, rect2)
            }
        }
    }
}
