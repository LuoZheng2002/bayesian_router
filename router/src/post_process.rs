use fixed::traits::Fixed;
// use crate::block_or_sleep::{block_or_sleep, block_thread};
use shared::{
    binary_heap_item::BinaryHeapItem,
    hyperparameters::{ASTAR_STRIDE, DISPLAY_OPTIMIZATION, OPTIMIZATION_PRO},
    pcb_render_model::{PcbRenderModel, RenderableBatch, ShapeRenderable, UpdatePcbRenderModel},
    prim_shape::{CircleShape, PrimShape, RectangleShape},
    trace_path::{Direction, TraceAnchor, TraceAnchors, TracePath, TraceSegment, Via},
    vec2::{FixedPoint, FixedVec2, FloatVec2, IntVec2},
};

use plotters::prelude::*;
use wgpu::rwh::XcbWindowHandle;
use std::ops::Range;
use std::path::Path;

pub fn draw_tracepath_to_file(
    trace_path: &TracePath,
    output_path: &Path,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let (x_range, y_range) = calculate_plot_range(trace_path);

    let mut chart = ChartBuilder::on(&root)
        .caption("Trace Path Visualization", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_range, y_range)?;

    chart.configure_mesh().draw()?;

    for segment in &trace_path.segments {
        let line_style = ShapeStyle {
            color: BLUE.mix(0.7),
            filled: false,
            stroke_width: 2,
        };
        chart.draw_series(std::iter::once(PathElement::new(
            vec![
                (f64::from(segment.start.x), f64::from(segment.start.y)),
                (f64::from(segment.end.x), f64::from(segment.end.y)),
            ],
            line_style,
        )))?;
    }

    let anchor_points: Vec<_> = trace_path
        .anchors
        .0
        .iter()
        .map(|p| (f64::from(p.position.x), f64::from(p.position.y)))
        .collect();

    chart.draw_series(
        anchor_points
            .iter()
            .map(|(x, y)| Circle::new((*x, *y), 5, RED.filled())),
    )?;

    root.draw(&Text::new(
        format!("Total Length: {:.2}", trace_path.total_length),
        (50i32, 10i32),
        ("sans-serif", 20).into_font().color(&BLACK),
    ))?;

    Ok(())
}




fn calculate_plot_range(trace_path: &TracePath) -> (Range<f64>, Range<f64>) {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    for anchor in &trace_path.anchors.0 {
        min_x = min_x.min(f64::from(anchor.position.x));
        max_x = max_x.max(f64::from(anchor.position.x));
        min_y = min_y.min(f64::from(anchor.position.y));
        max_y = max_y.max(f64::from(anchor.position.y));
    }

    let x_margin = (max_x - min_x) * 0.1;
    let y_margin = (max_y - min_y) * 0.1;

    (
        (min_x - x_margin)..(max_x + x_margin),
        (min_y - y_margin)..(max_y + y_margin),
    )
}




fn is_convex(dir1: Direction, dir2: Direction, dir3: Direction) -> bool {
    let angle1 = (dir1.to_degree_angle() - dir3.to_degree_angle()).abs();
    let angle2 = (dir1.to_degree_angle() + dir3.to_degree_angle()).abs() / 2.0;
    (angle1 == 90.0 || angle1 == 270.0) && angle2 == dir2.to_degree_angle()
}



