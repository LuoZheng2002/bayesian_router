use std::collections::HashMap;

use cgmath::Deg;
use shared::vec2::FloatVec2;

pub struct Resolution {
    pub unit: String,
    pub value: f64,
}

pub struct Layer {
    pub name: String,
}

pub struct Boundary(pub Vec<FloatVec2>);

pub struct Structure {
    pub layers: Vec<Layer>,
    pub boundary: Boundary,
}
pub enum PlacementLayer {
    Front,
    Back,
}
impl PlacementLayer {
    const FRONT_STR: &'static str = "front";
    const BACK_STR: &'static str = "back";

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Front => Self::FRONT_STR,
            Self::Back => Self::BACK_STR,
        }
    }
}

pub struct ComponentInst {
    pub reference: String,
    pub position: FloatVec2,
    pub rotation: f32,
    pub placement_layer: PlacementLayer, // Layer where the component is placed
}
pub struct Component {
    pub name: String,
    pub instances: Vec<ComponentInst>,
}

pub struct Placement {
    pub components: Vec<Component>,
}

pub struct Pin {
    pub pad_stack_name: String,
    pub pin_number: String,
    pub position: FloatVec2,
    pub rotation: Deg<f32>,
}

pub struct Image {
    pub name: String,
    pub pins: HashMap<String, Pin>,
}
pub enum Shape {
    Circle {
        diameter: f32,
    },
    Rect {
        x_min: f32,
        y_min: f32,
        x_max: f32,
        y_max: f32,
    },
    Polygon {
        aperture_width: f32,
        vertices: Vec<FloatVec2>,
    },
}
pub struct PadStack {
    pub name: String,
    pub shape: Shape,
    pub through_hole: bool,
}

pub struct Library {
    pub images: HashMap<String, Image>,
    pub pad_stacks: HashMap<String, PadStack>,
}

pub struct Netclass {
    pub net_class_name: String,
    pub net_names: Vec<String>,
    pub via_name: String,
    pub width: f32,
    pub clearance: f32,
}

pub struct Pin2 {
    pub component_name: String,
    pub pin_number: String,
}

pub struct Net {
    pub name: String,
    pub pins: Vec<Pin2>,
}

pub struct Network {
    pub nets: Vec<Net>,
    pub netclasses: HashMap<String, Netclass>,
}

pub struct DsnStruct {
    pub resolution: Resolution,
    pub structure: Structure,
    pub placement: Placement,
    pub library: Library,
    pub network: Network,
}

impl DsnStruct {
    pub fn get_layer_names(&self) -> Vec<String> {
        self.structure
            .layers
            .iter()
            .map(|layer| layer.name.clone())
            .collect()
    }
}
