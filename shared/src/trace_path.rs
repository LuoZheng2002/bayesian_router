use std::collections::HashMap;

use crate::{
    collider::Collider,
    hyperparameters::{HALF_PROBABILITY_RAW_SCORE, LAYER_TO_TRACE_COLOR},
    pcb_render_model::{RenderableBatch, ShapeRenderable},
    prim_shape::{CircleShape, PrimShape, RectangleShape},
    vec2::{FixedPoint, FixedVec2, FloatVec2, IntVec2},
};

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

#[derive(Debug, Clone, Copy)]
pub enum AStarNodeDirection {
    None,              // neither horizontal nor vertical
    Planar(Direction), // Direction in the plane
    Vertical {
        from_layer: usize, // Layer to place the via from
    },
}
impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::TopRight => Direction::BottomLeft,
            Direction::TopLeft => Direction::BottomRight,
            Direction::BottomRight => Direction::TopLeft,
            Direction::BottomLeft => Direction::TopRight,
        }
    }
    pub fn is_diagonal(&self) -> bool {
        matches!(
            self,
            Direction::TopRight
                | Direction::TopLeft
                | Direction::BottomRight
                | Direction::BottomLeft
        )
    }
    pub fn to_degree_angle(&self) -> f32 {
        match self {
            Direction::Up => 90.0,
            Direction::Down => 270.0,
            Direction::Left => 180.0,
            Direction::Right => 0.0,
            Direction::TopRight => 45.0,
            Direction::TopLeft => 135.0,
            Direction::BottomRight => 315.0,
            Direction::BottomLeft => 225.0,
        }
    }
    fn direction_to_int(&self) -> i32 {
        match self {
            Direction::Up => 0,
            Direction::TopRight => 1,
            Direction::Right => 2,
            Direction::BottomRight => 3,
            Direction::Down => 4,
            Direction::BottomLeft => 5,
            Direction::Left => 6,
            Direction::TopLeft => 7,
        }
    }
    pub fn is_right_angle(&self, other: Direction) -> bool {
        let self_index = self.direction_to_int();
        let other_index = other.direction_to_int();
        match (self_index - other_index + 8) % 8 {
            2 | 6 => true, // 2: right angle, 6: left angle
            _ => false,
        }
    }
    pub fn is_sharp_angle(&self, other: Direction) -> bool {
        let self_index = self.direction_to_int();
        let other_index = other.direction_to_int();
        match (self_index - other_index + 8) % 8 {
            3 | 5 => true, // 1: right 45, 3: right 135, 5: left 135, 7: left 45
            _ => false,
        }
    }
    pub fn between_sharp_angle(dir1: Direction, dir2: Direction) -> Direction {
        assert!(dir1.is_sharp_angle(dir2));
        let dir1_int = dir1.direction_to_int();
        let dir2_int = dir2.direction_to_int();
        let difference = (dir1_int - dir2_int + 8) % 8;
        match difference {
            3 => {
                Direction::int_to_direction((dir2_int + 1) % 8)
            },
            5 => {
                Direction::int_to_direction((dir2_int + 7) % 8)
            },
            _ => panic!("Not a sharp angle between directions: {:?} and {:?}", dir1, dir2),
        }
    }
    pub fn between_right_angle(dir1: Direction, dir2: Direction) -> Direction {
        assert!(dir1.is_right_angle(dir2));
        let dir1_int = dir1.direction_to_int();
        let dir2_int = dir2.direction_to_int();
        let difference = (dir1_int - dir2_int + 8) % 8;
        match difference {
            2 => {
                Direction::int_to_direction((dir2_int + 1) % 8)
            },
            6 => {
                Direction::int_to_direction((dir2_int + 7) % 8)
            },
            _ => panic!("Not a right angle between directions: {:?} and {:?}", dir1, dir2),
        }
    }
    pub fn left_45_90_135(&self, other: Direction) -> bool {
        let self_index = self.direction_to_int();
        let other_index = other.direction_to_int();
        match (self_index - other_index + 8) % 8{
            7 | 6 | 5 => true, // 7: left 45, 6: left 90, 5: left 135
            _ => false,
        }
    }
    pub fn right_45_90_135(&self, other: Direction) -> bool {
        let self_index = self.direction_to_int();
        let other_index = other.direction_to_int();
        match (self_index - other_index + 8) % 8{
            1 | 2 | 3 => true, // 1: right 45, 2: right 90, 3: right 135
            _ => false,
        }
    }
    fn int_to_direction(i: i32) -> Direction {
        match i {
            0 => Direction::Up,
            1 => Direction::TopRight,
            2 => Direction::Right,
            3 => Direction::BottomRight,
            4 => Direction::Down,
            5 => Direction::BottomLeft,
            6 => Direction::Left,
            7 => Direction::TopLeft,
            _ => panic!("Invalid direction index"),
        }
    }
    pub fn left_90_dir(&self) -> Direction {
        let new_index = (self.direction_to_int() + 6) % 8; // 6 is equivalent to -2 in mod 8
        Direction::int_to_direction(new_index)
    }
    pub fn right_90_dir(&self) -> Direction {
        let new_index = (self.direction_to_int() + 2) % 8; // 2 is equivalent to +2 in mod 8
        Direction::int_to_direction(new_index)
    }
    pub fn left_45_dir(&self) -> Direction {
        let new_index = (self.direction_to_int() + 7) % 8; // 7 is equivalent to -1 in mod 8
        Direction::int_to_direction(new_index)
    }
    pub fn right_45_dir(&self) -> Direction {
        let new_index = (self.direction_to_int() + 1) % 8; // 1 is equivalent to +1 in mod 8
        Direction::int_to_direction(new_index)
    }
    pub fn all_directions() -> Vec<Direction> {
        vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
            Direction::TopRight,
            Direction::TopLeft,
            Direction::BottomRight,
            Direction::BottomLeft,
        ]
    }
    // pub fn to_fixed_vec2(&self) -> FixedVec2 {
    //     match self {
    //         Direction::Up => FloatVec2 { x: 0.0, y: 1.0 },
    //         Direction::Down => FloatVec2 { x: 0.0, y: -1.0 },
    //         Direction::Left => FloatVec2 { x: -1.0, y: 0.0 },
    //         Direction::Right => FloatVec2 { x: 1.0, y: 0.0 },
    //         Direction::TopRight => FloatVec2 { x: 1.0, y: 1.0 },
    //         Direction::TopLeft => FloatVec2 { x: -1.0, y: 1.0 },
    //         Direction::BottomRight => FloatVec2 { x: 1.0, y: -1.0 },
    //         Direction::BottomLeft => FloatVec2 { x: -1.0, y: -1.0 },
    //     }.to_fixed()
    // }
    pub fn to_int_vec2(&self) -> IntVec2 {
        match self {
            Direction::Up => IntVec2::new(0, 1),
            Direction::Down => IntVec2::new(0, -1),
            Direction::Left => IntVec2::new(-1, 0),
            Direction::Right => IntVec2::new(1, 0),
            Direction::TopRight => IntVec2::new(1, 1),
            Direction::TopLeft => IntVec2::new(-1, 1),
            Direction::BottomRight => IntVec2::new(1, -1),
            Direction::BottomLeft => IntVec2::new(-1, -1),
        }
    }
    pub fn to_fixed_vec2(&self, scale: FixedPoint) -> FixedVec2 {
        let int_vec = self.to_int_vec2();
        FixedVec2 {
            x: FixedPoint::from_num(int_vec.x) * scale,
            y: FixedPoint::from_num(int_vec.y) * scale,
        }
    }

    pub fn is_two_points_valid_direction(start: FixedVec2, end: FixedVec2) -> bool {
        match Self::from_points(start, end) {
            Ok(direction) => {
                match direction{
                    Some(_direction) =>{
                        true
                    },
                    None => false,
                }
            }
            Err(_) => false,
        }
    }

    pub fn from_points(start: FixedVec2, end: FixedVec2) -> Result<Option<Self>, String> {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let dy_minus_dx_abs = (dy.abs() - dx.abs()).abs();
        //
        match (
            dx.partial_cmp(&0.0),
            dy.partial_cmp(&0.0),
            dy_minus_dx_abs.partial_cmp(&0.0),
        ) {
            (
                Some(std::cmp::Ordering::Equal),
                Some(std::cmp::Ordering::Greater),
                Some(std::cmp::Ordering::Greater),
            ) => Ok(Some(Direction::Up)),
            (
                Some(std::cmp::Ordering::Equal),
                Some(std::cmp::Ordering::Less),
                Some(std::cmp::Ordering::Greater),
            ) => Ok(Some(Direction::Down)),
            (
                Some(std::cmp::Ordering::Greater),
                Some(std::cmp::Ordering::Equal),
                Some(std::cmp::Ordering::Greater),
            ) => Ok(Some(Direction::Right)),
            (
                Some(std::cmp::Ordering::Less),
                Some(std::cmp::Ordering::Equal),
                Some(std::cmp::Ordering::Greater),
            ) => Ok(Some(Direction::Left)),
            (
                Some(std::cmp::Ordering::Greater),
                Some(std::cmp::Ordering::Greater),
                Some(std::cmp::Ordering::Equal),
            ) => Ok(Some(Direction::TopRight)),
            (
                Some(std::cmp::Ordering::Less),
                Some(std::cmp::Ordering::Greater),
                Some(std::cmp::Ordering::Equal),
            ) => Ok(Some(Direction::TopLeft)),
            (
                Some(std::cmp::Ordering::Greater),
                Some(std::cmp::Ordering::Less),
                Some(std::cmp::Ordering::Equal),
            ) => Ok(Some(Direction::BottomRight)),
            (
                Some(std::cmp::Ordering::Less),
                Some(std::cmp::Ordering::Less),
                Some(std::cmp::Ordering::Equal),
            ) => Ok(Some(Direction::BottomLeft)),
            (
                Some(std::cmp::Ordering::Equal),
                Some(std::cmp::Ordering::Equal),
                Some(std::cmp::Ordering::Equal),
            ) => Ok(None), // No movement, no direction),
            _ => Err(format!(
                "Invalid points for direction calculation: dx: {}, dy: {}, dy_minus_dx_abs: {}",
                dx, dy, dy_minus_dx_abs
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceSegment {
    pub start: FixedVec2, // Start point of the trace segment
    pub end: FixedVec2,   // End point of the trace segment
    pub width: f32,       // Width of the trace segment
    pub clearance: f32,   // Clearance around the trace segment
    pub layer: usize,     // Layer of the trace segment
}

impl TraceSegment {
    pub fn get_direction(&self) -> Direction {
        Direction::from_points(self.start, self.end).unwrap().unwrap()
    }
    pub fn to_shapes(&self) -> Vec<PrimShape> {
        // a trace segment is composed of two circles and a rectangle
        let start = self.start.to_float();
        let end = self.end.to_float();
        let segment_length = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2)).sqrt();
        let start_circle = PrimShape::Circle(CircleShape {
            position: start,
            diameter: self.width,
        });
        let end_circle = PrimShape::Circle(CircleShape {
            position: end,
            diameter: self.width,
        });
        let segment_rect = PrimShape::Rectangle(RectangleShape {
            position: FloatVec2 {
                x: (start.x + end.x) / 2.0,
                y: (start.y + end.y) / 2.0,
            },
            width: segment_length,
            height: self.width,
            rotation: cgmath::Deg(self.get_direction().to_degree_angle()),
        });
        vec![start_circle, end_circle, segment_rect]
    }
    pub fn to_clearance_shapes(&self) -> Vec<PrimShape> {
        // Clearance is represented by a larger rectangle around the segment
        let start = self.start.to_float();
        let end = self.end.to_float();
        let segment_length = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2)).sqrt();
        let new_width = self.width + self.clearance * 2.0;
        let new_diameter = new_width;
        let clearance_start_circle = PrimShape::Circle(CircleShape {
            position: start,
            diameter: new_diameter,
        });
        let clearance_end_circle = PrimShape::Circle(CircleShape {
            position: end,
            diameter: new_diameter,
        });
        let clearance_rect = PrimShape::Rectangle(RectangleShape {
            position: FloatVec2 {
                x: (start.x + end.x) / 2.0,
                y: (start.y + end.y) / 2.0,
            },
            width: segment_length + self.clearance * 2.0,
            height: new_width,
            rotation: cgmath::Deg(self.get_direction().to_degree_angle()),
        });
        vec![clearance_start_circle, clearance_end_circle, clearance_rect]
    }
    pub fn to_colliders(&self) -> Vec<Collider> {
        let shapes = self.to_shapes();
        shapes.iter().map(Collider::from_prim_shape).collect()
    }
    pub fn to_clearance_colliders(&self) -> Vec<Collider> {
        let clearance_shapes = self.to_clearance_shapes();
        clearance_shapes
            .iter()
            .map(Collider::from_prim_shape)
            .collect()
    }
    pub fn collides_with(&self, other: &TraceSegment) -> bool {
        if self.layer != other.layer {
            return false; // No collision if they are on different layers
        }
        let self_colliders = self.to_colliders();
        let self_clearance_colliders = self.to_clearance_colliders();
        let other_colliders = other.to_colliders();
        let other_clearance_colliders = other.to_clearance_colliders();
        for self_collider in self_colliders {
            for other_clearance_collider in &other_clearance_colliders {
                if self_collider.collides_with(other_clearance_collider) {
                    return true;
                }
            }
        }
        for self_clearance_collider in self_clearance_colliders {
            for other_collider in &other_colliders {
                if self_clearance_collider.collides_with(other_collider) {
                    return true;
                }
            }
        }
        false
    }
    pub fn to_renderables(&self, color: [f32; 4]) -> Vec<ShapeRenderable> {
        let shapes = self.to_shapes();
        shapes
            .into_iter()
            .map(|shape| ShapeRenderable { shape, color })
            .collect()
    }
    pub fn to_clearance_renderables(&self, color: [f32; 4]) -> Vec<ShapeRenderable> {
        let clearance_shapes = self.to_clearance_shapes();
        clearance_shapes
            .into_iter()
            .map(|shape| ShapeRenderable { shape, color })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Via {
    pub position: FixedVec2, // Position of the via
    pub diameter: f32,       // Diameter of the via
    pub clearance: f32,      // Clearance around the via
    pub min_layer: usize,    // Inclusive, the layer where the via starts
    pub max_layer: usize,    // Inclusive, the layer where the via ends
}

impl Via {
    pub fn to_collider(&self) -> Collider {
        let shape = PrimShape::Circle(CircleShape {
            position: self.position.to_float(),
            diameter: self.diameter,
        });
        Collider::from_prim_shape(&shape)
    }
    pub fn to_clearance_collider(&self) -> Collider {
        let clearance_shape = PrimShape::Circle(CircleShape {
            position: self.position.to_float(),
            diameter: self.diameter + self.clearance * 2.0,
        });
        Collider::from_prim_shape(&clearance_shape)
    }
    pub fn to_shape(&self) -> PrimShape {
        let via_shape = PrimShape::Circle(CircleShape {
            position: self.position.to_float(),
            diameter: self.diameter,
        });
        via_shape
    }
    pub fn to_clearance_shape(&self) -> PrimShape {
        PrimShape::Circle(CircleShape {
            position: self.position.to_float(),
            diameter: self.diameter + self.clearance * 2.0,
        })
    }
    pub fn to_renderables(&self, color: [f32; 4]) -> Vec<ShapeRenderable> {
        // let hole_shape = PrimShape::Circle(CircleShape {
        //     position: self.position.to_float(),
        //     diameter: self.diameter / 2.0, // The hole is half the diameter
        // });
        // let hole_color = [0.0, 0.0, 0.0, color[3]]; // Black hole
        let via_shape = self.to_shape();
        // let hole_renderable = ShapeRenderable {
        //     shape: hole_shape,
        //     color: hole_color,
        // };
        let via_renderable = ShapeRenderable {
            shape: via_shape,
            color,
        };
        vec![via_renderable]
    }
    pub fn to_clearance_renderables(&self, color: [f32; 4]) -> Vec<ShapeRenderable> {
        let clearance_shape = self.to_clearance_shape();
        vec![ShapeRenderable {
            shape: clearance_shape,
            color,
        }]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceAnchor {
    pub position: FixedVec2,
    pub start_layer: usize, // Inclusive, the layer where the trace starts
    pub end_layer: usize,   // Inclusive, the layer where the trace ends
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceAnchors(pub Vec<TraceAnchor>); // List of turning points in the trace path, including start and end

#[derive(Debug, Clone)]
pub struct TracePath {
    pub anchors: TraceAnchors, // List of turning points in the trace path, including start and end
    pub segments: Vec<TraceSegment>, // List of segments in the trace path
    pub vias: Vec<Via>,        // List of vias in the trace path
    pub total_length: f64,
}
// shrink?

impl TracePath {
    pub fn from_anchors(
        anchors: TraceAnchors,
        trace_width: f32,
        trace_clearance: f32,
        via_diameter: f32,
    ) -> Self{
        let anchors_vec = &anchors.0;
        let mut segments = Vec::new();
        let mut vias = Vec::new();
        let mut total_length = 0.0;
        for i in 0..anchors_vec.len() - 1 {
            let start = anchors_vec[i].position;
            let end = anchors_vec[i + 1].position;
            let segment = TraceSegment {
                start,
                end,
                width: trace_width,
                clearance: trace_clearance,
                layer: anchors_vec[i].end_layer,
            };
            let start_x: f64 = start.x.to_num();
            let start_y: f64 = start.y.to_num();
            let end_x: f64 = end.x.to_num();
            let end_y: f64 = end.y.to_num();
            let segment_length = ((end_x - start_x).powi(2) + (end_y - start_y).powi(2)).sqrt();
            total_length += segment_length;
            segments.push(segment);
        }
        for i in 1..anchors_vec.len() - 1{
            let anchor = &anchors_vec[i];            
            if anchor.start_layer != anchor.end_layer {
                let min_layer = usize::min(anchor.start_layer, anchor.end_layer);
                let max_layer = usize::max(anchor.start_layer, anchor.end_layer);
                let via = Via {
                    position: anchor.position,
                    diameter: via_diameter,
                    clearance: trace_clearance,
                    min_layer,
                    max_layer,
                };
                vias.push(via);
            }
        }
        Self {
            anchors,
            segments,
            vias,
            total_length,
        }
    }
    pub fn to_shapes(&self, num_layers: usize) -> HashMap<usize, Vec<PrimShape>> {
        let mut shapes: HashMap<usize, Vec<PrimShape>> =
            (0..num_layers).map(|layer| (layer, Vec::new())).collect();
        for segment in &self.segments {
            let segment_shapes = segment.to_shapes();
            shapes
                .get_mut(&segment.layer)
                .unwrap()
                .extend(segment_shapes);
        }
        for via in &self.vias {
            let via_shape = via.to_shape();
            for layer in via.min_layer..=via.max_layer {
                shapes.get_mut(&layer).unwrap().push(via_shape.clone());
            }
        }
        shapes
    }
    pub fn to_clearance_shapes(&self, num_layers: usize) -> HashMap<usize, Vec<PrimShape>> {
        let mut shapes: HashMap<usize, Vec<PrimShape>> =
            (0..num_layers).map(|layer| (layer, Vec::new())).collect();
        for segment in &self.segments {
            let segment_clearance_shapes = segment.to_clearance_shapes();
            shapes
                .get_mut(&segment.layer)
                .unwrap().extend(segment_clearance_shapes);
        }
        for via in &self.vias {
            let clearance_shape = via.to_clearance_shape();
            for layer in via.min_layer..=via.max_layer {
                shapes.get_mut(&layer).unwrap().push(clearance_shape.clone());
            }
        }
        shapes
    }
    pub fn to_colliders(&self, num_layers: usize) -> HashMap<usize, Vec<Collider>> {
        let mut colliders: HashMap<usize, Vec<Collider>> =
            (0..num_layers).map(|layer| (layer, Vec::new())).collect();
        for segment in &self.segments {
            let segment_colliders = segment.to_colliders();
            colliders
                .get_mut(&segment.layer)
                .unwrap()
                .extend(segment_colliders);
        }
        for via in &self.vias {
            let collider = via.to_collider();
            for layer in via.min_layer..=via.max_layer {
                colliders.get_mut(&layer).unwrap().push(collider.clone());
            }
        }
        colliders
    }
    pub fn to_clearance_colliders(&self, num_layers: usize) -> HashMap<usize, Vec<Collider>> {
        let mut colliders: HashMap<usize, Vec<Collider>> =
            (0..num_layers).map(|layer| (layer, Vec::new())).collect();
        for segment in &self.segments {
            let segment_clearance_colliders = segment.to_clearance_colliders();
            colliders
                .get_mut(&segment.layer)
                .unwrap()
                .extend(segment_clearance_colliders);
        }
        for via in &self.vias {
            let clearance_collider = via.to_clearance_collider();
            for layer in via.min_layer..=via.max_layer {
                colliders
                    .get_mut(&layer)
                    .unwrap()
                    .push(clearance_collider.clone());
            }
        }
        colliders
    }

    pub fn collides_with(&self, other: &TracePath) -> bool {
        for segment_self in &self.segments {
            for segment_other in &other.segments {
                if segment_self.collides_with(segment_other) {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_score(&self) -> f64 {
        // to do
        let score_raw = self.total_length; // placeholder for actual score calculation
        let k = f64::ln(2.0) / HALF_PROBABILITY_RAW_SCORE;
        let score = f64::exp(-k * score_raw);
        println!("total length: {}, score: {}", self.total_length, score);
        assert!(
            score >= 0.0 && score <= 1.0,
            "Score must be between 0 and 1, got: {}",
            score
        );
        score
    }

    pub fn to_renderables(&self, color: [f32; 4]) -> [RenderableBatch; 2] {
        let mut renderables = Vec::new();
        let mut clearance_renderables = Vec::new();
        let clearance_color = [color[0], color[1], color[2], color[3] / 2.0]; // semi-transparent color
        // Render the segments
        for segment in &self.segments {
            let segment_color = LAYER_TO_TRACE_COLOR[segment.layer].to_float4(color[3]/2.0);
            let segment_renderables = segment.to_renderables(segment_color);
            let segment_clearance_renderables = segment.to_clearance_renderables(clearance_color); // semi-transparent color
            renderables.extend(segment_renderables);
            clearance_renderables.extend(segment_clearance_renderables);
        }
        for via in &self.vias {
            let via_renderables = via.to_renderables(color);
            let via_clearance_renderables = via.to_clearance_renderables(clearance_color); // semi-transparent color
            renderables.extend(via_renderables);
            clearance_renderables.extend(via_clearance_renderables);
        }
        [
            RenderableBatch(renderables),
            RenderableBatch(clearance_renderables),
        ]
    }
}
