use crate::block_or_sleep::{block_or_sleep, block_thread};
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
use std::path::Path; // 引入Range类型

pub fn draw_tracepath_to_file(
    trace_path: &TracePath,
    output_path: &Path,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建绘图区域
    let root = BitMapBackend::new(output_path, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    // 计算坐标范围（自动适应或手动指定）
    let (x_range, y_range) = calculate_plot_range(trace_path);

    // 创建图表上下文
    let mut chart = ChartBuilder::on(&root)
        .caption("Trace Path Visualization", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_range, y_range)?;

    // 绘制坐标网格
    chart.configure_mesh().draw()?;

    // 绘制路径线段
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

    // 绘制锚点（转折点）
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

    // 添加长度标注
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

    // 添加10%的边距
    let x_margin = (max_x - min_x) * 0.1;
    let y_margin = (max_y - min_y) * 0.1;

    (
        (min_x - x_margin)..(max_x + x_margin),
        (min_y - y_margin)..(max_y + y_margin),
    )
}

fn is_axis(d: (FixedPoint, FixedPoint)) -> bool {
    (d.0 == 0.0 && d.1 != 0.0) || (d.0 != 0.0 && d.1 == 0.0)
}

fn is_diagonal(d: (FixedPoint, FixedPoint)) -> bool {
    d.0 != 0.0 && d.1 != 0.0 && d.0.abs() == d.1.abs()
}

fn is_convex(
    d01: (FixedPoint, FixedPoint),
    d12: (FixedPoint, FixedPoint),
    d23: (FixedPoint, FixedPoint),
) -> bool {
    (is_axis(d01)
        && is_diagonal(d12)
        && is_axis(d23)
        && (d01.0 * d23.0 == FixedPoint::ZERO && d01.1 * d23.1 == FixedPoint::ZERO))
    //|| (is_diagonal(d01) && is_axis(d12) && is_diagonal(d23) && (d01.0.signum() != d23.0.signum() || d01.1.signum() != d23.1.signum()))
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
        block_thread();
    }

    let path = &trace_path.anchors.0;
    let mut length: f64 = trace_path.total_length;
    if path.len() < 4 {
        return trace_path.clone();
    }

    let mut success = false;
    let mut optimized = path.clone();
    let mut i = 0;
    let total_length = trace_path.total_length;
    while i < optimized.len() - 2 {
        // Check for inline segments that can be optimized
        let seg1 = (&optimized[i], &optimized[i + 1]);
        let seg2 = (&optimized[i + 1], &optimized[i + 2]);

        if seg1.0.end_layer == seg1.1.start_layer
            && seg1.1.start_layer == seg1.1.end_layer
            && seg1.1.end_layer == seg2.1.start_layer
        {
            let dx1 = seg1.1.position.x - seg1.0.position.x;
            let dy1 = seg1.1.position.y - seg1.0.position.y;
            let dx2 = seg2.1.position.x - seg2.0.position.x;
            let dy2 = seg2.1.position.y - seg2.0.position.y;
            if dx1 * dy2 == dx2 * dy1 {
                optimized.remove(i + 1);
                success = true;
            }
            i += 1;
            if i >= optimized.len() - 2 {
                if success {
                    i = 0; // restart from the beginning if any optimization was made
                    success = false;
                }
            }
        }
    }

    // i = 1;
    // while i < optimized.len() - 2 {
    //     // tight wrapping
    //     let p0 = optimized[i - 1];
    //     let p1 = optimized[i];
    //     let p2 = optimized[i + 1];
    //     let p3 = optimized[i + 2];

    //     let d01 = (p1.position.x - p0.position.x, p1.position.y - p0.position.y);
    //     let d12 = (p2.position.x - p1.position.x, p2.position.y - p1.position.y);
    //     let d23 = (p3.position.x - p2.position.x, p3.position.y - p2.position.y);

    //     if is_convex(d01, d12, d23) {
    //         // vertical-diagonal-horizontal or horizontal-diagonal-vertical
    //         // diagonal-axis-diagonal with different directions {

    //         let len_d01 = FixedPoint::max(d01.0.abs(), d01.1.abs());
    //         let len_d23 = FixedPoint::max(d23.0.abs(), d23.1.abs());
    //         let max_length = FixedPoint::min(len_d01, len_d23);
    //         let num_steps = (max_length / FixedPoint::DELTA / FixedPoint::from_num(2.0))
    //             .ceil()
    //             .to_num();
    //         if DISPLAY_OPTIMIZATION {
    //             println!(
    //                 "Enter is_convex: {:?} -> {:?} -> {:?}, max_length: {}, num_steps: {}",
    //                 (p0, p1),
    //                 (p1, p2),
    //                 (p2, p3),
    //                 max_length,
    //                 num_steps
    //             );
    //         }
    //         for step_idx in 0..=num_steps {
    //             let step =
    //                 FixedPoint::from_num(step_idx) * FixedPoint::DELTA * FixedPoint::from_num(2.0);
    //             let step = FixedPoint::min(step, max_length);
    //             let new_point1 = FixedVec2 {
    //                 x: p1.position.x - d01.0 / len_d01 * (max_length - step),
    //                 y: p1.position.y - d01.1 / len_d01 * (max_length - step),
    //             };
    //             let new_point2 = FixedVec2 {
    //                 x: p2.position.x + d23.0 / len_d23 * (max_length - step),
    //                 y: p2.position.y + d23.1 / len_d23 * (max_length - step),
    //             };
    //             if DISPLAY_OPTIMIZATION {
    //                 println!(
    //                     "Temp optimized points: {:?} -> {:?}",
    //                     (p1, p2),
    //                     (new_point1, new_point2)
    //                 );
    //             }

    //             if (p0.position == new_point1
    //                 || !check_collision(p0, new_point1, trace_width, trace_clearance))
    //                 && !check_collision(new_point1, new_point2, trace_width, trace_clearance)
    //                 && (new_point2 == p3.position
    //                     || !check_collision(new_point2, p3, trace_width, trace_clearance))
    //             {
    //                 length = length
    //                     - ((p1.position - p0.position).length().to_num::<f64>()
    //                         + (p3.position - p2.position).length().to_num::<f64>()
    //                         + (p2.position - p1.position).length().to_num::<f64>())
    //                     + (new_point1 - p0.position).length().to_num::<f64>()
    //                     + (p3.position - new_point2).length().to_num::<f64>()
    //                     + (new_point2 - new_point1).length().to_num::<f64>();
    //                 optimized[i].position = new_point1;
    //                 optimized[i + 1].position = new_point2;
    //                 if new_point2 == p3.position {
    //                     // If we reached the maximum length, we can remove the redundant points
    //                     optimized.remove(i + 2);
    //                     i -= 1;
    //                 }
    //                 if new_point1 == p0.position {
    //                     optimized.remove(i - 1);
    //                     i -= 1;
    //                 }
    //                 if DISPLAY_OPTIMIZATION {
    //                     println!(
    //                         "Optimized points: {:?} -> {:?}",
    //                         (p1, p2),
    //                         (new_point1, new_point2)
    //                     );
    //                 }
    //                 break;
    //             }
    //         }
    //     }
    //     i += 1;
    // }

    // i = 0;
    // while i < optimized.len() - 3 {
    //     // Check for parallel segments that can be optimized
    //     // trace shifting
    //     let seg1 = (&optimized[i], &optimized[i + 1]);
    //     let seg2 = (&optimized[i + 2], &optimized[i + 3]);

    //     let dx1 = seg1.1.x - seg1.0.x;
    //     let dy1 = seg1.1.y - seg1.0.y;
    //     let dx2 = seg2.1.x - seg2.0.x;
    //     let dy2 = seg2.1.y - seg2.0.y;

    //     if dx1 * dy2 == dx2 * dy1 {
    //         // debug
    //         if DISPLAY_OPTIMIZATION {
    //             println!(
    //                 "Optimizing segments {}-{} and {}-{} due to parallelism",
    //                 i,
    //                 i + 1,
    //                 i + 2,
    //                 i + 3
    //             );
    //         }

    //         let new_point1 = FixedVec2 {
    //             x: seg1.0.x + seg2.0.x - seg1.1.x,
    //             y: seg1.0.y + seg2.0.y - seg1.1.y,
    //         };
    //         let new_point2 = FixedVec2 {
    //             x: seg2.1.x - seg2.0.x + seg1.1.x,
    //             y: seg2.1.y - seg2.0.y + seg1.1.y,
    //         };

    //         let flag1 = !check_collision(optimized[i], new_point1, trace_width, trace_clearance)
    //             && !check_collision(new_point1, optimized[i + 2], trace_width, trace_clearance);
    //         let flag2 =
    //             !check_collision(optimized[i + 1], new_point2, trace_width, trace_clearance)
    //                 && !check_collision(new_point2, optimized[i + 3], trace_width, trace_clearance);

    //         if flag1 {
    //             optimized[i + 1] = new_point1;
    //             optimized.remove(i + 2);
    //             success = true;
    //         } else if flag2 {
    //             optimized[i + 2] = new_point2;
    //             optimized.remove(i + 1);
    //             success = true;
    //         }
    //     }
    //     i += 1;
    //     if i >= optimized.len() - 3 {
    //         if success {
    //             i = 0; // restart from the beginning if any optimization was made
    //             success = false;
    //         }
    //     }
    // }

    // success = false;
    // i = 0;
    // while i < optimized.len() - 2 {
    //     // Check for inline segments that can be optimized
    //     let seg1 = (&optimized[i], &optimized[i + 1]);
    //     let seg2 = (&optimized[i + 1], &optimized[i + 2]);
    //     let dx1 = seg1.1.x - seg1.0.x;
    //     let dy1 = seg1.1.y - seg1.0.y;
    //     let dx2 = seg2.1.x - seg2.0.x;
    //     let dy2 = seg2.1.y - seg2.0.y;
    //     if dx1 * dy2 == dx2 * dy1 {
    //         optimized.remove(i + 1);
    //         success = true;
    //     }
    //     i += 1;
    //     if i >= optimized.len() - 2 {
    //         if success {
    //             i = 0; // restart from the beginning if any optimization was made
    //             success = false;
    //         }
    //     }
    // }

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
        block_thread();
    }

    return_trace
}
