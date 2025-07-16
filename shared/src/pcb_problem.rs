use std::{
    collections::HashMap,
    rc::Rc,
};

use crate::{
    color_float3::ColorFloat3, pad::Pad, trace_path::TracePath
};

// use shared::interface_types::{Color, ColorGrid};

// use crate::{grid::Point, hyperparameters::{HALF_PROBABILITY_RAW_SCORE, ITERATION_TO_PRIOR_PROBABILITY, LENGTH_PENALTY_RATE, TURN_PENALTY_RATE}};

#[derive(Debug, Clone)]
pub struct Connection {
    pub net_name: NetName,               // The net that the connection belongs to
    pub connection_id: ConnectionID, // Unique identifier for the connection
    pub source: Pad,
    pub sink: Pad,
    pub trace_width: f32, // Width of the trace
    pub trace_clearance: f32, // Clearance around the trace
                          // pub traces: HashMap<TraceID, TraceInfo>, // List of traces connecting the source and sink pads
}

#[derive(Debug, Clone)]
pub struct NetInfo {
    pub net_id: NetName,
    pub color: ColorFloat3,                                       // Color of the net
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
    pub nets: HashMap<NetName, NetInfo>, // NetID to NetInfo
    pub net_id_generator: Box<dyn Iterator<Item = NetName> + Send + 'static>, // A generator for NetID, starting from 0
    pub connection_id_generator: Box<dyn Iterator<Item = ConnectionID> + Send + 'static>, // A generator for ConnectionID, starting from 0
}


#[derive(Debug, Clone)]
pub struct FixedTrace {
    pub net_id: NetName,               // The net that the trace belongs to
    pub connection_id: ConnectionID, // The connection that the trace belongs to
    pub trace_path: TracePath,
}

pub struct PcbSolution {
    pub determined_traces: HashMap<ConnectionID, FixedTrace>, // NetID to ConnectionID to FixedTrace
}

impl PcbProblem {
    pub fn new(width: f32, height: f32) -> Self {
        let net_id_generator = Box::new((0..).map(NetName));
        let connection_id_generator = Box::new((0..).map(ConnectionID));
        PcbProblem {
            width,
            height,
            nets: HashMap::new(),
            net_id_generator,
            connection_id_generator,
        }
    }
    pub fn add_net(&mut self, color: ColorFloat3) -> NetName {
        
        let net_id = self
            .net_id_generator
            .next()
            .expect("NetID generator exhausted");
        let net_info = NetInfo {
            net_id,
            color,
            connections: HashMap::new(),
        };
        self.nets.insert(net_id, net_info);
        net_id
    }
    /// assert the sources in the same net are the same
    pub fn add_connection(
        &mut self,
        net_id: NetName,
        source: Pad,
        sink: Pad,
        trace_width: f32,
        trace_clearance: f32,
    ) -> ConnectionID {
        let net_info = self.nets.get_mut(&net_id).expect("NetID not found");
        let connection_id = self
            .connection_id_generator
            .next()
            .expect("ConnectionID generator exhausted");
        let connection = Connection {
            net_name: net_id,
            connection_id,
            source,
            sink,
            trace_width,
            trace_clearance,
        };
        net_info
            .connections
            .insert(connection_id, Rc::new(connection));
        connection_id
    }

    
}
