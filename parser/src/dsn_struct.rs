use std::collections::HashMap;

use shared::vec2::FloatVec2;



pub struct Resolution{
    pub unit: String,
    pub value: f64,
}


pub struct Layer{
    pub name: String,
}

pub struct Boundary(pub Vec<(f64, f64)>);

pub struct Structure{
    pub layers: Vec<Layer>,
    pub boundary: Boundary,
}

pub struct ComponentInst{
    pub reference: String,
    pub position: FloatVec2,
    pub rotation: f32,
}
pub struct Component{
    pub name: String,
    pub instances: Vec<ComponentInst>,
}

pub struct Placement{
    pub components: Vec<Component>,
}

pub struct Pin{
    pub pad_stack_name: String,
    pub pin_number: usize,
    pub position: FloatVec2,
}

pub struct Image{
    pub name: String,
    pub pins: HashMap<usize, Pin>,
}
pub enum Shape{
    Circle{
        diameter: f64,
    },
    Rect{
        x_min: f64,
        y_min: f64,
        x_max: f64,
        y_max: f64,
    },
    Polygon{
        aperture_width: f64,
        vertices: Vec<FloatVec2>,
    }
}
pub struct PadStack{
    pub name: String,
    pub shape: Shape,
    pub through_hole: bool,
}

pub struct Library{
    pub images: HashMap<String, Image>,
    pub pad_stacks: HashMap<String, PadStack>,
}

pub struct Netclass{
    pub net_class_name: String,
    pub net_names: Vec<String>,
    pub via_name: String,
    pub width: f64,
    pub clearance: f64,
}

pub struct Pin2{
    pub component_name: String,
    pub pin_number: usize,
}

pub struct Net{
    pub name: String,
    pub pins: Vec<Pin2>,
}

pub struct Network{
    pub nets: Vec<Net>,
    pub netclasses: HashMap<String, Netclass>,
}

pub struct DsnStruct{
    pub resolution: Resolution,
    pub structure: Structure,
    pub placement: Placement,
    pub library: Library,
    pub network: Network,
}