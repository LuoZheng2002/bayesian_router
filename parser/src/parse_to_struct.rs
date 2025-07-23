//use core::net;
use std::collections::HashMap;

use cgmath::Deg;
use shared::vec2::FloatVec2;

use crate::{
    dsn_struct::{
        Boundary, Component, ComponentInst, DsnStruct, Image, Layer, Library, Net, Netclass,
        Network, PadStack, Pin, Pin2, Placement, PlacementLayer, Resolution, Shape, Structure,
    },
    s_expr::SExpr,
};

fn parse_layer(s_expr: &Vec<SExpr>) -> Result<Layer, String> {
    let first_item = s_expr
        .first()
        .ok_or("Expected at least one item in the layer scope")?;
    let first_item = first_item
        .as_atom()
        .ok_or("Expected an atom as the first item in the layer scope")?;
    if first_item != "layer" {
        return Err(format!(
            "Expected 'layer' as the first item, found: {}",
            first_item
        ));
    }
    let second_item = s_expr
        .get(1)
        .ok_or("Expected a second item in the layer scope")?;
    let second_item = second_item
        .as_atom()
        .ok_or("Expected an atom as the second item in the layer scope")?;
    Ok(Layer {
        name: second_item.to_string(),
    })
}

fn parse_boundary(s_expr: &Vec<SExpr>) -> Result<Boundary, String> {
    let first_item = s_expr
        .first()
        .ok_or("Expected at least one item in the boundary scope")?;
    let first_item = first_item
        .as_atom()
        .ok_or("Expected an atom as the first item in the boundary scope")?;
    if first_item != "boundary" {
        return Err(format!(
            "Expected 'boundary' as the first item, found: {}",
            first_item
        ));
    }
    let second_item = s_expr
        .get(1)
        .ok_or("Expected a second item in the boundary scope")?;
    let second_list = second_item
        .as_list()
        .ok_or("Expected a list as the second item in the boundary scope")?;
    if second_list.len() < 3 {
        return Err("Expected at least three items in the boundary list".to_string());
    }
    let mut points: Vec<FloatVec2> = Vec::new();
    let mut prev_number: Option<f32> = None;
    for item in second_list.iter().skip(3) {
        match prev_number {
            Some(prev_num) => {
                let number = item
                    .as_atom()
                    .ok_or("Expected an atom in the boundary list")?;
                let number = number
                    .parse::<f32>()
                    .map_err(|e| format!("Failed to parse boundary number: {}", e))?;
                points.push(FloatVec2 {
                    x: prev_num,
                    y: number,
                });
                prev_number = None;
            }
            None => {
                let number = item
                    .as_atom()
                    .ok_or("Expected an atom in the boundary list")?;
                let number = number
                    .parse::<f32>()
                    .map_err(|e| format!("Failed to parse boundary number: {}", e))?;
                prev_number = Some(number);
            }
        }
    }
    if prev_number.is_some() {
        return Err("Expected an even number of items in the boundary list".to_string());
    }
    let boundary = Boundary(points);
    Ok(boundary)
}

fn parse_structure(s_expr: &Vec<SExpr>) -> Result<Structure, String> {
    // Placeholder for structure parsing logic
    // This function should parse the structure part of the S-expression
    // and populate the DsnStruct accordingly.
    let first_item = s_expr
        .first()
        .ok_or("Expected at least one item in the structure scope")?;
    let first_item = first_item
        .as_atom()
        .ok_or("Expected an atom as the first item in the structure scope")?;
    if first_item != "structure" {
        return Err(format!(
            "Expected 'structure' as the first item, found: {}",
            first_item
        ));
    }
    let mut layers: Vec<Layer> = Vec::new();
    let mut boundary: Option<Boundary> = None;
    for item in s_expr.iter().skip(1) {
        let expr_list = item.as_list().ok_or(format!(
            "Expected a list in the structure scope, found: {:?}",
            item
        ))?;
        let first_item = expr_list
            .first()
            .ok_or("Expected at least one item in the structure item")?;
        let first_item = first_item
            .as_atom()
            .ok_or("Expected an atom as the first item in the structure item")?;
        match first_item.as_str() {
            "layer" => {
                let layer = parse_layer(expr_list)?;
                layers.push(layer);
            }
            "boundary" => {
                boundary = Some(parse_boundary(expr_list)?);
            }
            "via" => {
                continue;
            }
            "rule" => {
                continue;
            }
            "plane" => {
                continue;
            }
            "keepout" => {
                continue; // to do
            }
            _ => {
                return Err(format!("Unknown structure item: {}", first_item));
            }
        }
    }
    let boundary = boundary.ok_or("Expected a boundary in the structure scope")?;
    Ok(Structure { layers, boundary })
}