// fn anchor_to_tracepath(
//     anchors: Vec<TraceAnchor>,
//     width: f32,
//     clearance: f32,
//     via_diameter: f32,
// ) -> TracePath {
//     let mut segments: Vec<TraceSegment> = Vec::new();
//     let mut vias: Vec<Via> = Vec::new(); // initializes with the end position
//     for i in 0..anchors.len() - 1 {
//         let start_anchor = &anchors[i];
//         let end_anchor = &anchors[i + 1];
//         assert!(
//             start_anchor.end_layer == end_anchor.start_layer,
//             "The end layer of the start anchor should match the start layer of the end anchor"
//         );
//         assert_ne!(
//             start_anchor.position, end_anchor.position,
//             "Start and end positions should not be the same"
//         );
//         let segment = TraceSegment {
//             start: start_anchor.position,
//             end: end_anchor.position,
//             layer: start_anchor.end_layer,
//             width,
//             clearance,
//         };
//         segments.push(segment);
//         if start_anchor.start_layer != start_anchor.end_layer {
//             // if the start and end layers are different, we need to add a via
//             let via = Via {
//                 position: start_anchor.position,
//                 clearance,
//                 diameter: via_diameter,
//                 min_layer: usize::min(start_anchor.start_layer, start_anchor.end_layer),
//                 max_layer: usize::max(start_anchor.start_layer, start_anchor.end_layer),
//             };
//             vias.push(via);
//         }
//     }
//     let anchors_new = TraceAnchors(anchors);
//     TracePath {
//         anchors: anchors_new,
//         segments,
//         vias,
//         total_length,
//     }
// }


// merge path

// parallel shift

// loop:
// convex and merge
// end loop

// loop:
// cut sharp corners to right angle and blunt angle
// convex and merge
// end loop

// cut right angle to blunt angles
// convex and merge

pub fn binary_approach_to_obstacles(
    length_to_traces: &dyn Fn(FixedPoint) -> Vec<(FixedVec2, FixedVec2)>,
    start_length: FixedPoint,
    end_length: FixedPoint,
    check_collision_for_trace: &dyn Fn(FixedVec2, FixedVec2, f32, f32, usize) -> bool,
    trace_width: f32,
    trace_clearance: f32,
    layer: usize,
)->FixedPoint{
    println!("Called binary approach to obstacles");
    assert!(start_length < end_length, "start_length should be less than end_length");
    let mut lower_bound = start_length;
    let mut upper_bound = end_length;
    while lower_bound + FixedPoint::DELTA < upper_bound {
        let mid_length = (lower_bound + upper_bound) / 2;
        let traces = length_to_traces(mid_length);
        // assert_ne!(start_position, end_position, "assert 2");
        let mut found_collision = false;
        for trace in traces{
            let start_position = trace.0;
            let end_position = trace.1;
            if check_collision_for_trace(
                start_position,
                end_position,
                trace_width,
                trace_clearance,
                layer,
            ) {
                found_collision = true;
                break;
            }
        }
        if found_collision {
            upper_bound = mid_length; // collision found, search in the lower half
        } else {
            lower_bound = mid_length; // no collision, search in the upper half
        }
    }
    let result_length = lower_bound;
    result_length
}

