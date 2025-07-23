use crate::dsn_struct::{
    Boundary, Component, ComponentInst, DsnStruct, Netclass, Network, PadStack, Pin, Pin2,
    Placement, PlacementLayer, Shape,
};
use crate::parse_to_display_format::{DisplayFormat, DisplayNetInfo, ExtraInfo};

use cgmath::{Deg, Matrix2, Rad, Vector2};
use core::{f32, net};
use shared::collider::PolygonCollider;
use shared::pad::{Pad, PadLayer, PadName, PadShape};
use shared::pcb_problem::{NetClassName, NetName};
use shared::prim_shape::Line;
use shared::vec2::{FixedVec2, FloatVec2};
use std::collections::HashMap;

fn calculate_boundary_and_scale(
    boundary: &Boundary,
    scale_down_factor: f32,
) -> Result<(f32, f32, FloatVec2), String> {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    // for (x, y) in &boundary.0 {
    //     min_x = min_x.min(*x);
    //     max_x = max_x.max(*x);
    //     min_y = min_y.min(*y);
    //     max_y = max_y.max(*y);
    // }
    for point in &boundary.0 {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }

    let width = max_x - min_x;
    let height = max_y - min_y;
    let center = FloatVec2 {
        x: (min_x + max_x) / 2.0,
        y: (min_y + max_y) / 2.0,
    };

    Ok((
        width / scale_down_factor,
        height / scale_down_factor,
        center / scale_down_factor,
    ))
}

/*
fn buildpadmap(
    library: &Library,
    placement: &Placement,
) -> Result<HashMap<(String, usize), Pad>, String> {
    // This function builds a map of pads from the library.
    let mut pad_map: HashMap<(String, usize), Pad> = HashMap::new();
    for (image_name, image) in &library.images {
        for (pin_number, pin) in &image.pins {
            let pad_stack = library.pad_stacks.get(&pin.pad_stack_name).ok_or_else(|| {
                format!(
                    "Pad stack '{}' not found for {}-{}",
                    pin.pad_stack_name, image_name, pin_number
                )
            })?;

            let shape = match &pad_stack.shape {
                Shape::Circle { diameter } => PadShape::Circle {
                    diameter: *diameter as f32,
                },
                Shape::Rect {
                    x_min,
                    y_min,
                    x_max,
                    y_max,
                } => PadShape::Rectangle {
                    width: (*x_max - *x_min) as f32,
                    height: (*y_max - *y_min) as f32,
                },
                Shape::Polygon {
                    aperture_width,
                    vertices,
                } => PadShape::RoundRect {
                    width: *aperture_width as f32,
                    height: *aperture_width as f32, // Assuming square for simplicity
                    corner_radius: 0.0,             // Not specified in the original code
                },
            };
        }
    }
    todo!("Implement padmap building from Net");
}
*/

#[derive(Debug, Clone)]
pub struct TransformedPad {
    pub component_name: String, // 如 "J1"
    pub pin_number: String,
    pub position: FloatVec2, // 最终PCB坐标系下的位置
    pub shape: PadShape,
    pub rotation: cgmath::Deg<f32>, // 最终旋转角度（度）
    pub pad_layer: PadLayer,        // Pad所在的层
}

fn transform_point(point: FloatVec2, rotation_deg: f32, translation: FloatVec2) -> FloatVec2 {
    let rotation = Rad::from(Deg(rotation_deg));
    let mat = Matrix2::from_angle(rotation);
    let vec = Vector2::new(point.x, point.y);
    let rotated = mat * vec;
    FloatVec2::new(rotated.x + translation.x, rotated.y + translation.y)
}
/// Golden section search for 1D minimization.
/// f: objective function
/// x_start: lower bound
/// x_end: upper bound
/// tol: tolerance for stopping condition
fn golden_section_search<F>(f: F, x_start: f32, x_end: f32, num_iterations: usize) -> f32
where
    F: Fn(f32) -> f32,
{
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let resphi = 2.0 - phi; // 1 - (1/phi)

    let mut a = x_start;
    let mut b = x_end;

    let mut c = b - resphi * (b - a);
    let mut d = a + resphi * (b - a);

    let mut fc = f(c);
    let mut fd = f(d);

    for _ in 0..num_iterations {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - resphi * (b - a);
            fc = f(c);
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + resphi * (b - a);
            fd = f(d);
        }
    }

    (a + b) / 2.0
}