fn parse_placement(s_expr: &Vec<SExpr>) -> Result<Placement, String> {
    // Placeholder for placement parsing logic
    // This function should parse the placement part of the S-expression
    // and populate the DsnStruct accordingly.
    let first_item = s_expr
        .first()
        .ok_or("Expected at least one item in the placement scope")?;
    let first_item = first_item
        .as_atom()
        .ok_or("Expected an atom as the first item in the placement scope")?;
    if first_item != "placement" {
        return Err(format!(
            "Expected 'placement' as the first item, found: {}",
            first_item
        ));
    }
    let mut components: Vec<Component> = Vec::new();
    for item in s_expr.iter().skip(1) {
        let expr_list = item.as_list().ok_or(format!(
            "Expected a list in the placement scope, found: {:?}",
            item
        ))?;
        let first_item = expr_list
            .first()
            .ok_or("Expected at least one item in the placement item")?;
        let first_item = first_item
            .as_atom()
            .ok_or("Expected an atom as the first item in the placement item")?;
        if first_item != "component" {
            return Err(format!(
                "Expected 'component' as the first item in the placement item, found: {}",
                first_item
            ));
        }

        let component_name = expr_list
            .get(1)
            .ok_or("Expected component name")?
            .as_atom()
            .ok_or("Expected component name to be an atom")?
            .to_string();

        let mut instances = Vec::new();

        for place_expr in expr_list.iter().skip(2) {
            let place_list = place_expr
                .as_list()
                .ok_or(format!("Expected place list, found: {:?}", place_expr))?;

            let first_place_item = place_list
                .first()
                .ok_or("Expected at least one item in place list")?
                .as_atom()
                .ok_or("Expected 'place' as first item in place list")?;

            if first_place_item != "place" {
                return Err(format!(
                    "Expected 'place' as first item in place list, found: {}",
                    first_place_item
                ));
            }

            // Parse place instance details
            let reference = place_list
                .get(1)
                .ok_or("Expected reference in place list")?
                .as_atom()
                .ok_or("Expected reference to be an atom")?
                .to_string();

            let x_pos = place_list
                .get(2)
                .ok_or("Expected x position in place list")?
                .as_atom()
                .ok_or("Expected x position to be an atom")?
                .parse::<f32>()
                .map_err(|e| format!("Failed to parse x position: {}", e))?;

            let y_pos = place_list
                .get(3)
                .ok_or("Expected y position in place list")?
                .as_atom()
                .ok_or("Expected y position to be an atom")?
                .parse::<f32>()
                .map_err(|e| format!("Failed to parse y position: {}", e))?;

            let placement_layer_string = place_list
                .get(4)
                .ok_or("Expected placement layer in place list")?
                .as_atom()
                .ok_or("Expected placement layer to be an atom")?
                .to_string();
            let placement_layer = match placement_layer_string.as_str() {
                "front" => PlacementLayer::Front,
                "back" => PlacementLayer::Back,
                _ => {
                    return Err(format!(
                        "Unknown placement layer: {}",
                        placement_layer_string
                    ));
                }
            };

            let rotation = place_list
                .get(5)
                .ok_or("Expected rotation in place list")?
                .as_atom()
                .ok_or("Expected rotation to be an atom")?
                .parse::<f32>()
                .map_err(|e| format!("Failed to parse rotation: {}", e))?;

            // Create the component instance
            let instance = ComponentInst {
                reference,
                position: FloatVec2 { x: x_pos, y: y_pos },
                rotation,
                placement_layer, // Use the parsed placement layer
            };
            instances.push(instance);
        }

        let component = Component {
            name: component_name,
            instances,
        };
        components.push(component);
    }

    Ok(Placement { components })
}

