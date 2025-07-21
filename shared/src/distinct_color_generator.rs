use crate::color_float3::ColorFloat3;

pub struct DistinctColorGenerator {
    index: usize,
    golden_ratio_conjugate: f32,
    saturation: f32,
    value: f32,
}

impl DistinctColorGenerator {
    pub fn new() -> Self {
        Self {
            index: 0,
            golden_ratio_conjugate: 0.61803398875,
            saturation: 0.8, // vivid but not pure
            value: 0.75,     // avoid both black and white
        }
    }

    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> ColorFloat3 {
        let h = h.fract() * 6.0;
        let i = h.floor();
        let f = h - i;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));
        match i as u32 {
            0 => ColorFloat3 { r: v, g: t, b: p },
            1 => ColorFloat3 { r: q, g: v, b: p },
            2 => ColorFloat3 { r: p, g: v, b: t },
            3 => ColorFloat3 { r: p, g: q, b: v },
            4 => ColorFloat3 { r: t, g: p, b: v },
            _ => ColorFloat3 { r: v, g: p, b: q },
        }
    }
}

impl Iterator for DistinctColorGenerator {
    type Item = ColorFloat3;

    fn next(&mut self) -> Option<Self::Item> {
        let hue = ((self.index as f32) * self.golden_ratio_conjugate).fract();
        self.index += 1;
        Some(Self::hsv_to_rgb(hue, self.saturation, self.value))
    }
}
