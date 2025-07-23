use std::collections::HashMap;

use shared::{
    collider::PolygonCollider,
    pad::{Pad, PadName},
    pcb_problem::{NetClassName, NetName},
    prim_shape::Line,
    vec2::FloatVec2,
};

pub struct DisplayNetInfo {
    pub net_name: NetName,
    pub pads: Vec<Pad>, // including source and sink pads, and let the user decide which one is the source.
    // netclass settings
    pub net_class_name: NetClassName,
    // unwrap netclass information to each net for convenience
    pub default_trace_width: f32, // may be overridden by individual pads in the next pass
    pub default_trace_clearance: f32, // may be overridden by individual pads in the next pass
    pub via_diameter: f32,        // obtained from via name, and accessed through padstacks
}

pub struct DisplayFormat {
    pub width: f32,                              // in specctra dsn units
    pub height: f32,                             // in specctra dsn units
    pub center: FloatVec2,                       // Center of the PCB, in specctra dsn units
    pub num_layers: usize,                       // 0: front, num_layers - 1: back
    pub obstacle_lines: Vec<Line>,               // Lines that represent obstacles in the PCB
    pub obstacle_polygons: Vec<PolygonCollider>, // Polygons that represent obstacles in the PCB
    pub nets: HashMap<NetName, DisplayNetInfo>,  // NetID to DisplayNetInfo
    pub scale_down_factor: f32, // Scale down factor to convert specctra dsn units to float units
}

pub struct ExtraInfo {
    // for nets with 3 or more pads, choose the pad specified below as the source pad. If it's not specified, generate a warning and choose the first one.
    pub net_name_to_source_pad: HashMap<NetName, PadName>, // net name to source pad name
}