fn parse_image(s_expr: &Vec<SExpr>) -> Result<Image, String> {
    // Placeholder for image parsing logic
    let first_item = s_expr
        .first()
        .ok_or("Expected at least one item in the image scope")?;
    let first_item = first_item
        .as_atom()
        .ok_or("Expected an atom as the first item in the image scope")?;
    if first_item != "image" {
        return Err(format!(
            "Expected 'image' as the first item, found: {}",
            first_item
        ));
    }
    let image_name = s_expr
        .get(1)
        .ok_or("Expected image name as the second item")?
        .as_atom()
        .ok_or("Expected image name to be an atom")?
        .to_string();

    let mut pins: HashMap<String, Pin> = HashMap::new();
    for item in s_expr.iter().skip(2) {
        let expr_list = item.as_list().ok_or(format!(
            "Expected a list in the structure scope, found: {:?}",
            item
        ))?;
        let first_item = expr_list
            .first()
            .ok_or("Expected at least one item in the structure item")?;
        let first_item = first_item
            .as_atom()
            .ok_or("Expected an atom as the first item in the structure item")?;
        match first_item.as_str() {
            "outline" => {
                continue;
            },
            "keepout"=>{
                continue;
            },
            "pin" => {
                let pad_stack_name = expr_list
                    .get(1)
                    .ok_or("Expected pad stack name")?
                    .as_atom()
                    .ok_or("Pad stack name must be an atom")?
                    .to_string();

                let mut next_index: usize = 2;
                let mut rotation: Deg<f32> = Deg(0.0);
                if let SExpr::List(pin_expr_list) =
                    &expr_list.get(2).ok_or("Expected pin definition")?
                {
                    next_index = 3;
                    let first_item = pin_expr_list[0]
                        .as_atom()
                        .ok_or("Expected rotation as the first item in rotation definition")?;
                    let first_item = first_item
                        .parse::<String>()
                        .map_err(|_| "Failed to parse rotation as string")?;
                    if first_item != "rotate" {
                        return Err(format!("Expected 'rotate', found: {}", first_item));
                    }
                    let rot = pin_expr_list[1]
                        .as_atom()
                        .ok_or("Rotation must be an atom")?
                        .parse::<f32>()
                        .map_err(|e| format!("Invalid rotation: {}", e))?;
                    rotation = Deg(rot);
                }
                let pin_number = expr_list
                    .get(next_index)
                    .ok_or("Expect pin number, but out of bound")?
                    .as_atom()
                    .ok_or("Pin number must be an atom")?
                    .clone();
                next_index += 1;
                let x = expr_list
                    .get(next_index)
                    .ok_or("Expect x coordinate, but out of bound")?
                    .as_atom()
                    .ok_or("X coordinate must be a number")?
                    .parse::<f32>()
                    .map_err(|e| format!("Invalid x coordinate: {}", e))?;
                next_index += 1;
                let y = expr_list
                    .get(next_index)
                    .ok_or("Expect y coordinate, but out of bound")?
                    .as_atom()
                    .ok_or("Y coordinate must be a number")?
                    .parse::<f32>()
                    .map_err(|e| format!("Invalid y coordinate: {}", e))?;

                pins.insert(
                    pin_number.clone(),
                    Pin {
                        pad_stack_name,
                        pin_number,
                        position: FloatVec2 { x, y },
                        rotation,
                    },
                );
            }
            _ => {
                return Err(format!("Unknown image item: {}", first_item));
            }
        }
    }

    Ok(Image {
        name: image_name,
        pins,
    })
}

fn parse_shape(s_expr: &Vec<SExpr>) -> Result<Shape, String> {
    let shape_type = s_expr
        .get(1)
        .ok_or("Missing shape type")?
        .as_list()
        .ok_or("Shape type must be a list")?;

    let first_item = shape_type
        .first()
        .ok_or("Empty shape definition")?
        .as_atom()
        .ok_or("Shape type must be an atom")?;

    match first_item.as_str() {
        "circle" => {
            // (shape (circle F.Cu diameter))
            let diameter = shape_type[2]
                .as_atom()
                .ok_or("Circle diameter must be a number")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid circle diameter: {}", e))?;

            Ok(Shape::Circle { diameter })
        }
        "rect" => {
            // (shape (rect F.Cu x_min y_min x_max y_max))
            if shape_type.len() < 6 {
                return Err("Rect requires 4 coordinates".into());
            }
            let x_min = shape_type[2]
                .as_atom()
                .ok_or("Rect x_min must be a number")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid x_min: {}", e))?;
            let y_min = shape_type[3]
                .as_atom()
                .ok_or("Rect y_min must be a number")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid y_min: {}", e))?;
            let x_max = shape_type[4]
                .as_atom()
                .ok_or("Rect x_max must be a number")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid x_max: {}", e))?;
            let y_max = shape_type[5]
                .as_atom()
                .ok_or("Rect y_max must be a number")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid y_max: {}", e))?;

            Ok(Shape::Rect {
                x_min,
                y_min,
                x_max,
                y_max,
            })
        }
        "polygon" => {
            // (shape (polygon F.Cu aperture_width vertices...))
            if shape_type.len() < 4 {
                return Err("Polygon requires aperture width and vertices".into());
            }
            let aperture_width = shape_type[2]
                .as_atom()
                .ok_or("Aperture width must be a number")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid aperture width: {}", e))?;

            let mut vertices = Vec::new();
            for i in (3..shape_type.len()).step_by(2) {
                if i + 1 >= shape_type.len() {
                    break;
                }
                let x = shape_type[i]
                    .as_atom()
                    .ok_or("Vertex x must be a number")?
                    .parse::<f32>()
                    .map_err(|e| format!("Invalid vertex x: {}", e))?;
                let y = shape_type[i + 1]
                    .as_atom()
                    .ok_or("Vertex y must be a number")?
                    .parse::<f32>()
                    .map_err(|e| format!("Invalid vertex y: {}", e))?;
                vertices.push(FloatVec2 { x, y });
            }

            Ok(Shape::Polygon {
                aperture_width,
                vertices,
            })
        }
        "path" => {
            let aperture_width = shape_type[2]
                .as_atom()
                .ok_or("Aperture width must be a number")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid aperture width: {}", e))?;
            Ok(Shape::Circle {
                diameter: aperture_width,
            })
        }
        _ => Err(format!("Unknown shape type: {}", first_item)),
    }
}

