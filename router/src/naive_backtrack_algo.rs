use std::{cell::RefCell, cmp::Reverse, collections::{BinaryHeap, HashMap, VecDeque}, hash::Hash, rc::Rc, sync::{Arc, Mutex}};

use ordered_float::NotNan;
use shared::{binary_heap_item::BinaryHeapItem, pad::{Pad, PadName}, pcb_problem::{Connection, ConnectionID, FixedTrace, NetInfo, PcbProblem, PcbSolution}, pcb_render_model::PcbRenderModel, trace_path::TracePath};

use crate::astar::{self, AStarModel};



pub struct NaiveBacktrackNode{
    pub current_connection: Option<ConnectionID>,
    pub alternative_connections: VecDeque<ConnectionID>,
    pub fixed_connections: HashMap<ConnectionID, TracePath>,
}
impl NaiveBacktrackNode{
    pub fn new_empty(all_ordered_connections: &Vec<ConnectionID>) -> Self{
        assert!(!all_ordered_connections.is_empty(), "There must be at least one connection to start with");
        let alternative_connections = all_ordered_connections.iter().cloned().collect::<VecDeque<_>>();
        NaiveBacktrackNode {
            current_connection: None,
            alternative_connections,
            fixed_connections: HashMap::new(),
        }
    }
    pub fn add_fixed_connection(&self, connection: ConnectionID, trace: TracePath) -> Self {
        assert!(connection == self.current_connection.unwrap(), "Cannot add a fixed connection that is not the current connection");
        let mut fixed_connections = self.fixed_connections.clone();
        fixed_connections.insert(connection, trace);
        NaiveBacktrackNode {
            fixed_connections,
            current_connection: None,
            alternative_connections: self.alternative_connections.clone(),
        }
    }
}

