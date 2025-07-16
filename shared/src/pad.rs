use crate::{
    pcb_render_model::ShapeRenderable, prim_shape::{CircleShape, PrimShape, RectangleShape}, vec2::FloatVec2
};

#[derive(Debug, Clone)]
pub enum PadShape {
    Circle {
        diameter: f32,
    },
    Square {
        side_length: f32,
    },
    Rectangle {
        width: f32,
        height: f32,
    },
}

#[derive(Debug, Clone)]
pub struct Pad {
    pub position: FloatVec2,
    pub shape: PadShape,
    pub rotation: cgmath::Deg<f32>, // Rotation in degrees
    pub clearance: f32,             // Clearance around the pad
}

impl Pad {
    pub fn to_shapes(&self) -> Vec<PrimShape> {
        match &self.shape {
            PadShape::Circle { diameter } => vec![PrimShape::Circle (
                CircleShape {
                    position: self.position,
                    diameter: *diameter,
                }
            )],
            PadShape::Square { side_length } => vec![PrimShape::Rectangle (
                RectangleShape {
                    position: self.position,
                    width: *side_length,
                    height: *side_length,
                    rotation: self.rotation,
                }
            )],
            PadShape::Rectangle { width, height } => vec![PrimShape::Rectangle (
                RectangleShape {
                    position: self.position,
                    width: *width,
                    height: *height,
                    rotation: self.rotation,
                }
            )],
        }
    }
    pub fn to_clearance_shapes(&self) -> Vec<PrimShape> {
        match &self.shape {            
            PadShape::Circle { diameter } => vec![PrimShape::Circle(
                CircleShape {
                    position: self.position,
                    diameter: diameter + self.clearance * 2.0,
                }
            )],
            PadShape::Square { side_length } => vec![PrimShape::Rectangle(
                RectangleShape {
                    position: self.position,
                    width: side_length + self.clearance * 2.0,
                    height: side_length + self.clearance * 2.0,
                    rotation: self.rotation,
                }
            )],
            PadShape::Rectangle { width, height } => vec![PrimShape::Rectangle(
                RectangleShape {
                    position: self.position,
                    width: width + self.clearance * 2.0,
                    height: height + self.clearance * 2.0,
                    rotation: self.rotation,
                }
            )],
        }
    }
    pub fn to_renderables(&self, color: [f32; 4])-> Vec<ShapeRenderable> {
        let shapes = self.to_shapes();
        shapes
            .into_iter()
            .map(|shape| ShapeRenderable {
                shape,
                color,
            })
            .collect()
    }
    pub fn to_clearance_renderables(&self, color: [f32; 4]) -> Vec<ShapeRenderable> {
        let clearance_shapes = self.to_clearance_shapes();
        clearance_shapes
            .into_iter()
            .map(|shape| ShapeRenderable {
                shape,
                color,
            })
            .collect()
    }
}
