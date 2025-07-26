use std::{cell::RefCell, cmp::Reverse, collections::{BinaryHeap, HashMap, VecDeque}, hash::Hash, rc::Rc, sync::{atomic::Ordering, Arc, Mutex}, thread, time::Duration};

use ordered_float::NotNan;
use shared::{binary_heap_item::BinaryHeapItem, collider::Collider, color_float3::ColorFloat3, hyperparameters::SAMPLE_CNT, pad::{Pad, PadName}, pcb_problem::{Connection, ConnectionID, FixedTrace, NetInfo, NetName, PcbProblem, PcbSolution}, pcb_render_model::{PcbRenderModel, RenderableBatch, ShapeRenderable}, prim_shape::PrimShape, trace_path::{self, TracePath}};

use crate::{astar::{self, AStarModel}, astar_check_struct::AStarCheck, bayesian_backtrack_algo::TraceCache, command_flags::{CommandFlag, COMMAND_CVS, COMMAND_LEVEL, COMMAND_MUTEXES}, quad_tree::QuadTreeNode};



pub struct NaiveBacktrackNode{
    pub current_connection: Option<ConnectionID>,
    pub alternative_connections: VecDeque<ConnectionID>,
    pub failed_connections: Vec<ConnectionID>, // connections that fail to become current_connection
    pub fixed_connections: HashMap<ConnectionID, FixedTrace>,
}
impl NaiveBacktrackNode{
    pub fn new_empty(all_ordered_connections: &Vec<ConnectionID>) -> Self{
        assert!(!all_ordered_connections.is_empty(), "There must be at least one connection to start with");
        let alternative_connections = all_ordered_connections.iter().cloned().collect::<VecDeque<_>>();
        NaiveBacktrackNode {
            current_connection: None,
            alternative_connections,
            fixed_connections: HashMap::new(),
            failed_connections: Vec::new(),
        }
    }
    pub fn push_node(&mut self, connection: ConnectionID, fixed_trace: FixedTrace) -> Self {
        assert!(connection == self.current_connection.unwrap(), "Cannot add a fixed connection that is not the current connection");
        self.current_connection = None; // reset current connection        
        let mut new_fixed_connections = self.fixed_connections.clone();
        new_fixed_connections.insert(connection, fixed_trace);
        let mut new_alternative_connections = self.alternative_connections.clone();
        for failed_connection in self.failed_connections.iter() {
            new_alternative_connections.push_front(*failed_connection);
        }
        self.failed_connections.push(connection); // add the current connection to failed connections
        NaiveBacktrackNode {
            fixed_connections: new_fixed_connections,
            current_connection: None,
            alternative_connections: new_alternative_connections,
            failed_connections: Vec::new(), // reset failed connections
        }
    }
}

fn node_to_pcb_render_model(problem: &PcbProblem, node: &NaiveBacktrackNode) -> PcbRenderModel {
    let mut trace_shape_renderables: Vec<RenderableBatch> = Vec::new();
    let mut pad_shape_renderables: Vec<ShapeRenderable> = Vec::new();
    let mut other_shape_renderables: Vec<ShapeRenderable> = Vec::new();
    let mut net_name_to_color: HashMap<NetName, ColorFloat3> = HashMap::new();
    for (_, net_info) in problem.nets.iter() {
        net_name_to_color.insert(net_info.net_name.clone(), net_info.color);
        for pad in net_info.pads.values(){
            let pad_renderables = pad.to_renderables(net_info.color.to_float4(1.0));
            let pad_clearance_renderables = pad.to_clearance_renderables(net_info.color.to_float4(0.5));
            pad_shape_renderables.extend(pad_renderables);
            pad_shape_renderables.extend(pad_clearance_renderables);
        }
    }
    for fixed_trace in node.fixed_connections.values() {
        let trace_path = &fixed_trace.trace_path;
        let renderable_batches = trace_path
            .to_renderables(net_name_to_color[&fixed_trace.net_name].to_float4(1.0));
        trace_shape_renderables.extend(renderable_batches);
    }
    for line in &problem.obstacle_border_outlines {
        other_shape_renderables.push(ShapeRenderable {
            shape: PrimShape::Line(line.clone()),
            color: [1.0, 0.0, 1.0, 1.0], // magenta color for borders
        });
    }
    PcbRenderModel {
        width: problem.width,
        height: problem.height,
        center: problem.center,
        trace_shape_renderables,
        pad_shape_renderables,
        other_shape_renderables,
    }
}