pub fn try_parallel_shift(
    optimized: &mut Vec<TraceAnchor>,
    check_collision_for_trace: &dyn Fn(FixedVec2, FixedVec2, f32, f32, usize) -> bool,
    trace_width: f32,
    trace_clearance: f32,
)-> bool{
    for i in 0..optimized.len() - 3 {
        // Check for parallel segments that can be optimized
        // trace shifting
        let p0 = optimized[i].position;
        let p1 = optimized[i + 1].position;
        let p2 = optimized[i + 2].position;
        let p3 = optimized[i + 3].position;
        assert!(optimized[i].end_layer == optimized[i + 1].start_layer);
        assert!(optimized[i + 1].end_layer == optimized[i + 2].start_layer);
        assert!(optimized[i + 2].end_layer == optimized[i + 3].start_layer);
        if optimized[i + 1].start_layer != optimized[i + 1].end_layer
            || optimized[i + 2].start_layer != optimized[i + 2].end_layer{
            continue;
        }
        let my_layer = optimized[i].end_layer;
        // let dir0: Option<Direction> =  if i == 0 {
        //     None
        // } else {
        //     Direction::from_points(optimized[i - 1].position, p0).unwrap()
        // };
        let dir1 = Direction::from_points(p0, p1).unwrap();
        let dir1 = match dir1 {
            Some(dir) => dir,
            None => continue, // not a valid direction
        };
        let dir2 = Direction::from_points(p1, p2).unwrap();
        let dir2 = match dir2 {
            Some(dir) => dir,
            None => continue, // not a valid direction
        };
        let dir3 = Direction::from_points(p2, p3).unwrap();
        let dir3 = match dir3 {
            Some(dir) => dir,
            None => continue, // not a valid direction
        };
        // let dir4 = if i == optimized.len() - 4 {None} else {Some(Direction::from_points(p3, optimized[i + 4].position).unwrap())};
        if dir1 != dir3{
            continue; // not parallel
        }
        assert!(dir1 != dir2, "dir1 and dir2 should not be the same, dir1: {:?}, dir2: {:?}", dir1, dir2);
        println!("Found a parallel shift");
        let new_point1 = FixedVec2 {
            x: p0.x + p2.x - p1.x,
            y: p0.y + p2.y - p1.y,
        };
        let new_point2 = FixedVec2 {
            x: p3.x - p2.x + p1.x,
            y: p3.y - p2.y + p1.y,
        };

        let length_0_1 = FixedPoint::max((p1 - p0).x.abs(), (p1 - p0).y.abs());
        let length_2_3 = FixedPoint::max((p3 - p2).x.abs(), (p3 - p2).y.abs());

        let delta_0_1 = -dir1.to_fixed_vec2(length_0_1);
        let delta_2_3 = dir3.to_fixed_vec2(length_2_3); // dir1 = dir3
        let new_point_left1 = p1 + delta_0_1;
        let new_point_left2 = p2 + delta_0_1;
        let new_point_right1 = p1 + delta_2_3;
        let new_point_right2 = p2 + delta_2_3;
        assert_eq!(new_point_left1, p0);
        assert_eq!(new_point_left2, new_point1);
        assert_eq!(new_point_right1, new_point2);
        assert_eq!(new_point_right2, p3);


        if !check_collision_for_trace(
            new_point_left1,
            new_point_left2,
            trace_width,
            trace_clearance,
            my_layer,
        ) && !check_collision_for_trace(
            new_point_left2,
            p3,
            trace_width,
            trace_clearance,
            my_layer,
        ) {
            // If no collision is detected, we can safely update the positions
            println!("Successfully shifted left");
            // optimized[i + 1].position = new_point_left1;
            optimized[i + 2].position = new_point_left2;            
            optimized.remove(i + 1);
            return true;
        }
        if !check_collision_for_trace(
            new_point_right1,
            new_point_right2,
            trace_width,
            trace_clearance,
            my_layer,
        ) && !check_collision_for_trace(
            new_point_right1,
            p0,
            trace_width,
            trace_clearance,
            my_layer,
        ) {
            // If no collision is detected, we can safely update the positions
            println!("Successfully shifted right");
            optimized[i + 1].position = new_point_right1;
            // optimized[i + 2].position = new_point_right2;
            optimized.remove(i + 2);
            return true;
        }
        println!("Failed to shift left or right, trying to stick to obstacles");
        let length_to_trace = |length: FixedPoint| {
            let start_position = p1 - dir1.to_fixed_vec2(length);
            let end_position = p2 - dir1.to_fixed_vec2(length);
            vec![
                (p0, start_position),
                (start_position, end_position),
                (end_position, p3),
            ]
        };
        println!("Called binary approach to obstacles in try_parallel_shift");
        let length = binary_approach_to_obstacles(&length_to_trace, FixedPoint::ZERO, length_0_1, check_collision_for_trace, trace_width, trace_clearance, my_layer);
        if length == FixedPoint::ZERO {
            println!("In trying to stick to obstacles, length = 0, fail");
            continue; // no valid length found
        }
        println!("Successfully sticked to an obstacle");
        optimized[i + 1].position = p1 - dir1.to_fixed_vec2(length);
        optimized[i + 2].position = p2 - dir1.to_fixed_vec2(length);
        return true;       
    }
    false
}

