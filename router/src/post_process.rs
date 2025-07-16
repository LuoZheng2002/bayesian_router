use crate::block_or_sleep::{block_or_sleep, block_thread};
use shared::{
    binary_heap_item::BinaryHeapItem,
    hyperparameters::{ASTAR_STRIDE, DISPLAY_ASTAR, ESTIMATE_COEFFICIENT},
    pcb_render_model::{PcbRenderModel, RenderableBatch, ShapeRenderable, UpdatePcbRenderModel},
    prim_shape::{CircleShape, PrimShape, RectangleShape},
    trace_path::{Direction, TraceAnchors, TracePath, TraceSegment},
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
        .map(|p| (f64::from(p.x), f64::from(p.y)))
        .collect();

    chart.draw_series(
        anchor_points
            .iter()
            .map(|(x, y)| Circle::new((*x, *y), 5, RED.filled())),
    )?;

    // 添加长度标注
    root.draw(&Text::new(
        format!("Total Length: {:.2}", trace_path.length),
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
        min_x = min_x.min(f64::from(anchor.x));
        max_x = max_x.max(f64::from(anchor.x));
        min_y = min_y.min(f64::from(anchor.y));
        max_y = max_y.max(f64::from(anchor.y));
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

fn rebuild_segments(anchors: &Vec<FixedVec2>, width: f32, clearance: f32) -> Vec<TraceSegment> {
    let mut segments = Vec::new();
    for i in 0..anchors.len() - 1 {
        let start = anchors[i];
        let end = anchors[i + 1];
        assert_ne!(start, end, "Start and end positions should not be the same");
        let segment = TraceSegment {
            start,
            end,
            width,
            clearance,
        };
        segments.push(segment);
    }
    segments
}

pub fn optimize_path(
    trace_path: &TracePath,
    check_collision: &dyn Fn(FixedVec2, FixedVec2, f32, f32) -> bool,
    trace_width: f32,
    trace_clearance: f32,
) -> TracePath {
    // display input trace path and optimized trace path on the PCB render model
    if DISPLAY_ASTAR {
        draw_tracepath_to_file(trace_path, Path::new("input_trace_path.png"), 800, 600)
            .expect("Failed to draw input trace path");
        println!("input trace path drawn to input_trace_path.png");
        block_thread();
    }

    let path = &trace_path.anchors.0;
    let mut length: f64 = trace_path.length;
    if path.len() < 4 {
        return trace_path.clone();
    }

    let mut success = false;
    let mut optimized = path.clone();
    let mut i = 0;
    while i < optimized.len() - 3 {
        // trace shifting
        let seg1 = (&optimized[i], &optimized[i + 1]);
        let seg2 = (&optimized[i + 2], &optimized[i + 3]);

        let dx1 = seg1.1.x - seg1.0.x;
        let dy1 = seg1.1.y - seg1.0.y;
        let dx2 = seg2.1.x - seg2.0.x;
        let dy2 = seg2.1.y - seg2.0.y;

        if dx1 * dy2 == dx2 * dy1 {
            // debug
            if DISPLAY_ASTAR {
                println!(
                    "Optimizing segments {}-{} and {}-{} due to parallelism",
                    i,
                    i + 1,
                    i + 2,
                    i + 3
                );
            }

            let new_point1 = FixedVec2 {
                x: seg1.0.x + seg2.0.x - seg1.1.x,
                y: seg1.0.y + seg2.0.y - seg1.1.y,
            };
            let new_point2 = FixedVec2 {
                x: seg2.1.x - seg2.0.x + seg1.1.x,
                y: seg2.1.y - seg2.0.y + seg1.1.y,
            };

            let flag1 = !check_collision(optimized[i], new_point1, trace_width, trace_clearance)
                && !check_collision(new_point1, optimized[i + 2], trace_width, trace_clearance);
            let flag2 =
                !check_collision(optimized[i + 1], new_point2, trace_width, trace_clearance)
                    && !check_collision(new_point2, optimized[i + 3], trace_width, trace_clearance);

            if flag1 {
                assert_ne!((new_point1.x - new_point1.y) % 2, 0);
                optimized[i + 1] = new_point1;
                optimized.remove(i + 2);
                success = true;
            } else if flag2 {
                optimized[i + 2] = new_point2;
                optimized.remove(i + 1);
                success = true;
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

    if DISPLAY_ASTAR {
        draw_tracepath_to_file(
            &TracePath {
                anchors: TraceAnchors(optimized.clone()),
                segments: rebuild_segments(&optimized, trace_width, trace_clearance),
                length,
            },
            Path::new("optimized_trace_path.png"),
            800,
            600,
        )
        .expect("Failed to draw optimized trace path");
        println!("optimized trace path drawn to optimized_trace_path.png");
        block_thread();
    }

    // i = 1;
    // while i < optimized.len() - 2 {
    //     // tight wrapping
    //     let p0 = optimized[i - 1];
    //     let p1 = optimized[i];
    //     let p2 = optimized[i + 1];
    //     let p3 = optimized[i + 2];

    //     let d01 = (p1.x - p0.x, p1.y - p0.y);
    //     let d12 = (p2.x - p1.x, p2.y - p1.y);
    //     let d23 = (p3.x - p2.x, p3.y - p2.y);

    //     if (is_axis(d01) && is_diagonal(d12) && is_axis(d23) && (d01.0 * d23.0 == FixedPoint::ZERO)) ||  // vertical-diagonal-horizontal or horizontal-diagonal-vertical
    //        (is_diagonal(d01) && is_axis(d12) && is_diagonal(d23) && ((d01.0.signum() != d23.0.signum()) || (d01.1.signum() != d23.1.signum())))
    //     {
    //         // diagonal-axis-diagonal with different directions {

    //         let len_d01 = FixedPoint::max(d01.0.abs(), d01.1.abs());
    //         let len_d23 = FixedPoint::max(d23.0.abs(), d23.1.abs());
    //         let max_length = FixedPoint::min(len_d01, len_d23);
    //         let num_steps = (max_length / FixedPoint::DELTA).ceil().to_num();

    //         for step_idx in 0..=num_steps {
    //             let step = FixedPoint::from_num(step_idx) * FixedPoint::DELTA;
    //             let step = FixedPoint::min(step, max_length);

    //             let new_point1 = FixedVec2 {
    //                 x: p1.x - d01.0 * (max_length - step) / len_d01,
    //                 y: p1.y - d01.1 * (max_length - step) / len_d01,
    //             };
    //             let new_point2 = FixedVec2 {
    //                 x: p2.x + d23.0 * (max_length - step) / len_d23,
    //                 y: p2.y + d23.1 * (max_length - step) / len_d23,
    //             };

    //             if !check_collision(p0, new_point1, trace_width, trace_clearance)
    //                 && !check_collision(new_point1, new_point2, trace_width, trace_clearance)
    //                 && !check_collision(new_point2, p3, trace_width, trace_clearance)
    //             {
    //                 length = length
    //                     - ((p1 - p0).length().to_num::<f64>()
    //                         + (p3 - p2).length().to_num::<f64>()
    //                         + (p2 - p1).length().to_num::<f64>())
    //                     + (new_point1 - p0).length().to_num::<f64>()
    //                     + (p3 - new_point2).length().to_num::<f64>()
    //                     + (new_point2 - new_point1).length().to_num::<f64>();
    //                 optimized[i] = new_point1;
    //                 optimized[i + 1] = new_point2;
    //                 break;
    //             }
    //         }
    //     }
    //     i += 1;
    // }
    let segments = rebuild_segments(&optimized, trace_width, trace_clearance);
    TracePath {
        anchors: TraceAnchors(optimized),
        segments,
        length,
    }
}