fn display_when_necessary(
    node: &NaiveBacktrackNode,
    pcb_problem: &PcbProblem,
    pcb_render_model: Arc<Mutex<Option<PcbRenderModel>>>,
) {
    let command_level = COMMAND_LEVEL.load(Ordering::Relaxed);
    {
        let mut pcb_render_model = pcb_render_model.lock().unwrap();
        if pcb_render_model.is_some() {
            return; // already rendered, no need to update
        }
        let render_model = node_to_pcb_render_model(pcb_problem, node);
        *pcb_render_model = Some(render_model);
    }
    if command_level <= CommandFlag::ProbaModelResult.get_level() {
        // block the thread until the user clicks a button
        {
            let mutex_guard = COMMAND_MUTEXES[3].lock().unwrap();
            let _unused = COMMAND_CVS[3].wait(mutex_guard).unwrap();
        }
    } else {
        thread::sleep(Duration::from_millis(0));
    }
}
pub fn naive_backtrack(problem: &PcbProblem, 
    pcb_render_model: Arc<Mutex<Option<PcbRenderModel>>>,
    trace_cache: &mut TraceCache,
    heuristics: Option<Vec<ConnectionID>>,
) -> Result<PcbSolution, String> {
    // prepare the obstacles for the first A* run    
    let border_colliders = AStarModel::calculate_border_colliders(problem.width, problem.height, problem.center);
        
    let quad_tree_side_length = f32::max(problem.width as f32, problem.height as f32);
        let quad_tree_x_min = problem.center.x as f32 - quad_tree_side_length / 2.0;
        let quad_tree_x_max = problem.center.x as f32 + quad_tree_side_length / 2.0;
        let quad_tree_y_min = problem.center.y as f32 - quad_tree_side_length / 2.0;
        let quad_tree_y_max = problem.center.y as f32 + quad_tree_side_length / 2.0;
    
    let ordered_connection_vec = if let Some(heuristics) = heuristics {
        heuristics
    } else {        
        let mut connection_to_length: HashMap<ConnectionID, NotNan<f32>> = HashMap::new();
        for (net_name, net_info) in problem.nets.iter() {
            // obstacles are pads
            let mut obstacle_shapes: HashMap<usize, Vec<PrimShape>> = (0..problem.num_layers)
                .map(|layer| (layer, Vec::new()))
                .collect();
            let mut obstacle_clearance_shapes: HashMap<usize, Vec<PrimShape>> = (0..problem
                .num_layers)
                .map(|layer| (layer, Vec::new()))
                .collect();
            let mut obstacle_colliders: HashMap<usize, QuadTreeNode> = (0..problem.num_layers)
                .map(|layer| {
                    (
                        layer,
                        QuadTreeNode::new(
                            quad_tree_x_min,
                            quad_tree_x_max,
                            quad_tree_y_min,
                            quad_tree_y_max,
                            0,
                        ),
                    )
                })
                .collect();
            let mut obstacle_clearance_colliders: HashMap<usize, QuadTreeNode> = (0..problem
                .num_layers)
                .map(|layer| {
                    (
                        layer,
                        QuadTreeNode::new(
                            quad_tree_x_min,
                            quad_tree_x_max,
                            quad_tree_y_min,
                            quad_tree_y_max,
                            0,
                        ),
                    )
                })
                .collect();
            for (_, net_info) in problem
                .nets
                .iter()
                .filter(|(other_net_id, _)| **other_net_id != *net_name)
            {
                for pad in net_info.pads.values(){
                    let pad_layers = pad.pad_layer.get_iter(problem.num_layers);
                    for layer in pad_layers{
                        let pad_shapes = pad.to_shapes();
                        let pad_clearance_shapes = pad.to_clearance_shapes();                    
                        for pad_shape in pad_shapes.iter() {
                            let pad_collider = Collider::from_prim_shape(pad_shape);
                            obstacle_colliders.get_mut(&layer).unwrap().insert(pad_collider);
                        }
                        for pad_clearance_shape in pad_clearance_shapes.iter() {
                            let pad_clearance_collider = Collider::from_prim_shape(pad_clearance_shape);
                            obstacle_clearance_colliders.get_mut(&layer).unwrap().insert(pad_clearance_collider);
                        }
                        obstacle_shapes.get_mut(&layer).unwrap().extend(pad_shapes);
                        obstacle_clearance_shapes.get_mut(&layer).unwrap().extend(pad_clearance_shapes);
                    }
                }
            }
            let obstacle_shapes = Rc::new(obstacle_shapes);
            let obstacle_clearance_shapes = Rc::new(obstacle_clearance_shapes);
            let obstacle_colliders = Rc::new(obstacle_colliders);
            let obstacle_clearance_colliders = Rc::new(obstacle_clearance_colliders);
            
            for connection in net_info.connections.values() {
                let mut trace_path: Option<TracePath> = None;
                let current_connection_trace_cache = trace_cache.traces.get_mut(&connection.connection_id).unwrap();
                for cache_trace_path in current_connection_trace_cache.iter() {
                    let astar_check = AStarCheck{
                        border_colliders: border_colliders.clone(),
                        obstacle_colliders: obstacle_colliders.clone(),
                        obstacle_clearance_colliders: obstacle_clearance_colliders.clone(),
                        solution_trace: cache_trace_path.clone(),
                        num_layers: problem.num_layers,
                    };
                    if astar_check.check() {
                        println!("Cache Hit!");                            
                        trace_path = Some(cache_trace_path.clone());
                        break; // we found a trace that satisfies the constraints, no need to generate a new one
                    }else{
                        println!("Cache Miss!");
                    }
                }
                let trace_path = if let Some(trace_path) = trace_path{
                    trace_path
                }else{
                    // run A* algorithm
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
                        num_layers: problem.num_layers,
                        trace_width: net_info.trace_width,
                        trace_clearance: net_info.trace_clearance,
                        via_diameter: net_info.via_diameter,
                        width: problem.width,
                        height: problem.height,
                        center: problem.center,
                        obstacle_shapes: obstacle_shapes.clone(),
                        obstacle_clearance_shapes: obstacle_clearance_shapes.clone(),
                        obstacle_colliders: obstacle_colliders.clone(),
                        obstacle_clearance_colliders: obstacle_clearance_colliders.clone(),
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
                    current_connection_trace_cache.push(result.trace_path.clone());
                    result.trace_path
                };
                connection_to_length.insert(connection.connection_id, NotNan::new(trace_path.total_length as f32).unwrap());
            }
        }
        let mut connection_heap: BinaryHeap<BinaryHeapItem<Reverse<NotNan<f32>>, ConnectionID>> = BinaryHeap::new();
        for (connection_id, length) in connection_to_length.iter() {
            connection_heap.push(BinaryHeapItem::new(Reverse(*length), *connection_id));
        }
        let ordered_connection_vec: Vec<ConnectionID> = connection_heap.drain().map(|item| item.value).collect();
        ordered_connection_vec
    };
    // SAMPLE_CNT.store(0, Ordering::Relaxed);
    let mut backtrack_stack: Vec<NaiveBacktrackNode> = Vec::new();

    let root_node = NaiveBacktrackNode::new_empty(&ordered_connection_vec);
    backtrack_stack.push(root_node);

    let connections: HashMap<ConnectionID, Rc<Connection>> = problem.nets.values()
        .flat_map(|net_info| net_info.connections.iter())
        .map(|(id, connection)| (*id, connection.clone()))
        .collect();
    let pads: HashMap<PadName, &Pad> = problem.nets.values()
        .flat_map(|net_info| net_info.pads.iter())
        .map(|(name, pad)| (name.clone(), pad))
        .collect();
    let connection_to_net_info: HashMap<ConnectionID, &NetInfo> = problem.nets.iter()
        .flat_map(|(net_name, net_info)| {
            net_info.connections.iter().map(move |(connection_id, _)| {
                (*connection_id, net_info)
            })
        })
        .collect();

    // dfs
    fn print_top_node(top_node: &NaiveBacktrackNode) {
        print!("Top node: fixed_connections: ");
        for connection_id in top_node.fixed_connections.keys() {
            print!("{},", connection_id.0);
        }
        print!("current connection: {:?}, ", top_node.current_connection);
        print!("alternative connections: ");
        for connection_id in top_node.alternative_connections.iter() {
            print!("{},", connection_id.0);
        }
        println!();
    }

    while !backtrack_stack.is_empty() {
        // Get the top node from the stack
        
        let top_node = backtrack_stack.last_mut().unwrap();
        print_top_node(top_node);
        assert!(top_node.current_connection.is_none());

        display_when_necessary(&top_node, &problem, pcb_render_model.clone());
        if top_node.alternative_connections.is_empty() {
            if !top_node.failed_connections.is_empty() {
                println!("No more alternative connections but have failed connections, fail to solve");
                return Err("Failed to solve PCB problem: No more alternative connections but have failed connections".to_string());
            }
            // is solution
            let fixed_connections = std::mem::take(&mut top_node.fixed_connections);
            let fixed_traces: HashMap<ConnectionID, FixedTrace> = fixed_connections.into_iter()
                .map(|(connection_id, fixed_trace)| {
 
                    (connection_id, fixed_trace)
                })
                .collect();
            let pcb_solution = PcbSolution{
                determined_traces: fixed_traces,
                scale_down_factor: problem.scale_down_factor,
            };
            return Ok(pcb_solution);
        }
        // select the next connection
        top_node.current_connection = Some(top_node.alternative_connections.pop_front().unwrap());
        let current_connection = top_node.current_connection.unwrap();
        
        // let is_current_connection_valid: bool = todo!();

        // here: prepare the obstacles for current connection with fixed traces
        let current_net_name = connections.get(&current_connection).unwrap().net_name.clone();
        let mut obstacle_shapes: HashMap<usize, Vec<PrimShape>> = (0..problem.num_layers)
            .map(|layer| (layer, Vec::new()))
            .collect();
        let mut obstacle_clearance_shapes: HashMap<usize, Vec<PrimShape>> = (0..problem
            .num_layers)
            .map(|layer| (layer, Vec::new()))
            .collect();
        let mut obstacle_colliders: HashMap<usize, QuadTreeNode> = (0..problem.num_layers)
            .map(|layer| {
                (
                    layer,
                    QuadTreeNode::new(
                        quad_tree_x_min,
                        quad_tree_x_max,
                        quad_tree_y_min,
                        quad_tree_y_max,
                        0,
                    ),
                )
            })
            .collect();
        let mut obstacle_clearance_colliders: HashMap<usize, QuadTreeNode> = (0..problem
            .num_layers)
            .map(|layer| {
                (
                    layer,
                    QuadTreeNode::new(
                        quad_tree_x_min,
                        quad_tree_x_max,
                        quad_tree_y_min,
                        quad_tree_y_max,
                        0,
                    ),
                )
            })
            .collect();
        // add all pads from other nets
        for (_, net_info) in problem
            .nets
            .iter()
            .filter(|(other_net_id, _)| **other_net_id != current_net_name)
        {
            for pad in net_info.pads.values(){
                let pad_layers = pad.pad_layer.get_iter(problem.num_layers);
                for layer in pad_layers{
                    let pad_shapes = pad.to_shapes();
                    let pad_clearance_shapes = pad.to_clearance_shapes();    
                    let pad_colliders = pad_shapes.iter()
                        .map(|shape| Collider::from_prim_shape(shape));
                    let pad_clearance_colliders = pad_clearance_shapes.iter()
                        .map(|shape| Collider::from_prim_shape(shape));
                    obstacle_colliders.get_mut(&layer).unwrap().extend(pad_colliders);
                    obstacle_clearance_colliders.get_mut(&layer).unwrap().extend(pad_clearance_colliders);
                    obstacle_shapes.get_mut(&layer).unwrap().extend(pad_shapes);
                    obstacle_clearance_shapes.get_mut(&layer).unwrap().extend(pad_clearance_shapes);                    
                }
            }
        }
        // add fixed traces
        for (connection_id, fixed_trace) in top_node.fixed_connections.iter(){
            let trace_path = &fixed_trace.trace_path;
            let connection_net_name = connections.get(connection_id).unwrap().net_name.clone();
            if current_net_name != connection_net_name {
                let trace_shapes = trace_path.to_shapes(problem.num_layers);
                let trace_clearance_shapes = trace_path.to_clearance_shapes(problem.num_layers);
                let trace_colliders = trace_path.to_colliders(problem.num_layers);
                let trace_clearance_colliders = trace_path.to_clearance_colliders(problem.num_layers);
                for layer in (0..problem.num_layers).into_iter() {
                    let shapes = trace_shapes.get(&layer).unwrap();
                    let clearance_shapes = trace_clearance_shapes.get(&layer).unwrap();
                    let colliders = trace_colliders.get(&layer).unwrap();
                    let clearance_colliders = trace_clearance_colliders.get(&layer).unwrap();
                    obstacle_shapes.get_mut(&layer).unwrap().extend(shapes.iter().cloned());
                    obstacle_clearance_shapes.get_mut(&layer).unwrap().extend(clearance_shapes.iter().cloned());
                    obstacle_colliders.get_mut(&layer).unwrap().extend(colliders.iter().cloned());
                    obstacle_clearance_colliders.get_mut(&layer).unwrap().extend(clearance_colliders.iter().cloned());
                }
            }            
        }
        let obstacle_shapes = Rc::new(obstacle_shapes);
        let obstacle_clearance_shapes = Rc::new(obstacle_clearance_shapes);
        let obstacle_colliders = Rc::new(obstacle_colliders);
        let obstacle_clearance_colliders = Rc::new(obstacle_clearance_colliders);


        // check cache first
        let mut trace_path: Option<TracePath> = None;
        let current_connection_trace_cache = trace_cache.traces.get_mut(&current_connection).unwrap();
        for cache_trace_path in current_connection_trace_cache.iter() {
            let astar_check = AStarCheck{
                border_colliders: border_colliders.clone(),
                obstacle_colliders: obstacle_colliders.clone(),
                obstacle_clearance_colliders: obstacle_clearance_colliders.clone(),
                solution_trace: cache_trace_path.clone(),
                num_layers: problem.num_layers,
            };
            if astar_check.check() {
                println!("Cache Hit!");                            
                trace_path = Some(cache_trace_path.clone());
                break; // we found a trace that satisfies the constraints, no need to generate a new one
            }else{
                println!("Cache Miss!");
            }
        }
        let connection = connections.get(&current_connection).unwrap();
        let trace_path = if let Some(trace_path) = trace_path{
            trace_path
        }else{            
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
                num_layers: problem.num_layers,
                trace_width: net_info.trace_width,
                trace_clearance: net_info.trace_clearance,
                via_diameter: net_info.via_diameter,
                width: problem.width,
                height: problem.height,
                center: problem.center,
                obstacle_shapes: obstacle_shapes.clone(),
                obstacle_clearance_shapes: obstacle_clearance_shapes.clone(),
                obstacle_colliders: obstacle_colliders.clone(),
                obstacle_clearance_colliders: obstacle_clearance_colliders.clone(),
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
            current_connection_trace_cache.push(result.trace_path.clone());
            result.trace_path
        };
        let fixed_trace = FixedTrace{
            net_name: connection.net_name.clone(),
            connection_id: connection.connection_id,
            trace_path,
        };
        let new_node = top_node.push_node(current_connection, fixed_trace);
        backtrack_stack.push(new_node);  
    }
    Err("No solution found".to_string())
}