#[derive(Debug, Clone, Copy)]
pub struct Line{
    pub point: FixedVec2,
    pub dir: IntVec2,
}
impl Line{
    pub fn new(point: FixedVec2, dir: IntVec2) -> Self {
        Line { point, dir }
    }
    pub fn intersection(&self, other: &Self) -> FixedVec2{
        let x_numerator = -other.dir.y * (other.point.x - self.point.x) + other.dir.x * (other.point.y - self.point.y);
        let x_denominator = -self.dir.x * other.dir.y + self.dir.y * other.dir.x;
        let x_denominator = FixedPoint::from_num(x_denominator);
        assert!(x_denominator != FixedPoint::ZERO, "Lines are parallel, no intersection");
        let x = x_numerator / x_denominator;
        let result = self.point + self.dir.to_fixed() * x_numerator / x_denominator;
        let y = if other.dir.x != FixedPoint::ZERO {
            (result.x - other.point.x) / other.dir.x
        } else {
            (result.y - other.point.y) / other.dir.y
        };
        assert!(other.point.x + other.dir.x * y == result.x && other.point.y + other.dir.y * y == result.y, "Intersection point does not lie on the other line");
        result        
    }
}


pub fn try_convex_and_merge(
    optimized: &mut Vec<TraceAnchor>,
    check_collision_for_trace: &dyn Fn(FixedVec2, FixedVec2, f32, f32, usize) -> bool,
    trace_width: f32,
    trace_clearance: f32,
)-> bool{
    for i in 0.. optimized.len() - 3{
        let p0 = optimized[i].position;
        let p1 = optimized[i + 1].position;
        let p2 = optimized[i + 2].position;
        let p3 = optimized[i + 3].position;
        assert!(optimized[i].end_layer == optimized[i + 1].start_layer);
        assert!(optimized[i + 1].end_layer == optimized[i + 2].start_layer);
        assert!(optimized[i + 2].end_layer == optimized[i + 3].start_layer);
        if optimized[i + 1].start_layer != optimized[i + 1].end_layer
            || optimized[i + 2].start_layer != optimized[i + 2].end_layer{
            continue;
        }
        let my_layer = optimized[i].end_layer;
        // let dir0: Option<Direction> =  if i == 0 {
        //     None
        // } else {
        //     Direction::from_points(optimized[i - 1].position, p0).unwrap()
        // };
        let dir1 = match Direction::from_points(p0, p1).unwrap(){
            Some(dir) => dir,
            None => continue, // not a valid direction
        };
        let dir2 = Direction::from_points(p1, p2).unwrap();
        let dir3 = match Direction::from_points(p2, p3).unwrap(){
            Some(dir) => dir,
            None => continue, // not a valid direction
        };
        println!("dir1: {:?}, dir2: {:?}, dir3: {:?}", dir1, dir2, dir3);
        let dir2 = match dir2 {
            Some(dir) => dir,
            None => if dir1.is_sharp_angle(dir3){
                Direction::between_sharp_angle(dir1, dir3)
            }else if dir1.is_right_angle(dir3){
                Direction::between_right_angle(dir1, dir3)
            }else{
                println!("dir2 is None, but dir 1 and dir 3 do not form sharp or right angle");
                continue;
            }
        };
        let left_spin = dir2.left_45_90_135(dir1) && dir3.left_45_90_135(dir2);
        let right_spin = dir2.right_45_90_135(dir1) && dir3.right_45_90_135(dir2);
        if !left_spin && !right_spin {
            println!("neither left spin or right spin, skipping");
            continue; // not a convex corner
        }
        println!("found left spin or right spin");
        let line1 = Line::new(p0, dir1.to_int_vec2());
        let line2 = Line::new(p1, dir2.to_int_vec2());
        let line3 = Line::new(p2, dir3.to_int_vec2());
        let line2_normal_dir = if left_spin{
            println!("Found a left spin, anchor indices: {}, {}, {}, {}", i, i + 1, i + 2, i + 3);
            dir2.left_90_dir()
        }else{
            assert!(right_spin);
            println!("Found a right spin, anchor indices: {}, {}, {}, {}", i, i + 1, i + 2, i + 3);
            dir2.right_90_dir()
        };
        println!("line 2 normal direction: {:?}", line2_normal_dir);
        let line2_normal_vec = line2_normal_dir.to_fixed_vec2(FixedPoint::ONE);
        let relative_distance_1 = {
            let dx_0_1 = (p0.x - p1.x) * line2_normal_vec.x;
            assert!(dx_0_1 >= FixedPoint::ZERO, "dx_0_1 should be non-negative, got {}, p0.x: {}, p1.x: {}, vec.x: {}", dx_0_1, p0.x, p1.x, line2_normal_vec.x);
            let dy_0_1 = (p0.y - p1.y) * line2_normal_vec.y;
            assert!(dy_0_1 >= FixedPoint::ZERO, "dy_0_1 should be non-negative, got {}, p0.y: {}, p1.y: {}, vec.y: {}", dy_0_1, p0.y, p1.y, line2_normal_vec.y);
            FixedPoint::max(dx_0_1, dy_0_1)
        };
        println!("Relative distance 1: {}", relative_distance_1);
        let relative_distance_2 = {
            let dx_3_2 = (p3.x - p2.x) * line2_normal_vec.x;
            assert!(dx_3_2 >= FixedPoint::ZERO, "dx_3_2 should be non-negative");
            let dy_3_2 = (p3.y - p2.y) * line2_normal_vec.y;
            assert!(dy_3_2 >= FixedPoint::ZERO, "dy_3_2 should be non-negative");
            FixedPoint::max(dx_3_2, dy_3_2)
        };
        println!("Relative distance 2: {}", relative_distance_2);
        let (new_point1, new_point2) = if relative_distance_1 <= relative_distance_2{
            let mut new_parallel_line = line2.clone();
            let offset = if line2_normal_dir.is_diagonal(){
                line2_normal_dir.to_fixed_vec2(relative_distance_1) / FixedPoint::from_num(2.0)
            } else{
                line2_normal_dir.to_fixed_vec2(relative_distance_1)
            };
            new_parallel_line.point = p1 + offset;
            let new_point1 = new_parallel_line.intersection(&line1);
            let new_point2 = new_parallel_line.intersection(&line3);
            // assert_eq!(new_point1, p0);
            (new_point1, new_point2)
        }else{
            let mut new_parallel_line = line2.clone();
            let offset = if line2_normal_dir.is_diagonal(){
                line2_normal_dir.to_fixed_vec2(relative_distance_2) / FixedPoint::from_num(2.0)
            } else{
                line2_normal_dir.to_fixed_vec2(relative_distance_2)
            };
            new_parallel_line.point = p1 + offset;
            let new_point1 = new_parallel_line.intersection(&line1);
            let new_point2 = new_parallel_line.intersection(&line3);
            // assert_eq!(new_point2, p3);
            (new_point1, new_point2)
        };
        if new_point1 == p1{
            assert!(new_point2 == p2, "new_point1 is p1, but new_point2 is not p2, new_point2: {:?}", new_point2);
            println!("new_point1 is p1, no need to change, skipping");
            continue;
        }
        if new_point1 != new_point2 && !check_collision_for_trace(
            new_point1,
            new_point2,
            trace_width,
            trace_clearance,
            my_layer,
        ) {
            // If no collision is detected, we can safely update the positions
            optimized[i + 1].position = new_point1;
            optimized[i + 2].position = new_point2;
            if relative_distance_1 < relative_distance_2 && p0 == new_point1{
                optimized.remove(i + 1);
            }else if relative_distance_1 > relative_distance_2 && p3 == new_point2{
                optimized.remove(i + 2);
            }else if p0 == new_point1 && p3 == new_point2 {
                optimized.remove(i + 2);
                optimized.remove(i + 1);                
            }            
            println!("Successfully convex and merged");
            return true;
        }else if new_point1 == new_point2 {
            println!("new_point1 == new_point2, no collision, but points are the same, skipping");
            continue; // no valid length found
        }
        let length_to_trace = |length: FixedPoint| {
            let new_point_on_line = p2 + line2_normal_dir.to_fixed_vec2(length);
            let mut new_line = line2.clone();
            new_line.point = new_point_on_line;
            let start_position = new_line.intersection(&line1);
            let end_position = new_line.intersection(&line3);
            vec![(start_position, end_position)]
        };
        let end_length = FixedPoint::min(relative_distance_1, relative_distance_2);
        let length = binary_approach_to_obstacles(
            &length_to_trace,
            FixedPoint::ZERO,
            end_length,
            check_collision_for_trace,
            trace_width,
            trace_clearance,
            my_layer,
        );
        if length == FixedPoint::ZERO {
            println!("Failed to binary approach to obstacles, length = 0");
            continue; // no valid length found
        }
        let mut new_parallel_line = line2.clone();
        let new_point_on_line = p2 + line2_normal_dir.to_fixed_vec2(length);
        new_parallel_line.point = new_point_on_line;
        let new_point1 = new_parallel_line.intersection(&line1);
        let new_point2 = new_parallel_line.intersection(&line3);
        optimized[i + 1].position = new_point1;
        optimized[i + 2].position = new_point2;
        println!("Successfully convex and merged");
        return true;
        // let dir4 = if i == optimized.len() - 4 {None} else {Some(Direction::from_points(p3, optimized[i + 4].position).unwrap())};
    }
    false
}

