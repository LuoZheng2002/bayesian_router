use fixed::traits::Fixed;
// use crate::block_or_sleep::{block_or_sleep, block_thread};
use shared::{
    binary_heap_item::BinaryHeapItem,
    hyperparameters::{ASTAR_STRIDE, DISPLAY_OPTIMIZATION},
    pcb_render_model::{PcbRenderModel, RenderableBatch, ShapeRenderable, UpdatePcbRenderModel},
    prim_shape::{CircleShape, PrimShape, RectangleShape},
    trace_path::{Direction, TraceAnchor, TraceAnchors, TracePath, TraceSegment, Via},
    vec2::{FixedPoint, FixedVec2, FloatVec2},
};

use plotters::prelude::*;
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

fn is_right_angle(dir1: Direction, dir2: Direction) -> bool {
    let angle = (dir1.to_degree_angle() - dir2.to_degree_angle()).abs();
    angle == 90.0 || angle == 270.0
}

fn is_convex(dir1: Direction, dir2: Direction, dir3: Direction) -> bool {
    let angle1 = (dir1.to_degree_angle() - dir3.to_degree_angle()).abs();
    let angle2 = (dir1.to_degree_angle() + dir3.to_degree_angle()).abs() / 2.0;
    (angle1 == 90.0 || angle1 == 270.0) && angle2 == dir2.to_degree_angle()
}

fn anchor_to_tracepath(
    anchors: Vec<TraceAnchor>,
    width: f32,
    clearance: f32,
    total_length: f64,
    via_diameter: f32,
) -> TracePath {
    let mut segments: Vec<TraceSegment> = Vec::new();
    let mut vias: Vec<Via> = Vec::new(); // initializes with the end position
    for i in 0..anchors.len() - 1 {
        let start_anchor = &anchors[i];
        let end_anchor = &anchors[i + 1];
        assert!(
            start_anchor.end_layer == end_anchor.start_layer,
            "The end layer of the start anchor should match the start layer of the end anchor"
        );
        assert_ne!(
            start_anchor.position, end_anchor.position,
            "Start and end positions should not be the same"
        );
        let segment = TraceSegment {
            start: start_anchor.position,
            end: end_anchor.position,
            layer: start_anchor.end_layer,
            width,
            clearance,
        };
        segments.push(segment);
        if start_anchor.start_layer != start_anchor.end_layer {
            // if the start and end layers are different, we need to add a via
            let via = Via {
                position: start_anchor.position,
                clearance,
                diameter: via_diameter,
                min_layer: usize::min(start_anchor.start_layer, start_anchor.end_layer),
                max_layer: usize::max(start_anchor.start_layer, start_anchor.end_layer),
            };
            vias.push(via);
        }
    }
    let anchors_new = TraceAnchors(anchors);
    TracePath {
        anchors: anchors_new,
        segments,
        vias,
        total_length,
    }
}

