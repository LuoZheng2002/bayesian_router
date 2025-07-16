#[derive(Debug, Clone)]
pub enum PadShape {
    Circle {
        diameter: f32,
    },
    Rectangle {
        width: f32,
        height: f32,
    },
    RoundRect {
        width: f32,
        height: f32,
        corner_radius: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PadName(pub String);

#[derive(Debug, Clone)]
pub struct Pad {
    pub name: PadName,
    pub position: (f64, f64),
    pub shape: PadShape,
    pub rotation: cgmath::Deg<f32>, // Rotation in degrees
    pub clearance: f32,             // Clearance around the pad
}