pub fn try_cut_right_or_sharp_angle(
    optimized: &mut Vec<TraceAnchor>,
    // check_collision_for_trace: &dyn Fn(FixedVec2, FixedVec2, f32, f32, usize) -> bool,
    // check_collision_for_via: &dyn Fn(FixedVec2, f32, f32, usize, usize) -> bool,
    // trace_width: f32,
    // trace_clearance: f32,
) -> bool{
    for i in 0..optimized.len() - 2 {
        let p1 = optimized[i].position;
        let anchor2 = &optimized[i + 1];
        let p2 = optimized[i + 1].position;
        let p3 = optimized[i + 2].position;
        assert!(optimized[i].end_layer == optimized[i + 1].start_layer, "End layer of anchor {} should match start layer of anchor {}", i, i + 1);
        if optimized[i + 1].start_layer != optimized[i + 1].end_layer
        {
            continue;
        }
        let dir1 = Direction::from_points(p1, p2).unwrap();
        let dir1 = match dir1 {
            Some(dir) => dir,
            None => continue, // not a valid direction
        };
        let dir2 = Direction::from_points(p2, p3).unwrap();
        let dir2 = match dir2 {
            Some(dir) => dir,
            None => continue, // not a valid direction
        };
        let my_layer = optimized[i].end_layer;

        if dir1.is_right_angle(dir2) || dir1.is_sharp_angle(dir2) {
            optimized.insert(i + 2, anchor2.clone());
            return true;
        }
    }
    false
}



