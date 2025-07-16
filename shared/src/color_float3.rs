#[derive(Debug, Clone, Copy)]
pub struct ColorFloat3 {
    pub r: f32, // [0.0, 1.0]
    pub g: f32,
    pub b: f32,
}

impl ColorFloat3 {
    pub fn to_float4(&self, alpha: f32) -> [f32; 4] {
        [self.r, self.g, self.b, alpha]
    }
}