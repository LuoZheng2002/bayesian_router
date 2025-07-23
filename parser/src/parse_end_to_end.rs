use std::collections::HashMap;

use shared::pcb_problem::PcbProblem;

use crate::{
    dsn_struct::{DsnStruct, Shape},
    parse_to_display::{self, dsn_to_display},
    parse_to_display_format::ExtraInfo,
    parse_to_pcbproblem::{self, Converter},
    parse_to_s_expr::parse_dsn_to_s_expr,
    parse_to_struct::parse_s_expr_to_struct,
};

pub fn parse_struct_to_end(dsn_struct: &DsnStruct) -> Result<PcbProblem, String> {
    let display_format = dsn_to_display(dsn_struct)?;
    let extra_info = ExtraInfo {
        net_name_to_source_pad: HashMap::new(),
    };
    let pcb_problem = Converter::convert(&display_format, &extra_info)?;
    Ok(pcb_problem)
}
pub fn parse_start_to_dsn_struct(dsn_file_content: String) -> Result<DsnStruct, String> {
    let s_expr = parse_dsn_to_s_expr(&dsn_file_content)
        .map_err(|e| format!("Failed to parse DSN: {}", e))?;
    parse_s_expr_to_struct(&s_expr)
}

pub fn parse_end_to_end(dsn_file_content: String) -> Result<PcbProblem, String> {
    let s_expr = parse_dsn_to_s_expr(&dsn_file_content)
        .map_err(|e| format!("Failed to parse DSN: {}", e))?;

    let dsn_struct = parse_s_expr_to_struct(&s_expr)?;

    /*
        println!(
            "Resolution: {} {}",
            dsn_struct.resolution.value, dsn_struct.resolution.unit
        );
        println!(
            "Layers: {:?}",
            dsn_struct
                .structure
                .layers
                .iter()
                .map(|l| &l.name)
                .collect::<Vec<_>>()
        );
        println!("Boundary: {:?}", dsn_struct.structure.boundary.0);
        println!(
            "COMPONENTS: {:?}",
            dsn_struct
                .placement
                .components
                .iter()
                .map(|c| &c.name)
                .collect::<Vec<_>>()
        );
        for component in &dsn_struct.placement.components {
            println!("Component: {}", component.name);
            for instance in &component.instances {
                println!(
                    "  Instance: {}, {:?} rotation {}",
                    instance.reference, instance.position, instance.rotation
                );
            }
        }
        println!("\nLIBRARY IMAGES:");
        for (image_name, image) in &dsn_struct.library.images {
            println!("Image: {}", image_name);
            println!("  Pins:");
            for (pin_num, pin) in &image.pins {
                println!(
                    "    Pin {}: pad_stack={}, position={:?}",
                    pin_num, pin.pad_stack_name, pin.position
                );
            }
        }

        println!("\nLIBRARY PADSTACKS:");
        for (padstack_name, padstack) in &dsn_struct.library.pad_stacks {
            println!("PadStack: {}", padstack_name);
            println!("  Through hole: {}", padstack.through_hole);
            match &padstack.shape {
                Shape::Circle { diameter } => {
                    println!("  Shape: Circle (diameter: {})", diameter);
                }
                Shape::Rect {
                    x_min,
                    y_min,
                    x_max,
                    y_max,
                } => {
                    println!(
                        "  Shape: Rect (x: {} to {}, y: {} to {})",
                        x_min, x_max, y_min, y_max
                    );
                }
                Shape::Polygon {
                    aperture_width,
                    vertices,
                } => {
                    println!(
                        "  Shape: Polygon (aperture width: {}, vertices: {})",
                        aperture_width,
                        vertices.len()
                    );
                    for (i, vertex) in vertices.iter().enumerate() {
                        println!("    Vertex {}: {:?}", i + 1, vertex);
                    }
                }
            }
        }

        println!("\nNETWORK:");
        println!("Netclasses:");
        for (class_name, netclass) in &dsn_struct.network.netclasses {
            println!("  Class: {}", class_name);
            println!("    Via: {}", netclass.via_name);
            println!("    Width: {}", netclass.width);
            println!("    Clearance: {}", netclass.clearance);
            println!("    Nets: {:?}", netclass.net_names);
        }

        println!("\nNets:");
        for net in &dsn_struct.network.nets {
            println!("  Net: {}", net.name);
            println!("    Pins:");
            for pin in &net.pins {
                println!("      {} pin {}", pin.component_name, pin.pin_number);
            }
        }
    */
    let display_format = dsn_to_display(&dsn_struct)?;
    let extra_info = ExtraInfo {
        net_name_to_source_pad: HashMap::new(),
    };
    let pcb_problem = Converter::convert(&display_format, &extra_info)?;
    Ok(pcb_problem)
}
