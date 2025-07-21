use cgmath::{Rotation, Rotation2};

use crate::vec2::FloatVec2;

#[derive(Debug, Clone)]
pub struct CircleShape {
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

#[derive(Debug, Clone)]
pub struct Line {
    pub start: FloatVec2,
    pub end: FloatVec2,
}

#[derive(Debug, Clone)]
pub enum PrimShape {
    Circle(CircleShape),
    Rectangle(RectangleShape),
    Line(Line),
}
