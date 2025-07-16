use crate::dsn_struct::{
    Boundary, Component, ComponentInst, DsnStruct, Network, PadStack, Pin, Pin2, Placement, Shape,
};
use crate::parse_to_display_format::{DisplayFormat, DisplayNetInfo, ExtraInfo};

use cgmath::{Deg, Matrix2, Rad, Vector2};
use shared::pad::{Pad, PadName, PadShape};
use shared::pcb_problem::{NetClassName, NetName};
use shared::prim_shape::{LineForCollision, PolygonForCollision};
use shared::vec2::{FixedVec2, FloatVec2};
use core::net;
use std::collections::HashMap;

fn calculate_boundary(boundary: &Boundary) -> Result<(f32, f32, FloatVec2), String> {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    for (x, y) in &boundary.0 {
        min_x = min_x.min(*x);
        max_x = max_x.max(*x);
        min_y = min_y.min(*y);
        max_y = max_y.max(*y);
    }

    let width = (max_x - min_x) as f32;
    let height = (max_y - min_y) as f32;
    let center = FloatVec2 {
        x: (min_x + max_x) as f32 / 2.0,
        y: (min_y + max_y) as f32 / 2.0,
    };

    Ok((width, height, center))
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
    pub pin_number: usize,
    pub position: FloatVec2, // 最终PCB坐标系下的位置
    pub shape: PadShape,
    pub rotation: cgmath::Deg<f32>, // 最终旋转角度（度）
}

fn transform_point(point: FloatVec2, rotation_deg: f32, translation: FloatVec2) -> FloatVec2 {
    let rotation = Rad::from(Deg(rotation_deg));
    let mat = Matrix2::from_angle(rotation);
    let vec = Vector2::new(point.x, point.y);
    let rotated = mat * vec;
    FloatVec2::new(rotated.x + translation.x, rotated.y + translation.y)
}

fn vertices_to_round_rect(vertices: &Vec<FloatVec2>)->PadShape{
    todo!("Convert vertices to round rectangle shape");
}

fn convert_shape(shape: &Shape) -> Result<PadShape, String> {
    match shape {
        Shape::Circle { diameter } => Ok(PadShape::Circle {
            diameter: *diameter as f32,
        }),
        Shape::Rect {
            x_min,
            y_min,
            x_max,
            y_max,
        } => Ok(PadShape::Rectangle {
            width: (*x_max - *x_min) as f32,
            height: (*y_max - *y_min) as f32,
        }),
        Shape::Polygon {
            aperture_width,
            vertices,
        } => {
            if vertices.len() < 3 {
                return Err("Polygon must have at least 3 vertices".to_string());
            }
            // For simplicity, we treat the polygon as a round rectangle
            Ok(PadShape::RoundRect {
                width: 10000.0,
                height: 10000.0, // Assuming square for simplicity
                corner_radius: 2000.0,             // Not specified in the original code
            })
        }
    }
}

fn build_pad_map(dsn: &DsnStruct) -> Result<HashMap<String, TransformedPad>, String> {
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

                // 1. 先应用pin相对footprint的位移
                let mut position = pin.position;

                // 2. 应用footprint旋转
                position = transform_point(position, instance.rotation, FloatVec2 { x: 0.0, y: 0.0 });

                // 3. 应用footprint位移
                position.x += instance.position.x;
                position.y += instance.position.y;

                // 转换形状
                let shape = convert_shape(&pad_stack.shape)?;

                // 创建唯一标识符
                let pad_key = format!("{}-{}", instance.reference, pin_number);

                pad_map.insert(
                    pad_key,
                    TransformedPad {
                        component_name: instance.reference.clone(),
                        pin_number: *pin_number,
                        position,
                        shape,
                        rotation: Deg(instance.rotation as f32),
                    },
                );
            }
        }
    }

    Ok(pad_map)
}

fn pins_to_pads(pins: &Vec<Pin2>, dsn: &DsnStruct) -> Result<Vec<Pad>, String> {
    let mut pad_map = build_pad_map(&dsn)?;
    let mut pads: Vec<Pad> = Vec::new();
    let mut net_clearance_map = HashMap::new();
    for (_, netclass) in &dsn.network.netclasses {
        for net_name in &netclass.net_names {
            net_clearance_map.insert(net_name.clone(), netclass.clearance as f32);
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
        });
    }

    Ok(pads)
}

#[derive(Debug)]
pub struct NetClassProperties {
    pub name: NetClassName,
    pub width: f32,
    pub clearance: f32,
    pub via_name: String,
}

fn find_netclass(network: &Network, net_name: &String) -> Result<NetClassProperties, String> {
    network
        .netclasses
        .values()
        .find(|netclass| netclass.net_names.iter().any(|net| net == net_name))
        .map(|found_class| NetClassProperties {
            name: NetClassName(found_class.net_class_name.clone()),
            width: found_class.width as f32,
            clearance: found_class.clearance as f32,
            via_name: found_class.via_name.clone(),
        })
        .ok_or_else(|| format!("Net '{}' doesn't belong to any netclass", net_name))
}

fn parse_net_info(dsn: &DsnStruct) -> Result<HashMap<NetName, DisplayNetInfo>, String> {
    let mut net_info: HashMap<NetName, DisplayNetInfo> = HashMap::new();
    for all_nets in dsn.network.nets.iter() {
        let net_name = all_nets.name.clone();
        let pads = pins_to_pads(&all_nets.pins, &dsn)?;
        let net_class_properties = find_netclass(&dsn.network, &net_name)?;
        let via_diameter = dsn
            .library
            .pad_stacks
            .get(&net_class_properties.via_name)
            .and_then(|pad_stack| match &pad_stack.shape {
                Shape::Circle { diameter } => Some(*diameter as f32),
                _ => None,
            })
            .ok_or_else(|| {
                format!(
                    "Invalid via '{}' for net '{}': not found or not circular",
                    net_class_properties.via_name, net_name
                )
            })?;
        net_info.insert(
            NetName(net_name.clone()),
            DisplayNetInfo {
                net_name: NetName(net_name),
                pads,
                net_class_name: net_class_properties.name,
                default_trace_width: net_class_properties.width,
                default_trace_clearance: net_class_properties.clearance,
                via_diameter,
            },
        );
    }
    Ok(net_info)
}

pub fn dsn_to_display(dsn: &DsnStruct) -> Result<DisplayFormat, String> {
    let (width, height, center) = calculate_boundary(&dsn.structure.boundary)?;
    let obstacle_lines: Vec<LineForCollision> = Vec::new();
    let obstacle_polygons: Vec<PolygonForCollision> = Vec::new();
    let net_info: HashMap<NetName, DisplayNetInfo> = parse_net_info(&dsn)?;

    let display_format = DisplayFormat {
        width,
        height,
        center,
        obstacle_lines,
        obstacle_polygons,
        nets: net_info,
    };
    Ok(display_format)
}