fn parse_padstack(s_expr: &Vec<SExpr>) -> Result<PadStack, String> {
    // Placeholder for padstack parsing logic
    let first_item = s_expr
        .first()
        .ok_or("Expected at least one item in the padstack scope")?;
    let first_item = first_item
        .as_atom()
        .ok_or("Expected an atom as the first item in the padstack scope")?;
    if first_item != "padstack" {
        return Err(format!(
            "Expected 'padstack' as the first item, found: {}",
            first_item
        ));
    }
    let padstack_name = s_expr
        .get(1)
        .ok_or("Expected padstack name as the second item")?
        .as_atom()
        .ok_or("Expected padstack name to be an atom")?
        .to_string();

    let mut shapes = None;
    let mut shape_num = 0;
    for item in s_expr.iter().skip(2) {
        let expr_list = item.as_list().ok_or(format!(
            "Expected a list in the padstack scope, found: {:?}",
            item
        ))?;
        let first_item = expr_list
            .first()
            .ok_or("Expected at least one item in the padstack item")?;
        let first_item = first_item
            .as_atom()
            .ok_or("Expected an atom as the first item in the padstack item")?;
        match first_item.as_str() {
            "shape" => {
                shape_num += 1;
                if shapes.is_none() {
                    shapes = Some(parse_shape(expr_list)?);
                }
            }
            "attach" => {
                continue;
            }
            _ => {
                return Err(format!("Unknown padstack item: {}", first_item));
            }
        }
    }
    let shape = shapes.ok_or("Padstack must have at least one shape")?;
    let through_hole = shape_num > 1;
    Ok(PadStack {
        name: padstack_name,
        shape,
        through_hole,
    })
}

fn parse_library(s_expr: &Vec<SExpr>) -> Result<Library, String> {
    // Placeholder for library parsing logic
    // This function should parse the library part of the S-expression
    // and populate the DsnStruct accordingly.
    let first_item = s_expr
        .first()
        .ok_or("Expected at least one item in the library scope")?;
    let first_item = first_item
        .as_atom()
        .ok_or("Expected an atom as the first item in the library scope")?;
    if first_item != "library" {
        return Err(format!(
            "Expected 'library' as the first item, found: {}",
            first_item
        ));
    }

    let mut images: HashMap<String, Image> = HashMap::new();
    let mut pad_stacks: HashMap<String, PadStack> = HashMap::new();

    for item in s_expr.iter().skip(1) {
        let expr_list = item.as_list().ok_or(format!(
            "Expected a list in the library scope, found: {:?}",
            item
        ))?;
        let first_item = expr_list
            .first()
            .ok_or("Expected at least one item in the library item")?;
        let first_item = first_item
            .as_atom()
            .ok_or("Expected an atom as the first item in the library item")?;
        match first_item.as_str() {
            "image" => {
                let image = parse_image(expr_list)?;
                images.insert(image.name.clone(), image);
            }
            "padstack" => {
                let padstack = parse_padstack(expr_list)?;
                pad_stacks.insert(padstack.name.clone(), padstack);
            }
            _ => {
                return Err(format!("Unknown library item: {}", first_item));
            }
        }
    }
    Ok(Library { images, pad_stacks })
}

