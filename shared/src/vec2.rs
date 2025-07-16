use std::ops::{Add, Sub};

pub type FixedPoint = fixed::types::I24F8;

#[derive(Debug, Clone, PartialEq, Hash, Eq, Copy, PartialOrd, Ord)]
pub struct FixedVec2 {
    pub x: FixedPoint,
    pub y: FixedPoint,
}

impl FixedVec2 {
    pub fn new(x: FixedPoint, y: FixedPoint) -> Self {
        FixedVec2 { x, y }
    }
    pub fn to_float(&self) -> FloatVec2 {
        FloatVec2 {
            x: self.x.to_num(),
            y: self.y.to_num(),
        }
    }
    pub fn length(&self) -> FixedPoint {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn is_x_odd_y_odd(&self) -> bool {
        self.x.to_bits() & 1 == 1 && self.y.to_bits() & 1 == 1
    }
    pub fn is_sum_even(&self) -> bool {
        (self.x.to_bits() + self.y.to_bits()) % 2 == 0
    }
}

impl Sub for FixedVec2 {
    type Output = FixedVec2;

    fn sub(self, other: FixedVec2) -> FixedVec2 {
        FixedVec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Add for FixedVec2 {
    type Output = FixedVec2;

    fn add(self, other: FixedVec2) -> FixedVec2 {
        FixedVec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FloatVec2 {
    pub x: f32,
    pub y: f32,
}

impl FloatVec2 {
    pub fn to_fixed(&self) -> FixedVec2 {
        FixedVec2 {
            x: FixedPoint::from_num(self.x),
            y: FixedPoint::from_num(self.y),
        }
    }
    pub fn dot(self, other: FloatVec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn sub(self, other: FloatVec2) -> FloatVec2 {
        FloatVec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    /// Returns a vector perpendicular to self (normal to edge)
    pub fn perp(self) -> FloatVec2 {
        FloatVec2 {
            x: -self.y,
            y: self.x,
        }
    }

    /// Normalize the vector (used to prevent numerical issues)
    pub fn normalize(self) -> FloatVec2 {
        let len = (self.x * self.x + self.y * self.y).sqrt();
        if len > f32::EPSILON {
            FloatVec2 {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            self
        }
    }
    pub fn magnitude2(self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}