pub fn try_merge_path(optimized: &mut Vec<TraceAnchor>)->bool{
    for i in 0..optimized.len() - 2 {
        // Check for inline segments that can be optimized
        let p1 = optimized[i].position;
        let p2 = optimized[i + 1].position;
        let p3 = optimized[i + 2].position;
        assert!(optimized[i].end_layer == optimized[i + 1].start_layer, "End layer of anchor {} should match start layer of anchor {}", i, i + 1);
        assert!(optimized[i + 1].end_layer == optimized[i + 2].start_layer, "End layer of anchor {} should match start layer of anchor {}", i + 1, i + 2);

        if optimized[i + 1].start_layer != optimized[i + 1].end_layer
        {
            continue;
        }
        let dir1 = Direction::from_points(p1, p2).unwrap();
        let dir2 = Direction::from_points(p2, p3).unwrap();

        // eliminate redundant anchors
        match (dir1, dir2) {
            (Some(d1), Some(d2))  => {
                if d1 == d2{
                    optimized.remove(i + 1); // Remove the second anchor
                    return true;
                }else{
                    continue;
                }                
            },
            _=>{
                optimized.remove(i + 1); // Remove the second anchor
                return true;
            }
        }
    }
    false
}

pub fn print_directions(optimized: &Vec<TraceAnchor>) {
    print!("Updated directions: ");
    for i in 0..optimized.len() - 1 {
        let p1 = optimized[i].position;
        let p2 = optimized[i + 1].position;
        let dir_str = match Direction::from_points(p1, p2).unwrap(){
            Some(dir) => format!("{:?}", dir),
            None => "None".to_string(),
        };
        print!("{}", dir_str);
    }
    println!();
}