fn parse_net(s_expr: &Vec<SExpr>) -> Result<Net, String> {
    let first_item = s_expr
        .first()
        .ok_or("Expected at least one item in the net scope")?;
    let first_item = first_item
        .as_atom()
        .ok_or("Expected an atom as the first item in the net scope")?;
    if first_item != "net" {
        return Err(format!(
            "Expected 'net' as the first item, found: {}",
            first_item
        ));
    }

    let net_name = s_expr
        .get(1)
        .ok_or("Expected net name as the second item")?
        .as_atom()
        .ok_or("Expected net name to be an atom")?
        .to_string();

    let pins_list = match &s_expr[2] {
        SExpr::List(list) => list,
        _ => return Err("Pins must be a list".to_string()),
    };
    if pins_list.is_empty() {
        return Err("Empty pins list".to_string());
    }
    let pins_head = match &pins_list[0] {
        SExpr::Atom(head) => head,
        _ => return Err("Pins list must start with 'pins'".to_string()),
    };
    if pins_head != "pins" {
        return Err(format!("Expected 'pins', got '{}'", pins_head));
    }

    let mut pins: Vec<Pin2> = Vec::new();
    for pin_expr in pins_list.iter().skip(1) {
        let pin_str = pin_expr.as_atom().ok_or(format!(
            "Expected pin as atom (e.g. 'U1-5'), found: {:?}",
            pin_expr
        ))?;

        // Split the pin string into component name and pin number
        let parts: Vec<&str> = pin_str.split('-').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid pin format: expected 'COMPONENT-PINNUM', got '{}'",
                pin_str
            ));
        }

        let component_name = parts[0].to_string();
        let pin_number = parts[1].to_string();

        pins.push(Pin2 {
            component_name,
            pin_number,
        });
    }

    Ok(Net {
        name: net_name,
        pins,
    })
}

