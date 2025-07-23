use crate::vec2::{FixedVec2, FloatVec2};

pub fn octile_distance_fixed(start: FixedVec2, end: FixedVec2) -> f64 {
    let start = start.to_float();
    let end = end.to_float();
    let dx = (end.x - start.x).abs() as f64;
    let dy = (end.y - start.y).abs() as f64;
    f64::max(dx, dy) + (f64::sqrt(2.0) - 1.0) * f64::min(dx, dy)
}

pub fn octile_distance_float(start: FloatVec2, end: FloatVec2) -> f32 {
    let dx = (end.x - start.x).abs();
    let dy = (end.y - start.y).abs();
    f32::max(dx, dy) + (f32::sqrt(2.0) - 1.0) * f32::min(dx, dy)
}