pub fn optimize_path(
    trace_path: &TracePath,
    check_collision: &dyn Fn(FixedVec2, FixedVec2, f32, f32, usize) -> bool,
    trace_width: f32,
    trace_clearance: f32,
    via_diameter: f32,
) -> TracePath {
    // display input trace path and optimized trace path on the PCB render model
    if DISPLAY_OPTIMIZATION {
        draw_tracepath_to_file(trace_path, Path::new("input_trace_path.png"), 800, 600)
            .expect("Failed to draw input trace path");
        println!("input trace path drawn to input_trace_path.png");
        // block_thread();
    }

    let path = &trace_path.anchors.0;
    if path.len() < 4 {
        return trace_path.clone();
    }
    let mut optflag = true;
    let mut optimized = path.clone();
    let mut total_length = trace_path.total_length;

    while optflag {
        optflag = false;
        let mut success = false;
        let mut i = 0;
        while i < optimized.len() - 2 {
            // Check for inline segments that can be optimized
            let p1 = optimized[i].position;
            let p2 = optimized[i + 1].position;
            let p3 = optimized[i + 2].position;

            if optimized[i].end_layer == optimized[i + 1].start_layer
                && optimized[i + 1].start_layer == optimized[i + 1].end_layer
                && optimized[i + 1].end_layer == optimized[i + 2].start_layer
            {
                let my_layer = optimized[i].end_layer;
                let dir1 = Direction::from_points(p1, p2).unwrap();
                let dir2 = Direction::from_points(p2, p3).unwrap();

                // eliminate redundant anchors
                if dir1 == dir2 {
                    optimized.remove(i + 1);
                    success = true;
                    optflag = true;
                }
                // convert right angle
                else if is_right_angle(dir1, dir2) {
                    if FixedPoint::max((p3 - p2).x.abs(), (p3 - p2).y.abs()) > FixedPoint::DELTA
                        && FixedPoint::max((p1 - p2).x.abs(), (p1 - p2).y.abs()) > FixedPoint::DELTA
                    {
                        let new_position1 = p2 - dir1.to_fixed_vec2(FixedPoint::DELTA);
                        let new_position2 = p2 + dir2.to_fixed_vec2(FixedPoint::DELTA);
                        assert!(
                            Direction::is_two_points_valid_direction(new_position1, new_position2),
                            "New positions should form a valid direction, but got {:?} and {:?}",
                            new_position1,
                            new_position2
                        );
                        if !check_collision(
                            new_position1,
                            new_position2,
                            trace_width,
                            trace_clearance,
                            my_layer,
                        ) {
                            total_length = total_length
                                - ((p2 - p1).length().to_num::<f64>()
                                    + (p3 - p2).length().to_num::<f64>())
                                + (new_position1 - p1).length().to_num::<f64>()
                                + (p3 - new_position2).length().to_num::<f64>()
                                + (new_position2 - new_position1).length().to_num::<f64>();
                            if p1 == new_position1 && p3 == new_position2 {
                                optimized.remove(i + 1);
                                i -= 1;
                            } else if p3 == new_position2 {
                                optimized[i + 1].position = new_position1;
                            } else if p1 == new_position1 {
                                optimized[i + 1].position = new_position2;
                            } else {
                                let new_anchor1 = TraceAnchor {
                                    position: new_position1,
                                    start_layer: my_layer,
                                    end_layer: my_layer,
                                };
                                optimized.insert(i + 1, new_anchor1);
                                optimized[i + 2].position = new_position2;
                            }
                            optflag = true;
                        }
                    }else{
                        let step = FixedPoint::min(FixedPoint::max((p3 - p2).x.abs(), (p3 - p2).y.abs()), FixedPoint::max((p1 - p2).x.abs(), (p1 - p2).y.abs()));
                        let new_position1 = p2 - dir1.to_fixed_vec2(step);
                        let new_position2 = p2 + dir2.to_fixed_vec2(step);
                        assert!(
                            Direction::is_two_points_valid_direction(new_position1, new_position2),
                            "New positions should form a valid direction, but got {:?} and {:?}",
                            new_position1,
                            new_position2
                        );
                        if !check_collision(
                            new_position1,
                            new_position2,
                            trace_width,
                            trace_clearance,
                            my_layer,
                        ){
                            if p1 == new_position1 && p2 == new_position2 {
                                optimized.remove(i + 1);
                                i -= 1;
                            } else if p1 == new_position1 {
                                optimized[i + 1].position = new_position2;
                            } else {
                                optimized[i + 1].position = new_position1;
                            }
                            optflag = true;
                        }
                    }
                }
            }
            i += 1;
            if i >= optimized.len() - 2 {
                if success {
                    i = 0; // restart from the beginning if any optimization was made
                    success = false;
                }
            }
        }

        i = 0;
        success = false;
        while i < optimized.len() - 3 {
            // Check for parallel segments that can be optimized
            // trace shifting
            let p0 = optimized[i].position;
            let p1 = optimized[i + 1].position;
            let p2 = optimized[i + 2].position;
            let p3 = optimized[i + 3].position;

            if optimized[i].end_layer == optimized[i + 1].start_layer
                && optimized[i + 1].start_layer == optimized[i + 1].end_layer
                && optimized[i + 1].end_layer == optimized[i + 2].start_layer
                && optimized[i + 2].start_layer == optimized[i + 2].end_layer
                && optimized[i + 2].end_layer == optimized[i + 3].start_layer
            {
                let my_layer = optimized[i].end_layer;
                let dir0 =  if i == 0 {None} else {Some(Direction::from_points(optimized[i - 1].position, p0).unwrap())};
                let dir1 = Direction::from_points(p0, p1).unwrap();
                let dir2 = Direction::from_points(p1, p2).unwrap();
                let dir3 = Direction::from_points(p2, p3).unwrap();
                let dir4 = if i == optimized.len() - 4 {None} else {Some(Direction::from_points(p3, optimized[i + 4].position).unwrap())};

                if dir1 == dir3 && (dir0 == None || !is_convex(dir0.unwrap(), dir1, dir2)) && (dir4 == None || !is_convex(dir2, dir3, dir4.unwrap())){
                    // debug
                    if DISPLAY_OPTIMIZATION {
                        println!(
                            "Optimizing segments {}-{} and {}-{} due to parallelism",
                            i,
                            i + 1,
                            i + 2,
                            i + 3
                        );
                    }
                    let new_point1 = FixedVec2 {
                        x: p0.x + p2.x - p1.x,
                        y: p0.y + p2.y - p1.y,
                    };
                    let new_point2 = FixedVec2 {
                        x: p3.x - p2.x + p1.x,
                        y: p3.y - p2.y + p1.y,
                    };

                    let flag1 =
                        !check_collision(p0, new_point1, trace_width, trace_clearance, my_layer)
                            && !check_collision(new_point1, p2, trace_width, trace_clearance, my_layer);
                    let flag2 =
                        !check_collision(p1, new_point2, trace_width, trace_clearance, my_layer)
                            && !check_collision(new_point2, p3, trace_width, trace_clearance, my_layer);

                    if flag1 {
                        assert!(
                            Direction::is_two_points_valid_direction(new_point1, p2),
                            "New positions should form a valid direction"
                        );
                        assert!(
                            Direction::is_two_points_valid_direction(p0, new_point1),
                            "New positions should form a valid direction"
                        );
                        optimized[i + 1].position = new_point1;
                        optimized.remove(i + 2);
                        success = true;
                        optflag = true;
                    } else if flag2 {
                        assert!(
                            Direction::is_two_points_valid_direction(new_point2, p3),
                            "New positions should form a valid direction"
                        );
                        assert!(
                            Direction::is_two_points_valid_direction(p1, new_point2),
                            "New positions should form a valid direction"
                        );
                        optimized[i + 2].position = new_point2;
                        optimized.remove(i + 1);
                        success = true;
                        optflag = true;
                    }
                }
            }
            i += 1;
            if i >= optimized.len() - 3 {
                if success {
                    i = 0; // restart from the beginning if any optimization was made
                    success = false;
                }
            }
        }


        i = 1;
        while i < optimized.len() - 2 {
            // tight wrapping
            let p0 = optimized[i - 1].position;
            let p1 = optimized[i].position;
            let p2 = optimized[i + 1].position;
            let p3 = optimized[i + 2].position;

            if optimized[i - 1].end_layer == optimized[i].start_layer
                && optimized[i].start_layer == optimized[i].end_layer
                && optimized[i].end_layer == optimized[i + 1].start_layer
                && optimized[i + 1].start_layer == optimized[i + 1].end_layer
                && optimized[i + 1].end_layer == optimized[i + 2].start_layer
            {
                let my_layer = optimized[i - 1].end_layer;
                let dir1 = Direction::from_points(p0, p1).unwrap();
                let dir2 = Direction::from_points(p1, p2).unwrap();
                let dir3 = Direction::from_points(p2, p3).unwrap();

                if is_convex(dir1, dir2, dir3) {
                    let len1 = FixedPoint::max((p1 - p0).x.abs(), (p1 - p0).y.abs());
                    let len3 = FixedPoint::max((p3 - p2).x.abs(), (p3 - p2).y.abs());
                    let max_len =
                        FixedPoint::min(len1, len3);
                    let num_steps = (max_len / FixedPoint::DELTA / FixedPoint::from_num(2.0))
                        .ceil()
                        .to_num::<usize>();
                    if DISPLAY_OPTIMIZATION {
                        println!(
                            "Enter is_convex: {:?} -> {:?} -> {:?}, max_len: {}, num_steps: {}",
                            (p0, p1),
                            (p1, p2),
                            (p2, p3),
                            max_len,
                            num_steps
                        );
                    }
                    for step_idx in 0..=num_steps {
                        let step = FixedPoint::from_num(step_idx)
                            * FixedPoint::DELTA
                            * FixedPoint::from_num(2.0);
                        let step = FixedPoint::min(step, max_len);
                        let new_point1 = p1 - dir1.to_fixed_vec2(max_len - step);
                        let new_point2 = p2 + dir3.to_fixed_vec2(max_len - step);
                        if step == max_len {break;}
                        assert!(
                            Direction::is_two_points_valid_direction(new_point1, new_point2),
                            "New positions should form a valid direction"
                        );

                        if (p0 == new_point1
                            || !check_collision(p0, new_point1, trace_width, trace_clearance, my_layer))
                            && !check_collision(
                                new_point1,
                                new_point2,
                                trace_width,
                                trace_clearance,
                                my_layer,
                            )
                            && (new_point2 == p3
                                || !check_collision(
                                    new_point2,
                                    p3,
                                    trace_width,
                                    trace_clearance,
                                    my_layer,
                                ))
                        {
                            total_length = total_length
                                - ((p1 - p0).length().to_num::<f64>()
                                    + (p3 - p2).length().to_num::<f64>()
                                    + (p2 - p1).length().to_num::<f64>())
                                + (new_point1 - p0).length().to_num::<f64>()
                                + (p3 - new_point2).length().to_num::<f64>()
                                + (new_point2 - new_point1).length().to_num::<f64>();
                            optimized[i].position = new_point1;
                            optimized[i + 1].position = new_point2;
                            if new_point2 == p3 {
                                // If we reached the maximum length, we can remove the redundant points
                                optimized.remove(i + 2);
                                i -= 1;
                            }
                            if new_point1 == p0 {
                                optimized.remove(i - 1);
                                i -= 1;
                            }
                            if DISPLAY_OPTIMIZATION {
                                println!(
                                    "Optimized points: {:?} -> {:?}",
                                    (p1, p2),
                                    (new_point1, new_point2)
                                );
                            }
                            optflag = true;
                            break;
                        }
                    }
                }
            }
            i += 1;
        }


        // i = 0;
        // success = false;
        // while i < optimized.len() - 2 {
        //     // Check for inline segments that can be optimized
        //     let p1 = optimized[i].position;
        //     let p2 = optimized[i + 1].position;
        //     let p3 = optimized[i + 2].position;

        //     if optimized[i].end_layer == optimized[i + 1].start_layer
        //         && optimized[i + 1].start_layer == optimized[i + 1].end_layer
        //         && optimized[i + 1].end_layer == optimized[i + 2].start_layer
        //     {
        //         let dir1 = Direction::from_points(p1, p2).unwrap();
        //         let dir2 = Direction::from_points(p2, p3).unwrap();

        //         // eliminate redundant anchors
        //         if dir1 == dir2 {
        //             optimized.remove(i + 1);
        //             success = true;
        //         }
        //     }
        //     i += 1;
        //     if i >= optimized.len() - 2 {
        //         if success {
        //             i = 0; // restart from the beginning if any optimization was made
        //             success = false;
        //             println!()
        //         }
        //     }
        // }
    }
    

    let return_trace = anchor_to_tracepath(
        optimized,
        trace_width,
        trace_clearance,
        total_length,
        via_diameter,
    );
    if DISPLAY_OPTIMIZATION {
        draw_tracepath_to_file(
            &return_trace,
            Path::new("optimized_trace_path.png"),
            800,
            600,
        )
        .expect("Failed to draw optimized trace path");
        println!("optimized trace path drawn to optimized_trace_path.png");
        // block_thread();
    }

    return_trace
}
