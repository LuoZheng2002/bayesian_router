use cgmath::SquareMatrix;

use crate::camera_uniform::CameraUniform;

pub struct OrthographicCamera {
    pub position: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl OrthographicCamera {
    pub fn new(
        position: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let camera = Self {
            position,
            target,
            up,
            left,
            right,
            bottom,
            top,
            near,
            far,
        };
        camera
    }

    pub fn get_view_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::look_at_rh(self.position, self.target, self.up)
    }

    pub fn get_proj_matrix(&self) -> cgmath::Matrix4<f32> {
        ortho(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }

    pub fn to_uniform(&self) -> CameraUniform {
        let proj_matrix = self.get_proj_matrix();
        let view_matrix = self.get_view_matrix();
        let view_proj = proj_matrix * view_matrix;
        CameraUniform {
            view_proj: view_proj.into(),
        }
    }
}

// Helper function to create orthographic projection matrix
fn ortho(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> cgmath::Matrix4<f32> {
    let mut matrix = cgmath::Matrix4::identity();

    matrix[0][0] = 2.0 / (right - left);
    matrix[1][1] = 2.0 / (top - bottom);
    matrix[2][2] = 1.0 / (far - near);

    matrix[3][0] = -(right + left) / (right - left);
    matrix[3][1] = -(top + bottom) / (top - bottom);
    matrix[3][2] = -near / (far - near);

    matrix
}
