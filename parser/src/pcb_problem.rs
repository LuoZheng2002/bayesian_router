use core::panic;
use std::{
    cell::RefCell,
    collections::{BTreeSet, BinaryHeap, HashMap, HashSet},
    num::NonZeroUsize,
    rc::Rc,
    sync::{Arc, Mutex},
};

use ordered_float::NotNan;
use shared::color_float3::ColorFloat3;

use crate::{distinct_color_generator::{DistinctColorGenerator}, pad::Pad, shapes::{Line, Polygon}};


// use shared::interface_types::{Color, ColorGrid};

// use crate::{grid::Point, hyperparameters::{HALF_PROBABILITY_RAW_SCORE, ITERATION_TO_PRIOR_PROBABILITY, LENGTH_PENALTY_RATE, TURN_PENALTY_RATE}};

#[derive(Debug, Clone)]
pub struct Connection {
    pub net_name: NetName,               // The net that the connection belongs to
    pub connection_id: ConnectionID, // Unique identifier for the connection    
    pub sink: Pad,
    pub sink_trace_width: f32, // Width of the trace
    pub sink_trace_clearance: f32, // Clearance around the trace
}

#[derive(Debug, Clone)]
pub struct NetInfo {
    pub net_name: NetName,
    pub color: ColorFloat3,                                   // Color of the net
    pub source: Pad,
    pub source_trace_width: f32, // Width of the trace from the source pad
    pub source_trace_clearance: f32, // Clearance around the trace from the source pad
    pub connections: HashMap<ConnectionID, Rc<Connection>>, // List of connections in the net, the source pad is the same
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct NetName(pub String);
#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct NetClassName(pub String);
#[derive(Copy, Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct ConnectionID(pub usize);


/// use new, add_net, add_connection to construct this struct
pub struct PcbProblem {
    width: f32, // in specctra dsn units
    height: f32, // in specctra dsn units
    center: (f32, f32), // Center of the PCB, in specctra dsn units
    obstacle_lines: Vec<Line>, // Lines that represent obstacles in the PCB
    obstacle_polygons: Vec<Polygon>, // Polygons that represent obstacles in the PCB
    nets: HashMap<NetName, NetInfo>, // NetID to NetInfo
    connection_id_generator: Box<dyn Iterator<Item = ConnectionID> + Send + 'static>, // A generator for ConnectionID, starting from 0
    distinct_color_generator: Box<dyn Iterator<Item = ColorFloat3> + Send + 'static>, // A generator for distinct colors
}


impl PcbProblem {
    pub fn new(width: f32, height: f32, center: (f32, f32)) -> Self {
        PcbProblem {
            width,
            height,
            center,
            obstacle_lines: Vec::new(),
            obstacle_polygons: Vec::new(),
            nets: HashMap::new(),
            connection_id_generator: Box::new((0..).map(ConnectionID)),
            distinct_color_generator: Box::new(DistinctColorGenerator::new()),
        }
    }
    pub fn add_net(&mut self, net_name: NetName, source: Pad, source_trace_width: f32, source_trace_clearance: f32) {
        assert!(!self.nets.contains_key(&net_name), "NetID already exists: {}", net_name.0);
        let color = self.distinct_color_generator.next().expect("Distinct color generator exhausted");
        let net_info = NetInfo {
            net_name: net_name.clone(),
            color,
            connections: HashMap::new(),
            source,
            source_trace_width,
            source_trace_clearance,
        };
        self.nets.insert(net_name, net_info);
    }
    /// assert the sources in the same net are the same
    pub fn add_connection(&mut self, net_name: NetName, sink: Pad, trace_width: f32, trace_clearance: f32) -> ConnectionID {
        let net_info = self.nets.get_mut(&net_name).expect("NetID not found");
        let connection_id = self
            .connection_id_generator
            .next()
            .expect("ConnectionID generator exhausted");
        let connection = Connection {
            net_name,
            connection_id,
            sink,
            sink_trace_width: trace_width,
            sink_trace_clearance: trace_clearance,
        };
        net_info.connections.insert(connection_id, Rc::new(connection));
        connection_id
    }
}