pub fn optimize_path(
    trace_path: &TracePath,
    check_collision_for_trace: &dyn Fn(FixedVec2, FixedVec2, f32, f32, usize) -> bool,
    // check_collision_for_via: &dyn Fn(FixedVec2, f32, f32, usize, usize) -> bool, // min, max
    trace_width: f32,
    trace_clearance: f32,
    via_diameter: f32,
) -> TracePath {    
    let path = &trace_path.anchors.0;
    let mut optimized = path.clone();    
    loop{
        let success = try_merge_path(&mut optimized);
        if success{
            println!("Merged path successfully");
            print_directions(&optimized);
        }
        else{
            println!("Failed to merge path");
            break;
        }
        // return (TracePath::from_anchors(TraceAnchors(optimized), trace_width, trace_clearance, via_diameter), true);
    }
    loop{
        let mut has_success = false;        
        loop{
            let success = try_parallel_shift(&mut optimized,
                check_collision_for_trace,
                trace_width,
                trace_clearance,
            );
            if success{
                println!("Parallel shift successful");
                print_directions(&optimized);
                has_success = true;
            }
            else{
                println!("Failed to parallel shift");
                break;
            }
            // return (TracePath::from_anchors(TraceAnchors(optimized), trace_width, trace_clearance, via_diameter), true);
        }
        loop{
            let success = try_convex_and_merge(&mut optimized,
                check_collision_for_trace,
                trace_width,
                trace_clearance,
            );
            if success{
                println!("Convex and merge successful");
                print_directions(&optimized);
                has_success = true;
            }
            else{
                println!("Failed to convex and merge");
                break;
            }
            // return (TracePath::from_anchors(TraceAnchors(optimized), trace_width, trace_clearance, via_diameter), true);
        }
        loop{
            let success = try_cut_right_or_sharp_angle(&mut optimized);
            if success{
                println!("Cut right or sharp angle successful");
                let convex_success = try_convex_and_merge(&mut optimized, check_collision_for_trace, trace_width, trace_clearance);
                println!("Tried to convex and merge after cutting right or sharp angle");
                print_directions(&optimized);
                if !convex_success{
                    continue;
                }
                has_success = true;
            }
            else{
                println!("Failed to cut right or sharp angle");
                break;
            }        
            //  return (TracePath::from_anchors(TraceAnchors(optimized), trace_width, trace_clearance, via_diameter), true);   
        }    
        if !has_success {
            println!("No more optimizations possible, breaking the outer loop");
            break; // no more optimizations possible
        }
    }
    // loop{
    //     let success = try_parallel_shift(&mut optimized,
    //         check_collision_for_trace,
    //         trace_width,
    //         trace_clearance,
    //     );
    //     if success{
    //         println!("Parallel shift successful");
    //         print_directions(&optimized);
    //         has_success = true;
    //     }
    //     else{
    //         println!("Failed to parallel shift");
    //         break;
    //     }
    //     // return (TracePath::from_anchors(TraceAnchors(optimized), trace_width, trace_clearance, via_diameter), true);
    // }
    loop{
        let success = try_merge_path(&mut optimized);
        if success{
            println!("Merged path successfully");
            print_directions(&optimized);
            // has_success = true;
        }
        else{
            println!("Failed to merge path");
            break;
        }
        // return (TracePath::from_anchors(TraceAnchors(optimized), trace_width, trace_clearance, via_diameter), true);
    }
    let result_trace_anchors = TraceAnchors(optimized);
    let result_trace_path = TracePath::from_anchors(result_trace_anchors, trace_width, trace_clearance, via_diameter);
    result_trace_path
}
