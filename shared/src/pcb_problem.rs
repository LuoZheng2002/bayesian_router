use std::{collections::HashMap, rc::Rc};

use crate::{
    collider::{BorderCollider, PolygonCollider},
    color_float3::ColorFloat3,
    distinct_color_generator::DistinctColorGenerator,
    pad::{Pad, PadName},
    prim_shape::Line,
    trace_path::TracePath,
    vec2::FloatVec2,
};

// use shared::interface_types::{Color, ColorGrid};

// use crate::{grid::Point, hyperparameters::{HALF_PROBABILITY_RAW_SCORE, ITERATION_TO_PRIOR_PROBABILITY, LENGTH_PENALTY_RATE, TURN_PENALTY_RATE}};

#[derive(Debug, Clone)]
pub struct Connection {
    pub net_name: NetName,           // The net that the connection belongs to
    pub connection_id: ConnectionID, // Unique identifier for the connection
    pub start_pad: PadName,
    pub end_pad: PadName,
}

#[derive(Debug, Clone)]
pub struct NetInfo {
    pub net_name: NetName,
    pub color: ColorFloat3,
    pub pads: HashMap<PadName, Pad>,
    pub trace_width: f32,     // Width of the trace from the source pad
    pub trace_clearance: f32, // Clearance around the trace from the source pad
    pub via_diameter: f32, // Diameter of the via, obtained from via name and accessed through padstacks
    pub connections: HashMap<ConnectionID, Rc<Connection>>, // List of connections in the net, the source pad is the same
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct NetName(pub String);
#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct NetClassName(pub String);
#[derive(Copy, Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct ConnectionID(pub usize);

// backtrack search:
// each node contains trace candidates, their rankings, and already determined traces
// coarse mode: sample multiple traces at a time
// fine mode: change the model immediately when one trace is determined

// separate the problem, the probabilistic model, and the solution

// (0, 0) center, up, right
pub struct PcbProblem {
    pub width: f32,
    pub height: f32,
    pub center: FloatVec2,
    pub num_layers: usize,                     // 0: front, num_layers - 1: back
    pub obstacle_borders: Vec<BorderCollider>, // Borders that represent obstacles in the PCB
    pub obstacle_border_outlines: Vec<Line>,   // Outlines of the borders, used for rendering
    pub obstacle_polygons: Vec<PolygonCollider>, // Polygons that represent obstacles in the PCB
    pub nets: HashMap<NetName, NetInfo>,       // NetID to NetInfo
    pub scale_down_factor: f32, // Scale down factor to convert specctra dsn units to float units
}

#[derive(Debug, Clone)]
pub struct FixedTrace {
    pub net_name: NetName,           // The net that the trace belongs to
    pub connection_id: ConnectionID, // The connection that the trace belongs to
    pub trace_path: TracePath,
}

pub struct PcbSolution {
    pub determined_traces: HashMap<ConnectionID, FixedTrace>, // NetID to ConnectionID to FixedTrace
    pub scale_down_factor: f32, // Scale down factor to convert specctra dsn units to float units
}

impl PcbProblem {
    // pub fn new(
    //     width: f32,
    //     height: f32,
    //     center: FloatVec2,
    //     num_layers: usize,
    //     scale_down_factor: f32,
    // ) -> Self {
    //     PcbProblem {
    //         width,
    //         height,
    //         center,
    //         num_layers,
    //         obstacle_borders: Vec::new(),
    //         obstacle_border_outlines: Vec::new(),
    //         obstacle_polygons: Vec::new(),
    //         nets: HashMap::new(),
    //         connection_id_generator: Box::new((0..).map(ConnectionID)),
    //         distinct_color_generator: Box::new(DistinctColorGenerator::new()),
    //         scale_down_factor,
    //     }
    // }
    // pub fn add_net(
    //     &mut self,
    //     net_name: NetName,
    //     source: Pad,
    //     source_trace_width: f32,
    //     source_trace_clearance: f32,
    //     via_diameter: f32,
    // ) {
    //     assert!(
    //         !self.nets.contains_key(&net_name),
    //         "NetID already exists: {}",
    //         net_name.0
    //     );
    //     let color = self
    //         .distinct_color_generator
    //         .next()
    //         .expect("Distinct color generator exhausted");
    //     let net_info = NetInfo {
    //         net_name: net_name.clone(),
    //         color,
    //         connections: HashMap::new(),
    //         source,
    //         trace_width: source_trace_width,
    //         trace_clearance: source_trace_clearance,
    //         via_diameter,
    //     };
    //     self.nets.insert(net_name, net_info);
    // }
    // /// assert the sources in the same net are the same
    // pub fn add_connection(
    //     &mut self,
    //     net_name: NetName,
    //     sink: Pad,
    //     trace_width: f32,
    //     trace_clearance: f32,
    // ) -> ConnectionID {
    //     let net_info = self.nets.get_mut(&net_name).expect("NetID not found");
    //     let connection_id = self
    //         .connection_id_generator
    //         .next()
    //         .expect("ConnectionID generator exhausted");
    //     let connection = Connection {
    //         net_name,
    //         connection_id,
    //         sink,
    //         sink_trace_width: trace_width,
    //         sink_trace_clearance: trace_clearance,
    //     };
    //     net_info
    //         .connections
    //         .insert(connection_id, Rc::new(connection));
    //     connection_id
    // }
}