fn distance_to_round_rect(point: FloatVec2, width: f32, height: f32, corner_radius: f32) -> f32 {
    let right_threashold = width / 2.0 - corner_radius;
    let left_threashold = -right_threashold;
    let top_threashold = height / 2.0 - corner_radius;
    let bottom_threashold = -top_threashold;
    let horizontal_ordering = if point.x > right_threashold {
        std::cmp::Ordering::Greater
    } else if point.x < left_threashold {
        std::cmp::Ordering::Less
    } else {
        std::cmp::Ordering::Equal
    };
    let vertical_ordering = if point.y > top_threashold {
        std::cmp::Ordering::Greater
    } else if point.y < bottom_threashold {
        std::cmp::Ordering::Less
    } else {
        std::cmp::Ordering::Equal
    };
    match (horizontal_ordering, vertical_ordering) {
        (std::cmp::Ordering::Less, std::cmp::Ordering::Less) => {
            let top_left_corner = FloatVec2::new(left_threashold, top_threashold);
            ((point - top_left_corner).length() - corner_radius).abs()
        }
        (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => {
            let top_right_corner = FloatVec2::new(right_threashold, top_threashold);
            ((point - top_right_corner).length() - corner_radius).abs()
        }
        (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => {
            let bottom_left_corner = FloatVec2::new(left_threashold, bottom_threashold);
            ((point - bottom_left_corner).length() - corner_radius).abs()
        }
        (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => {
            let bottom_right_corner = FloatVec2::new(right_threashold, bottom_threashold);
            ((point - bottom_right_corner).length() - corner_radius).abs()
        }
        (std::cmp::Ordering::Less, std::cmp::Ordering::Equal) => (point.x - left_threashold).abs(),
        (std::cmp::Ordering::Greater, std::cmp::Ordering::Equal) => {
            (point.x - right_threashold).abs()
        }
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Less) => {
            (point.y - bottom_threashold).abs()
        }
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Greater) => {
            (point.y - top_threashold).abs()
        }
        _ => {
            let horizontal_min = f32::min(
                (point.x - left_threashold).abs(),
                (point.x - right_threashold).abs(),
            );
            let vertical_min = f32::min(
                (point.y - bottom_threashold).abs(),
                (point.y - top_threashold).abs(),
            );
            f32::min(horizontal_min, vertical_min)
        }
    }
}

fn vertices_to_round_rect_and_scale(vertices: &Vec<FloatVec2>, scale_down_factor: f32) -> PadShape {
    let mut x_max = f32::MIN;
    let mut x_min = f32::MAX;
    let mut y_max = f32::MIN;
    let mut y_min = f32::MAX;
    for vertex in vertices {
        x_max = x_max.max(vertex.x);
        x_min = x_min.min(vertex.x);
        y_max = y_max.max(vertex.y);
        y_min = y_min.min(vertex.y);
    }
    let width = x_max - x_min;
    let height = y_max - y_min;
    let min_corner_radius: f32 = 0.0;
    let max_corner_radius = f32::min(width, height) / 2.0;
    let distance_to_round_rect_closure = |corner_radius: f32| {
        let mut distance_sum = 0.0;
        for vertex in vertices {
            distance_sum += distance_to_round_rect(*vertex, width, height, corner_radius);
        }
        distance_sum
    };
    let corner_radius = golden_section_search(
        distance_to_round_rect_closure,
        min_corner_radius,
        max_corner_radius,
        100,
    );
    PadShape::RoundRect {
        width: width / scale_down_factor,
        height: height / scale_down_factor,
        corner_radius: corner_radius / scale_down_factor,
    }
    // PadShape::Rectangle { 
    //     width: width / scale_down_factor,
    //     height: height / scale_down_factor,
    // }
}

fn convert_shape_and_scale(shape: &Shape, scale_down_factor: f32) -> Result<PadShape, String> {
    match shape {
        Shape::Circle { diameter } => Ok(PadShape::Circle {
            diameter: *diameter / scale_down_factor,
        }),
        Shape::Rect {
            x_min,
            y_min,
            x_max,
            y_max,
        } => Ok(PadShape::Rectangle {
            width: (*x_max - *x_min) / scale_down_factor,
            height: (*y_max - *y_min) / scale_down_factor,
        }),
        Shape::Polygon {
            aperture_width: _,
            vertices,
        } => {
            if vertices.len() < 3 {
                return Err("Polygon must have at least 3 vertices".to_string());
            }
            // For simplicity, we treat the polygon as a round rectangle
            let round_rect_shape = vertices_to_round_rect_and_scale(vertices, scale_down_factor);
            Ok(round_rect_shape)
        }
    }
}

fn build_pad_map_and_scale(
    dsn: &DsnStruct,
    scale_down_factor: f32,
) -> Result<HashMap<String, TransformedPad>, String> {
    let mut pad_map: HashMap<String, TransformedPad> = HashMap::new();

    for component in &dsn.placement.components {
        let image = dsn
            .library
            .images
            .get(&component.name)
            .ok_or_else(|| format!("Image not found: {}", component.name))?;

        for instance in &component.instances {
            for (pin_number, pin) in &image.pins {
                let pad_stack = dsn
                    .library
                    .pad_stacks
                    .get(&pin.pad_stack_name)
                    .ok_or_else(|| format!("Pad stack not found: {}", pin.pad_stack_name))?;

                let pin_rotation = pin.rotation;
                // 1. 先应用pin相对footprint的位移
                let mut position = pin.position;

                // 2. 应用footprint旋转
                position =
                    transform_point(position, instance.rotation, FloatVec2 { x: 0.0, y: 0.0 });

                // 3. 应用footprint位移
                position.x += instance.position.x;
                position.y += instance.position.y;

                // 转换形状
                let shape = convert_shape_and_scale(&pad_stack.shape, scale_down_factor)?;

                // 创建唯一标识符
                let pad_key = format!("{}-{}", instance.reference, pin_number);
                let pad_layer = if pad_stack.through_hole {
                    PadLayer::All
                } else {
                    match instance.placement_layer {
                        PlacementLayer::Front => PadLayer::Front,
                        PlacementLayer::Back => PadLayer::Back,
                    }
                };
                let total_rotation = Deg(instance.rotation + pin_rotation.0);
                pad_map.insert(
                    pad_key,
                    TransformedPad {
                        component_name: instance.reference.clone(),
                        pin_number: pin_number.clone(),
                        position: position / scale_down_factor,
                        shape,
                        rotation: total_rotation,
                        pad_layer,
                    },
                );
            }
        }
    }

    Ok(pad_map)
}

fn pins_to_pads_and_scale(
    pins: &Vec<Pin2>,
    dsn: &DsnStruct,
    scale_down_factor: f32,
) -> Result<Vec<Pad>, String> {
    let pad_map = build_pad_map_and_scale(&dsn, scale_down_factor)?;
    let mut pads: Vec<Pad> = Vec::new();
    let mut net_clearance_map = HashMap::new();
    for (_, netclass) in &dsn.network.netclasses {
        for net_name in &netclass.net_names {
            net_clearance_map.insert(net_name.clone(), netclass.clearance / scale_down_factor);
        }
    }

    // 预构建pin到net_name的映射
    let mut pin_to_net = HashMap::new();
    for net in &dsn.network.nets {
        for pin in &net.pins {
            let key = format!("{}-{}", pin.component_name, pin.pin_number);
            pin_to_net.insert(key, net.name.clone());
        }
    }

    // 转换每个Pin2
    for pin in pins {
        let pad_key = format!("{}-{}", pin.component_name, pin.pin_number);

        // 查找pad基本信息
        let transformed_pad = pad_map
            .get(&pad_key)
            .ok_or_else(|| format!("Pad {}-{} not found", pin.component_name, pin.pin_number))?;

        // 查找所属网络的clearance
        let clearance = pin_to_net
            .get(&pad_key)
            .and_then(|net_name| net_clearance_map.get(net_name))
            .copied()
            .unwrap_or(0.0); // 默认值

        pads.push(Pad {
            name: PadName(pad_key),
            position: transformed_pad.position,
            shape: transformed_pad.shape.clone(),
            rotation: transformed_pad.rotation,
            clearance,
            pad_layer: transformed_pad.pad_layer,
        });
    }

    Ok(pads)
}

// #[derive(Debug)]
// pub struct ScaledNetClassProperties {
//     pub name: NetClassName,
//     pub width: f32,
//     pub clearance: f32,
//     pub via_name: String,
// }

// fn find_netclass_and_scale(network: &Network, net_name: &String, scale_down_factor: f32) -> Result<ScaledNetClassProperties, String> {
//     network
//         .netclasses
//         .values()
//         .find(|netclass| netclass.net_names.iter().any(|net| net == net_name))
//         .map(|found_class| ScaledNetClassProperties {
//             name: NetClassName(found_class.net_class_name.clone()),
//             width: found_class.width as f32,
//             clearance: found_class.clearance as f32,
//             via_name: found_class.via_name.clone(),
//         })
//         .ok_or_else(|| format!("Net '{}' doesn't belong to any netclass", net_name))
// }

fn parse_net_info_and_scale(
    dsn: &DsnStruct,
    scale_down_factor: f32,
) -> Result<HashMap<NetName, DisplayNetInfo>, String> {
    let mut net_info: HashMap<NetName, DisplayNetInfo> = HashMap::new();
    let mut net_to_net_class: HashMap<String, &Netclass> = HashMap::new();
    for netclass in dsn.network.netclasses.values() {
        for net_name in &netclass.net_names {
            net_to_net_class.insert(net_name.clone(), netclass);
        }
    }
    let mut net_to_via_diameter_scaled: HashMap<String, f32> = HashMap::new();
    for (net_name, netclass) in &net_to_net_class {
        let pad_stack = dsn
            .library
            .pad_stacks
            .get(&netclass.via_name)
            .ok_or_else(|| {
                format!(
                    "Via '{}' not found for net '{}'",
                    netclass.via_name, net_name
                )
            })?;
        let via_diameter = match &pad_stack.shape {
            Shape::Circle { diameter } => *diameter as f32,
            _ => {
                return Err(format!(
                    "Invalid via '{}' for net '{}': not circular",
                    netclass.via_name, net_name
                ));
            }
        };
        net_to_via_diameter_scaled.insert(net_name.clone(), via_diameter / scale_down_factor);
    }
    for all_nets in dsn.network.nets.iter() {
        let net_class = net_to_net_class
            .get(&all_nets.name)
            .ok_or_else(|| format!("Net '{}' doesn't belong to any netclass", all_nets.name))?;
        let net_name = all_nets.name.clone();
        let pads = pins_to_pads_and_scale(&all_nets.pins, &dsn, scale_down_factor)?;
        let via_diameter_scaled = *net_to_via_diameter_scaled
            .get(&net_name)
            .ok_or_else(|| format!("Via diameter not found for net '{}'", net_name))?;
        net_info.insert(
            NetName(net_name.clone()),
            DisplayNetInfo {
                net_name: NetName(net_name),
                pads,
                net_class_name: NetClassName(net_class.net_class_name.clone()),
                default_trace_width: net_class.width / scale_down_factor,
                default_trace_clearance: net_class.clearance / scale_down_factor,
                via_diameter: via_diameter_scaled,
            },
        );
    }
    Ok(net_info)
}

pub fn dsn_to_display(dsn: &DsnStruct) -> Result<DisplayFormat, String> {
    let unit = &dsn.resolution.unit;
    let scale_down_factor: f32 = match unit.as_str() {
        "um" => 1000.0,
        _ => panic!("Unsupported unit: {}", unit),
    };
    let (width, height, center) =
        calculate_boundary_and_scale(&dsn.structure.boundary, scale_down_factor)?;
    let num_layers = dsn.structure.layers.len();
    if num_layers == 0 || num_layers % 2 == 1 {
        return Err(format!(
            "Invalid number of layers: {}, must be even and greater than 0",
            num_layers
        ));
    }
    let obstacle_lines: Vec<Line> = Vec::new();
    let obstacle_polygons: Vec<PolygonCollider> = Vec::new();
    let net_info: HashMap<NetName, DisplayNetInfo> =
        parse_net_info_and_scale(&dsn, scale_down_factor)?;

    let display_format = DisplayFormat {
        width,
        height,
        center,
        num_layers,
        obstacle_lines,
        obstacle_polygons,
        nets: net_info,
        scale_down_factor,
    };
    Ok(display_format)
}