pub fn naive_backtrack(pcb_problem: &PcbProblem, pcb_render_model: Arc<Mutex<Option<PcbRenderModel>>>) -> Result<PcbSolution, String> {
    let mut connection_to_length: HashMap<ConnectionID, NotNan<f32>> = HashMap::new();
    for net_info in pcb_problem.nets.values() {

        for connection in net_info.connections.values() {
            let start_pad = net_info.pads.get(&connection.start_pad).unwrap();
            let end_pad = net_info.pads.get(&connection.end_pad).unwrap();
            let start = start_pad.position.to_fixed().to_nearest_even_even();
            let end = end_pad.position.to_fixed().to_nearest_even_even();
            let start_layers = start_pad.pad_layer;
            let end_layers = end_pad.pad_layer;
            let astar_model = AStarModel {
                start,
                end,
                start_layers,
                end_layers,
                num_layers: pcb_problem.num_layers,
                trace_width: net_info.trace_width,
                trace_clearance: net_info.trace_clearance,
                via_diameter: net_info.via_diameter,
                width: pcb_problem.width,
                height: pcb_problem.height,
                center: pcb_problem.center,
                obstacle_shapes: ,
                obstacle_clearance_shapes: Vec::new(),
                obstacle_colliders: Vec::new(),
                obstacle_clearance_colliders: Vec::new(),
                border_colliders_cache: RefCell::new(None),
                border_shapes_cache: RefCell::new(None),
            };
            let result = astar_model.run(pcb_render_model.clone());
            let result = match result{
                Ok(result) => result,
                Err(e) => {
                    println!("A star algorithm failed");
                    panic!("A star algorithm failed");
                }
            };
            connection_to_length.insert(connection.connection_id, NotNan::new(result.trace_path.total_length as f32).unwrap());
        }
    }
    let connection_heap: BinaryHeap<BinaryHeapItem<Reverse<NotNan<f32>>, ConnectionID>> = BinaryHeap::new();
    for (connection_id, length) in connection_to_length.iter() {
        connection_heap.push(BinaryHeapItem::new(Reverse(*length), *connection_id));
    }
    let ordered_connection_vec: Vec<ConnectionID> = connection_heap.drain().map(|item| item.value).collect();

    let backtrack_stack: Vec<NaiveBacktrackNode> = Vec::new();

    let root_node = NaiveBacktrackNode::new_empty(&ordered_connection_vec);
    backtrack_stack.push(root_node);

    let connections: HashMap<ConnectionID, Rc<Connection>> = pcb_problem.nets.values()
        .flat_map(|net_info| net_info.connections.iter())
        .map(|(id, connection)| (*id, connection.clone()))
        .collect();
    let pads: HashMap<PadName, &Pad> = pcb_problem.nets.values()
        .flat_map(|net_info| net_info.pads.iter())
        .map(|(name, pad)| (name.clone(), pad))
        .collect();
    let connection_to_net_info: HashMap<ConnectionID, &NetInfo> = pcb_problem.nets.iter()
        .flat_map(|(_, net_info)| net_info.connections.iter().map(|(id, _)| (*id, net_info)))
        .collect();

    // dfs
    while !backtrack_stack.is_empty() {
        // Get the top node from the stack
        let top_node = backtrack_stack.last_mut().unwrap();
        assert!(top_node.current_connection.is_none());
        if top_node.alternative_connections.is_empty() {
            // is solution
            let fixed_connections = std::mem::take(&mut top_node.fixed_connections);
            let fixed_traces: HashMap<ConnectionID, FixedTrace> = fixed_connections.into_iter()
                .map(|(connection_id, trace_path)| {
                    let fixed_trace = FixedTrace {
                        net_name: connections.get(&connection_id).unwrap().net_name.clone(),
                        connection_id: connection_id,
                        trace_path: trace_path.clone(),
                    };
                    (connection_id, fixed_trace)
                })
                .collect();
            let pcb_solution = PcbSolution{
                determined_traces: fixed_traces,
                scale_down_factor: pcb_problem.scale_down_factor,
            };
            return Ok(pcb_solution);
        }
        top_node.current_connection = Some(top_node.alternative_connections.pop_front().unwrap());
        
        // let is_current_connection_valid: bool = todo!();

        let connection = connections.get(&top_node.current_connection).unwrap();
        let start_pad = pads.get(&connection.start_pad).unwrap();
        let end_pad = pads.get(&connection.end_pad).unwrap();
        let start = start_pad.position.to_fixed().to_nearest_even_even();
        let end = end_pad.position.to_fixed().to_nearest_even_even();
        let start_layers = start_pad.pad_layer;
        let end_layers = end_pad.pad_layer;
        let net_info = connection_to_net_info.get(&connection.connection_id).unwrap();
        let astar_model = AStarModel {
            start,
            end,
            start_layers,
            end_layers,
            num_layers: pcb_problem.num_layers,
            trace_width: net_info.trace_width,
            trace_clearance: net_info.trace_clearance,
            via_diameter: net_info.via_diameter,
            width: pcb_problem.width,
            height: pcb_problem.height,
            center: pcb_problem.center,
            obstacle_shapes: None,
            obstacle_clearance_shapes: None,
            obstacle_colliders: None,
            obstacle_clearance_colliders: None,
            border_colliders_cache: RefCell::new(None),
            border_shapes_cache: RefCell::new(None),
        };
        let result = astar_model.run(pcb_render_model.clone());
        let result = match result {
            Ok(result) => result,
            Err(e) => {
                println!("Cannot find a path for connection {:?}, popping node", connection.connection_id);
                backtrack_stack.pop();
                continue;
            }
        };
        if 

        let mut new_fixed_traces: HashMap<ConnectionID, TracePath> = todo!();


        // Check if the top node is a solution
        if top_node.is_solution(pcb_problem) {
            println!("Found a solution!");
            // If the top node is a solution, we can return it
            let fixed_traces = top_node.fixed_connections.clone();
            let solution = PcbSolution {
                fixed_traces,
                pcb_render_model: pcb_render_model.clone(),
            };
            return Ok(solution);
        }

        // Expand the current node
        let next_nodes = top_node.expand(pcb_problem);

        if next_nodes.is_empty() {
            // If there are no next nodes, we backtrack by popping the current node
            node_stack.pop();
        } else {
            // Otherwise, we push the next nodes onto the stack
            for next_node in next_nodes {
                node_stack.push(next_node);
            }
        }
    }

    Err("No solution found".to_string())
}