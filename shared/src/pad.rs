use std::f32::consts::PI;

use cgmath::{Rad, Vector2};

use crate::{
    pcb_render_model::ShapeRenderable, prim_shape::{CircleShape, PrimShape, RectangleShape}, vec2::FloatVec2
};

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
    pub position: FloatVec2,
    pub shape: PadShape,
    pub rotation: cgmath::Deg<f32>, // Rotation in degrees
    pub clearance: f32,             // Clearance around the pad
}


impl Pad {
    fn rounded_rect_to_shapes(width: f32, height: f32, corner_radius: f32, position: FloatVec2, rotation: cgmath::Deg<f32>) -> Vec<PrimShape>{
        let vertical_rectangle_shape = RectangleShape{
            position,
            width: width - 2.0 * corner_radius,
            height,
            rotation,
        };

        let horizontal_rectangle_shape = RectangleShape{
            position,
            width,
            height: height - 2.0 * corner_radius,
            rotation,
        };
        let dy_abs = (height / 2.0 - corner_radius).abs();
        let dx_abs = (width / 2.0 - corner_radius).abs();
        let translation_matrix1 = cgmath::Matrix3::from_translation(Vector2{ x: dx_abs, y: dy_abs });
        let translation_matrix2 = cgmath::Matrix3::from_translation(Vector2{ x: -dx_abs, y: dy_abs });
        let translation_matrix3 = cgmath::Matrix3::from_translation(Vector2{ x: dx_abs, y: -dy_abs });
        let translation_matrix4 = cgmath::Matrix3::from_translation(Vector2{ x: -dx_abs, y: -dy_abs });
        // let rotation_matrix = cgmath::Matrix3::from_axis_angle(cgmath::Vector3::unit_z(), self.rotation);
        let rotation_radians = Rad::from(rotation);
        let cos_theta = f32::cos(rotation_radians.0);
        let sin_theta = f32::sin(rotation_radians.0);
        let rotation_matrix = cgmath::Matrix3::new(
            cos_theta, -sin_theta, 0.0,
            sin_theta,  cos_theta, 0.0,
            0.0,        0.0,       1.0
        );
        let compound_matrix1 = rotation_matrix * translation_matrix1;
        let compound_matrix2 = rotation_matrix * translation_matrix2;
        let compound_matrix3 = rotation_matrix * translation_matrix3;
        let compound_matrix4 = rotation_matrix * translation_matrix4;
        fn extract_translation(matrix: cgmath::Matrix3<f32>) -> FloatVec2 {
            FloatVec2 {
                x: matrix.z.x,
                y: matrix.z.y,
            }
        }
        let translation1 = extract_translation(compound_matrix1);
        let translation2 = extract_translation(compound_matrix2);
        let translation3 = extract_translation(compound_matrix3);
        let translation4 = extract_translation(compound_matrix4);
        let new_position1 = position + translation1;
        let new_position2 = position + translation2;
        let new_position3 = position + translation3;
        let new_position4 = position + translation4;
        let circle_shape1 = CircleShape {
            position: new_position1,
            diameter: corner_radius * 2.0,
        };
        let circle_shape2 = CircleShape {
            position: new_position2,
            diameter: corner_radius * 2.0,
        };
        let circle_shape3 = CircleShape {
            position: new_position3,
            diameter: corner_radius * 2.0,
        };
        let circle_shape4 = CircleShape {
            position: new_position4,
            diameter: corner_radius * 2.0,
        };
        vec![
            PrimShape::Rectangle(vertical_rectangle_shape),
            PrimShape::Rectangle(horizontal_rectangle_shape),
            PrimShape::Circle(circle_shape1),
            PrimShape::Circle(circle_shape2),
            PrimShape::Circle(circle_shape3),
            PrimShape::Circle(circle_shape4),
        ]
    }

    pub fn to_shapes(&self) -> Vec<PrimShape> {
        match &self.shape {
            PadShape::Circle { diameter } => vec![PrimShape::Circle (
                CircleShape {
                    position: self.position,
                    diameter: *diameter,
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
            PadShape::RoundRect { width, height, corner_radius } => {
                Self::rounded_rect_to_shapes(*width, *height, *corner_radius, self.position, self.rotation)
            }
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
            PadShape::Rectangle { width, height } => vec![PrimShape::Rectangle(
                RectangleShape {
                    position: self.position,
                    width: width + self.clearance * 2.0,
                    height: height + self.clearance * 2.0,
                    rotation: self.rotation,
                }
            )],
            // to do: make a finer clearance shape
            PadShape::RoundRect { width, height, corner_radius } => {
                let clearance = self.clearance;
                let clearance_width = width + clearance * 2.0;
                let clearance_height = height + clearance * 2.0;
                let clearance_corner_radius = corner_radius + clearance;
                Self::rounded_rect_to_shapes(clearance_width, clearance_height, clearance_corner_radius, self.position, self.rotation)
            },
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