fn parse_netclass(s_expr: &Vec<SExpr>) -> Result<Netclass, String> {
    let first_item = s_expr
        .first()
        .ok_or("Expected at least one item in the netclass scope")?;
    let first_item = first_item
        .as_atom()
        .ok_or("Expected an atom as the first item in the netclass scope")?;
    if first_item != "class" {
        return Err(format!(
            "Expected 'class' as the first item, found: {}",
            first_item
        ));
    }

    let net_class_name = s_expr
        .get(1)
        .ok_or("Expected net class name as the second item")?
        .as_atom()
        .ok_or("Expected net class name to be an atom")?
        .to_string();

    let mut net_names: Vec<String> = Vec::new();
    let mut current_pos = 2;
    while current_pos < s_expr.len() {
        match s_expr.get(current_pos) {
            Some(SExpr::Atom(name)) => {
                net_names.push(name.to_string());
                current_pos += 1;
            }
            Some(SExpr::List(_)) => break,
            None => break,
            _ => return Err("Unexpected non-atom in net names".into()),
        }
    }

    let mut via_name = String::new();
    let mut width: Option<f32> = None;
    let mut clearance: Option<f32> = None;
    for item in s_expr.iter().skip(current_pos) {
        if let SExpr::List(list) = item {
            match list.first().and_then(|x| x.as_atom()).map(|s| s.as_str()) {
                Some("circuit") => {
                    if let Some(use_via) = list.get(1) {
                        if let Some(use_via_list) = use_via.as_list() {
                            if use_via_list
                                .first()
                                .and_then(|x| x.as_atom())
                                .map(|s| s.as_str())
                                == Some("use_via")
                            {
                                via_name = use_via_list
                                    .get(1)
                                    .ok_or("Missing via name in use_via")?
                                    .as_atom()
                                    .ok_or("Via name must be an atom")?
                                    .to_string();
                            }
                        }
                    }
                }
                Some("rule") => {
                    for rule_item in list.iter().skip(1) {
                        if let Some(rule_list) = rule_item.as_list() {
                            match rule_list
                                .first()
                                .and_then(|x| x.as_atom())
                                .map(|s| s.as_str())
                            {
                                Some("width") => {
                                    let temp_width = rule_list
                                        .get(1)
                                        .ok_or("Missing width value")?
                                        .as_atom()
                                        .ok_or("Width must be a number")?
                                        .parse::<f32>()
                                        .map_err(|e| format!("Invalid width: {}", e))?;
                                    width = Some(temp_width);
                                }
                                Some("clearance") => {
                                    let temp_clearance = rule_list
                                        .get(1)
                                        .ok_or("Missing clearance value")?
                                        .as_atom()
                                        .ok_or("Clearance must be a number")?
                                        .parse::<f32>()
                                        .map_err(|e| format!("Invalid clearance: {}", e))?;
                                    clearance = Some(temp_clearance);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let width = width.ok_or("Missing required field: width")?;
    let clearance = clearance.ok_or("Missing required field: clearance")?;
    Ok(Netclass {
        net_class_name,
        net_names,
        via_name,
        width,
        clearance,
    })
}

fn parse_network(s_expr: &Vec<SExpr>) -> Result<Network, String> {
    // Placeholder for network parsing logic
    // This function should parse the network part of the S-expression
    // and populate the DsnStruct accordingly.

    let first_item = s_expr
        .first()
        .ok_or("Expected at least one item in the network scope")?;
    let first_item = first_item
        .as_atom()
        .ok_or("Expected an atom as the first item in the network scope")?;
    if first_item != "network" {
        return Err(format!(
            "Expected 'network' as the first item, found: {}",
            first_item
        ));
    }

    let mut nets: Vec<Net> = Vec::new();
    let mut netclasses: HashMap<String, Netclass> = HashMap::new();

    for item in s_expr.iter().skip(1) {
        let expr_list = item.as_list().ok_or(format!(
            "Expected a list in the network scope, found: {:?}",
            item
        ))?;
        let first_item = expr_list
            .first()
            .ok_or("Expected at least one item in the network item")?;
        let first_item = first_item
            .as_atom()
            .ok_or("Expected an atom as the first item in the network item")?;
        match first_item.as_str() {
            "net" => {
                let net = parse_net(expr_list)?;
                nets.push(net);
            }
            "class" => {
                let netclass = parse_netclass(expr_list)?;
                netclasses.insert(netclass.net_class_name.clone(), netclass);
            }
            _ => {
                return Err(format!("Unknown network item: {}", first_item));
            }
        }
    }

    Ok(Network { nets, netclasses })
}

pub fn parse_s_expr_to_struct(s_expr: &SExpr) -> Result<DsnStruct, String> {
    let mut resolution: Option<Resolution> = None;
    let mut structure: Option<Structure> = None;
    let mut placement: Option<Placement> = None;
    let mut library: Option<Library> = None;
    let mut network: Option<Network> = None;
    let expr_list = s_expr.as_list().ok_or("Expected a list at the top level")?;
    for expr in expr_list {
        let expr_list2 = match expr {
            SExpr::List(list) => list,
            _ => continue,
        };
        let first_item = expr_list2
            .first()
            .ok_or(format!("Expected at least one item in the outermost scope"))?;
        let first_item = first_item.as_atom().ok_or(format!(
            "Expected an atom as the first item in the outermost scope"
        ))?;
        match first_item.as_str() {
            "parser" => {
                continue;
            }
            "resolution" => {
                let second_item = expr_list2
                    .get(1)
                    .ok_or(format!("Expected a second item in the resolution scope"))?;
                let third_item = expr_list2
                    .get(2)
                    .ok_or(format!("Expected a third item in the resolution scope"))?;
                let second_item = second_item.as_atom().ok_or(format!(
                    "Expected an atom as the second item in the resolution scope"
                ))?;
                let third_item = third_item.as_atom().ok_or(format!(
                    "Expected an atom as the third item in the resolution scope"
                ))?;
                let value = third_item
                    .parse::<f64>()
                    .map_err(|e| format!("Failed to parse resolution value: {}", e))?;
                resolution = Some(Resolution {
                    unit: second_item.to_string(),
                    value,
                });
            }
            "unit" => {
                continue;
            }
            "structure" => {
                structure = Some(parse_structure(expr_list2)?);
            }
            "placement" => {
                placement = Some(parse_placement(expr_list2)?);
            }
            "library" => {
                library = Some(parse_library(expr_list2)?);
            }
            "network" => {
                network = Some(parse_network(expr_list2)?);
            }
            "wiring" => {
                continue;
            }
            _ => {
                return Err(format!("Unknown S-expression type: {}", first_item));
            }
        }
    }
    Ok(DsnStruct {
        resolution: resolution.ok_or("Missing required field: resolution")?,
        structure: structure.ok_or("Missing required field: structure")?,
        placement: placement.ok_or("Missing required field: placement")?,
        library: library.ok_or("Missing required field: library")?,
        network: network.ok_or("Missing required field: network")?,
    })
}
