#[derive(Debug, Clone)]
pub struct OutlinePolygon(pub Vec<(f32, f32)>);

use cgmath::{Rotation, Rotation2};

#[derive(Debug, Clone)]
pub struct CircleShape{
    pub position: (f32, f32),
    pub diameter: f32,
}
#[derive(Debug, Clone)]
pub struct RectangleShape {
    pub position: (f32, f32), // center position of the rectangle
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
        let rotated_corners: Vec<(f32, f32)> = corners
            .iter()
            .map(|corner| {
                let rotated_corner = rotation.rotate_vector(*corner);

                // self.position + rotation.rotate_vector(*corner)
                (self.position.0 + rotated_corner.x, self.position.1 + rotated_corner.y)
            })
            .collect();

        Polygon(rotated_corners)
    }
}

#[derive(Debug, Clone)]
pub struct RoundedRectShape {
    pub position: (f32, f32), // center position of the rectangle
    pub width: f32,
    pub height: f32,
    pub rotation: cgmath::Deg<f32>, // Rotation counterclockwise in degrees
    pub corner_radius: f32,
}

#[derive(Debug, Clone)]
pub enum PrimShape {
    Circle(CircleShape),
    Rectangle(RectangleShape),
    RoundedRectShape(RoundedRectShape),
}

#[derive(Debug, Clone)]
pub struct Polygon(pub Vec<(f32, f32)>);

#[derive(Debug, Clone)]
pub struct Line(pub (f32, f32), pub (f32, f32));