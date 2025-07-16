use parser::dsn_struct::Shape;
use parser::parse_to_s_expr::parse_dsn_to_s_expr;
use parser::parse_to_struct::parse_s_expr_to_struct;

fn main() {
    let data = std::fs::read_to_string("specctra_test.dsn").unwrap();
    let result = match parse_dsn_to_s_expr(&data) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            panic!("Failed to parse the DSN file");
        }
    };
    //println!("{:#?}", result);
    let dsn_struct = match parse_s_expr_to_struct(&result) {
        Ok(structure) => structure,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            panic!("Failed to convert S-Expression to struct");
        }
    };
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
}
