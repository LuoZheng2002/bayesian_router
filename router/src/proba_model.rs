use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet},
    num::NonZeroUsize,
    rc::Rc,
    sync::{Arc, Mutex, atomic::Ordering},
};

use rand::distr::{Distribution, weighted::WeightedIndex};
use shared::{
    collider::Collider,
    deterministic_rand::create_deterministic_rng,
    hyperparameters::{
        CONSTANT_LEARNING_RATE, ITERATION_TO_NUM_TRACES, ITERATION_TO_PRIOR_PROBABILITY,
        LINEAR_LEARNING_RATE, MAX_GENERATION_ATTEMPTS, NEXT_ITERATION_TO_REMAINING_PROBABILITY,
        OPPORTUNITY_COST_WEIGHT, SAMPLE_ITERATIONS, SCORE_WEIGHT,
    },
    pcb_problem::{Connection, ConnectionID, FixedTrace, NetName, PcbProblem},
    pcb_render_model::{PcbRenderModel, RenderableBatch, ShapeRenderable, UpdatePcbRenderModel},
    prim_shape::PrimShape,
    trace_path::{TraceAnchors, TracePath},
    vec2::{FixedPoint, FixedVec2},
};

use crate::{
    astar::AStarModel, astar_check_struct::AStarCheck, command_flags::{CommandFlag, COMMAND_CVS, COMMAND_LEVEL, COMMAND_MUTEXES}, quad_tree::{self, QuadTreeNode}
};

#[derive(Copy, Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct ProbaTraceID(pub usize);

#[derive(Debug)]
pub struct ProbaTrace {
    pub net_name: NetName,                    // The net that the trace belongs to
    pub connection_id: ConnectionID,          // The connection that the trace belongs to
    pub proba_trace_id: ProbaTraceID,         // Unique identifier for the trace
    pub trace_path: TracePath,                // The path of the trace
    pub iteration: NonZeroUsize, // The iteration that the trace belongs to, starting from 1
    pub posterior: RefCell<Option<f64>>, // to be accessed in the next iteration
    pub temp_posterior: RefCell<Option<f64>>, // serve as a buffer for simultaneous updates
}

impl ProbaTrace {
    fn get_normalized_prior(&self) -> f64 {
        *ITERATION_TO_PRIOR_PROBABILITY
            .get(&self.iteration)
            .expect(format!("No prior probability for iteration {:?}", self.iteration).as_str())
    }

    pub fn get_posterior_with_fallback(&self) -> f64 {
        let posterior = self.posterior.borrow();
        if let Some(posterior) = posterior.as_ref() {
            *posterior
        } else {
            self.get_normalized_prior()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Traces {
    Fixed(FixedTrace), // A trace that is fixed and does not change
    Probabilistic(HashMap<ProbaTraceID, Rc<ProbaTrace>>), // A trace that is probabilistic and can change
}

// a connection can have either a determined trace or multiple probabilistic traces
pub struct ProbaModel {
    pub trace_id_generator: Box<dyn Iterator<Item = ProbaTraceID> + Send + 'static>, // A generator for TraceID, starting from 0
    pub connection_to_traces: HashMap<ConnectionID, Traces>, // ConnectionID to list of traces
    // pub visited_traces: BTreeSet<TraceAnchors>,
    pub collision_adjacency: HashMap<ProbaTraceID, HashSet<ProbaTraceID>>, // TraceID to set of colliding TraceIDs
    pub next_iteration: NonZeroUsize, // The next iteration to be processed, starting from 1
}

impl ProbaModel {
    pub fn create_and_solve(
        problem: &PcbProblem,
        fixed_traces: &HashMap<ConnectionID, FixedTrace>,
        pcb_render_model: Arc<Mutex<Option<PcbRenderModel>>>,
    ) -> Self {
        let mut connection_ids: Vec<ConnectionID> = Vec::new();
        for net_info in problem.nets.values() {
            for connection in net_info.connections.keys() {
                connection_ids.push(*connection);
            }
        }
        let mut connection_to_traces: HashMap<ConnectionID, Traces> = HashMap::new();
        for connection_id in connection_ids {
            let traces = if let Some(fixed_trace) = fixed_traces.get(&connection_id) {
                Traces::Fixed(fixed_trace.clone())
            } else {
                Traces::Probabilistic(HashMap::new())
            };
            connection_to_traces.insert(connection_id, traces);
        }
        let mut proba_model = ProbaModel {
            trace_id_generator: Box::new((0..).map(ProbaTraceID)),
            connection_to_traces,
            collision_adjacency: HashMap::new(),
            next_iteration: NonZeroUsize::new(1).expect("Next iteration must be non-zero"),
        };
        // display and block
        let display_when_necessary = |proba_model: &ProbaModel, command_flag: CommandFlag| {
            let current_command_level = COMMAND_LEVEL.load(Ordering::Relaxed);
            let target_command_level = command_flag.get_level();
            if current_command_level <= target_command_level {
                {
                    let mut pcb_render_model = pcb_render_model.lock().unwrap();
                    if pcb_render_model.is_some() {
                        return;
                    }
                    let render_model = proba_model.to_pcb_render_model(problem);
                    *pcb_render_model = Some(render_model);
                }
                {
                    let mutex_guard = COMMAND_MUTEXES[target_command_level as usize]
                        .lock()
                        .unwrap();
                    let _unused = COMMAND_CVS[target_command_level as usize]
                        .wait(mutex_guard)
                        .unwrap();
                }
            }
        };
        display_when_necessary(&proba_model, CommandFlag::UpdatePosteriorResult);

        // sample and then update posterior
        // to do: specify iteration number
        for j in 0..SAMPLE_ITERATIONS {
            println!("Sampling new traces for iteration {}", j + 1);
            proba_model.sample_new_traces(problem, pcb_render_model.clone());
            println!("Done sampling new traces");
            display_when_necessary(&proba_model, CommandFlag::UpdatePosteriorResult);

            for i in 0..10 {
                println!("Updating posterior for the {}th time", i + 1);
                proba_model.update_posterior();
                display_when_necessary(&proba_model, CommandFlag::AstarFrontierOrUpdatePosterior);
            }
        }
        proba_model
    }

    fn sample_new_traces(
        &mut self,
        problem: &PcbProblem,
        pcb_render_model: Arc<Mutex<Option<PcbRenderModel>>>,
    ) {
        let mut rng = create_deterministic_rng();
        let mut new_proba_traces: Vec<Rc<ProbaTrace>> = Vec::new();
        // connection_id to connection
        let mut connections: HashMap<ConnectionID, Rc<Connection>> = HashMap::new();
        // connection_id to net_id
        let mut connection_to_net: HashMap<ConnectionID, NetName> = HashMap::new();
        for (net_name, net_info) in problem.nets.iter() {
            for (connection_id, connection) in net_info.connections.iter() {
                connections.insert(*connection_id, connection.clone());
                connection_to_net.insert(*connection_id, net_name.clone());
            }
        }

        // proba_trace_id to proba_trace
        let mut proba_traces: HashMap<ProbaTraceID, Rc<ProbaTrace>> = HashMap::new();
        // visited TraceAnchors
        let mut visited_traces: BTreeSet<TraceAnchors> = BTreeSet::new();
        let mut connection_to_visited_traces: HashMap<ConnectionID, Vec<TracePath>> = HashMap::new();
        for (connection_id, traces) in self.connection_to_traces.iter() {
            if let Traces::Probabilistic(trace_map) = traces {
                for (proba_trace_id, proba_trace) in trace_map.iter() {
                    proba_traces.insert(*proba_trace_id, proba_trace.clone());
                    visited_traces.insert(proba_trace.trace_path.anchors.clone());
                    connection_to_visited_traces.entry(*connection_id)
                        .or_insert_with(Vec::new)
                        .push(proba_trace.trace_path.clone());
                }
            }
        }

        // net_id to proba_trace_ids
        let mut net_to_proba_traces: HashMap<NetName, Vec<ProbaTraceID>> = problem
            .nets
            .keys()
            .map(|net_name| (net_name.clone(), Vec::new()))
            .collect();
        for (connection_id, traces) in self.connection_to_traces.iter() {
            if let Traces::Probabilistic(trace_ids) = traces {
                let net_id = connection_to_net.get(connection_id).expect(
                    format!(
                        "ConnectionID {:?} not found in connection_id_to_net_id",
                        connection_id
                    )
                    .as_str(),
                );
                net_to_proba_traces
                    .get_mut(net_id)
                    .expect(format!("NetID {:?} not found in net_to_proba_traces", net_id).as_str())
                    .extend(trace_ids.keys().cloned());
            }
        }
        // temporary normalized posterior for each proba_trace
        let mut temp_normalized_posteriors: HashMap<ProbaTraceID, f64> = HashMap::new();
        for (_, traces) in self.connection_to_traces.iter() {
            if let Traces::Probabilistic(trace_ids) = traces {
                let mut sum_posterior: f64 = 0.0;
                for (_, proba_trace) in trace_ids.iter() {
                    let posterior = proba_trace.get_posterior_with_fallback();
                    sum_posterior += posterior;
                }
                let remaining_probability = *NEXT_ITERATION_TO_REMAINING_PROBABILITY
                    .get(&self.next_iteration)
                    .expect(
                        format!(
                            "No remaining probability for iteration {:?}",
                            self.next_iteration
                        )
                        .as_str(),
                    );
                assert_ne!(remaining_probability, 0.0);
                sum_posterior += remaining_probability;

                // normalize the posterior for each trace
                // divide each posterior by the sum of all posteriors
                for (proba_trace_id, proba_trace) in trace_ids.iter() {
                    let posterior = proba_trace.get_posterior_with_fallback();
                    assert!(!posterior.is_nan(), "Posterior is NaN for trace ID {:?}", proba_trace_id);
                    let normalized_posterior = posterior / sum_posterior;
                    assert!(!normalized_posterior.is_nan(), "Normalized posterior is NaN");
                    temp_normalized_posteriors.insert(*proba_trace_id, normalized_posterior);
                }
            }
        }

        let border_colliders = AStarModel::calculate_border_colliders(problem.width, problem.height, problem.center);
        
        let sample_cnt = 0;
        // the outer loop for generating the dijkstra model
        for (net_name, net_info) in problem.nets.iter() {
            // collect connections that are not in this net
            let obstacle_connections: HashSet<ConnectionID> = problem
                .nets
                .iter()
                .filter(|(other_net_id, _)| **other_net_id != *net_name)
                .flat_map(|(_, net_info)| net_info.connections.keys())
                .cloned()
                .collect();
            let mut obstacle_source_pad_shapes: HashMap<usize, Vec<PrimShape>> = (0..problem
                .num_layers)
                .map(|layer| (layer, Vec::new()))
                .collect();
            let mut obstacle_source_pad_clearance_shapes: HashMap<usize, Vec<PrimShape>> = (0
                ..problem.num_layers)
                .map(|layer| (layer, Vec::new()))
                .collect();
            for (_, net_info) in problem
                .nets
                .iter()
                .filter(|(other_net_id, _)| **other_net_id != *net_name)
            {
                let source_pad_layers = net_info.source.pad_layer.get_iter(problem.num_layers);
                for layer in source_pad_layers {
                    obstacle_source_pad_shapes
                        .get_mut(&layer)
                        .unwrap()
                        .extend(net_info.source.to_shapes());
                    obstacle_source_pad_clearance_shapes
                        .get_mut(&layer)
                        .unwrap()
                        .extend(net_info.source.to_clearance_shapes());
                }
            }
            // initialize the number of generated traces for each connection
            let mut num_generated_traces: HashMap<ConnectionID, usize> = self
                .connection_to_traces
                .keys()
                .map(|connection_id| (*connection_id, 0))
                .collect();
            // initialize the number of generation attempts
            let mut num_generation_attempts: usize = 0;
            // the inner loop for generating traces for each connection in the net
            let max_num_traces = *ITERATION_TO_NUM_TRACES.get(&self.next_iteration).expect(
                format!(
                    "No number of traces for iteration {:?}",
                    self.next_iteration
                )
                .as_str(),
            );

            // in this while loop, we will generate obstacles f            
            
            // let mut connection_to_visited_traces: HashMap<ConnectionID, Vec<TracePath>> =
            //     HashMap::new();
            while num_generation_attempts < MAX_GENERATION_ATTEMPTS
                && num_generated_traces
                    .values()
                    .any(|&count| count < max_num_traces)
            {
                // println!("Generation attempt: {}", num_generation_attempts + 1);
                num_generation_attempts += 1;
                let mut sampled_obstacle_traces: HashMap<ConnectionID, Option<ProbaTraceID>> =
                    HashMap::new();
                // randomly generate a trace for each pad pair of other nets (in a rare case the trace will not be generated)
                for obstacle_connection_id in obstacle_connections.iter() {
                    // sample a trace from this connection
                    let traces = self
                        .connection_to_traces
                        .get(obstacle_connection_id)
                        .expect(
                            format!(
                                "ConnectionID {:?} not found in connection_to_traces",
                                obstacle_connection_id
                            )
                            .as_str(),
                        );
                    let trace_ids = if let Traces::Probabilistic(trace_ids) = traces {
                        trace_ids.keys().cloned().collect::<Vec<ProbaTraceID>>()
                    } else {
                        continue; // Skip fixed traces
                    };
                    let mut sum_normalized_posterior: f64 = 0.0;
                    let mut normalized_posteriors: Vec<f64> = Vec::new();
                    for trace_id in trace_ids.iter() {
                        let normalized_posterior =
                            *temp_normalized_posteriors.get(trace_id).expect(
                                format!("No normalized posterior for trace ID {:?}", trace_id)
                                    .as_str(),
                            );
                        sum_normalized_posterior += normalized_posterior;
                        normalized_posteriors.push(normalized_posterior);
                    }
                    assert!(
                        sum_normalized_posterior < 1.0,
                        "Sum of normalized posteriors must be less than 1.0, got: {}",
                        sum_normalized_posterior
                    );
                    let num_trace_candidates = normalized_posteriors.len();
                    let remaining_probability = 1.0 - sum_normalized_posterior;
                    normalized_posteriors.push(remaining_probability);
                    let dist = WeightedIndex::new(normalized_posteriors).unwrap();
                    let index = dist.sample(&mut rng);
                    let chosen_proba_trace_id: Option<ProbaTraceID> =
                        if index < num_trace_candidates {
                            Some(trace_ids[index])
                        } else {
                            None
                        };
                    sampled_obstacle_traces.insert(*obstacle_connection_id, chosen_proba_trace_id);
                }
                let mut obstacle_shapes: HashMap<usize, Vec<PrimShape>> = (0..problem.num_layers)
                    .map(|layer| (layer, Vec::new()))
                    .collect();
                let mut obstacle_clearance_shapes: HashMap<usize, Vec<PrimShape>> = (0..problem
                    .num_layers)
                    .map(|layer| (layer, Vec::new()))
                    .collect();
                let quad_tree_side_length = f32::max(problem.width as f32, problem.height as f32);
                let quad_tree_x_min = problem.center.x as f32 - quad_tree_side_length / 2.0;
                let quad_tree_x_max = problem.center.x as f32 + quad_tree_side_length / 2.0;
                let quad_tree_y_min = problem.center.y as f32 - quad_tree_side_length / 2.0;
                let quad_tree_y_max = problem.center.y as f32 + quad_tree_side_length / 2.0;
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
                for i in (0..problem.num_layers).into_iter() {
                    // add source pad shapes and clearance shapes
                    let obstacle_shapes = obstacle_shapes.get_mut(&i).unwrap();
                    let obstacle_clearance_shapes = obstacle_clearance_shapes.get_mut(&i).unwrap();
                    let obstacle_colliders = obstacle_colliders.get_mut(&i).unwrap();
                    let obstacle_clearance_colliders =
                        obstacle_clearance_colliders.get_mut(&i).unwrap();
                    for shape in obstacle_source_pad_shapes.get(&i).unwrap().iter() {
                        obstacle_shapes.push(shape.clone());
                        let collider = Collider::from_prim_shape(shape);
                        obstacle_colliders.insert(collider);
                    }
                    for clearance_shape in
                        obstacle_source_pad_clearance_shapes.get(&i).unwrap().iter()
                    {
                        obstacle_clearance_shapes.push(clearance_shape.clone());
                        let clearance_collider = Collider::from_prim_shape(clearance_shape);
                        obstacle_clearance_colliders.insert(clearance_collider);
                    }
                }
                // add fixed traces to the obstacle shapes
                for obstacle_connection_id in obstacle_connections.iter() {
                    let traces = self
                        .connection_to_traces
                        .get(obstacle_connection_id)
                        .expect(
                            format!(
                                "ConnectionID {:?} not found in connection_to_traces",
                                obstacle_connection_id
                            )
                            .as_str(),
                        );
                    let fixed_trace = if let Traces::Fixed(fixed_trace) = traces {
                        fixed_trace
                    } else {
                        continue; // Skip probabilistic traces
                    };
                    let trace_segments = &fixed_trace.trace_path.segments;
                    for segment in trace_segments.iter() {
                        let layer = segment.layer;
                        let shapes = segment.to_shapes();
                        let clearance_shapes = segment.to_clearance_shapes();
                        let obstacle_shapes = obstacle_shapes.get_mut(&layer).unwrap();
                        let obstacle_clearance_shapes =
                            obstacle_clearance_shapes.get_mut(&layer).unwrap();
                        let obstacle_colliders = obstacle_colliders.get_mut(&layer).unwrap();
                        let obstacle_clearance_colliders =
                            obstacle_clearance_colliders.get_mut(&layer).unwrap();
                        for shape in shapes.iter() {
                            obstacle_shapes.push(shape.clone());
                            let collider = Collider::from_prim_shape(shape);
                            obstacle_colliders.insert(collider);
                        }
                        for clearance_shape in clearance_shapes.iter() {
                            obstacle_clearance_shapes.push(clearance_shape.clone());
                            let clearance_collider = Collider::from_prim_shape(clearance_shape);
                            obstacle_clearance_colliders.insert(clearance_collider);
                        }
                    }
                }
                // add all sampled traces to the obstacle shapes
                for (_, proba_trace_id) in sampled_obstacle_traces.iter() {
                    let proba_trace_id = if let Some(proba_trace_id) = proba_trace_id {
                        *proba_trace_id
                    } else {
                        continue; // Skip if no trace was sampled
                    };
                    let proba_trace = proba_traces.get(&proba_trace_id).expect(
                        format!(
                            "ProbaTraceID {:?} not found in proba_traces",
                            proba_trace_id
                        )
                        .as_str(),
                    );
                    let trace_segments = &proba_trace.trace_path.segments;
                    for segment in trace_segments.iter() {
                        let layer = segment.layer;
                        let shapes = segment.to_shapes();
                        let clearance_shapes = segment.to_clearance_shapes();
                        let obstacle_shapes = obstacle_shapes.get_mut(&layer).unwrap();
                        let obstacle_clearance_shapes =
                            obstacle_clearance_shapes.get_mut(&layer).unwrap();
                        let obstacle_colliders = obstacle_colliders.get_mut(&layer).unwrap();
                        let obstacle_clearance_colliders =
                            obstacle_clearance_colliders.get_mut(&layer).unwrap();
                        for shape in shapes.iter() {
                            obstacle_shapes.push(shape.clone());
                            let collider = Collider::from_prim_shape(shape);
                            obstacle_colliders.insert(collider);
                        }
                        for clearance_shape in clearance_shapes.iter() {
                            obstacle_clearance_shapes.push(clearance_shape.clone());
                            let clearance_collider = Collider::from_prim_shape(clearance_shape);
                            obstacle_clearance_colliders.insert(clearance_collider);
                        }
                    }
                }
                // add all pads in other nets to the obstacle shapes
                for obstacle_connection_id in obstacle_connections.iter() {
                    let connection = connections.get(obstacle_connection_id).unwrap();
                    let pad = &connection.sink;
                    let pad_layers = pad.pad_layer.get_iter(problem.num_layers);
                    let pad_shapes = pad.to_shapes();
                    let pad_clearance_shapes = pad.to_clearance_shapes();
                    for layer in pad_layers {
                        let obstacle_shapes = obstacle_shapes.get_mut(&layer).unwrap();
                        let obstacle_clearance_shapes =
                            obstacle_clearance_shapes.get_mut(&layer).unwrap();
                        let obstacle_colliders = obstacle_colliders.get_mut(&layer).unwrap();
                        let obstacle_clearance_colliders =
                            obstacle_clearance_colliders.get_mut(&layer).unwrap();
                        for shape in pad_shapes.iter() {
                            obstacle_shapes.push(shape.clone());
                            let collider = Collider::from_prim_shape(shape);
                            obstacle_colliders.insert(collider);
                        }
                        for clearance_shape in pad_clearance_shapes.iter() {
                            obstacle_clearance_shapes.push(clearance_shape.clone());
                            let clearance_collider = Collider::from_prim_shape(clearance_shape);
                            obstacle_clearance_colliders.insert(clearance_collider);
                        }
                    }
                }
                let obstacle_shapes = Rc::new(obstacle_shapes);
                let obstacle_clearance_shapes = Rc::new(obstacle_clearance_shapes);
                let obstacle_colliders: Rc<HashMap<usize, QuadTreeNode>> =
                    Rc::new(obstacle_colliders);
                let obstacle_clearance_colliders: Rc<HashMap<usize, QuadTreeNode>> =
                    Rc::new(obstacle_clearance_colliders);
                // to do: reuse the obstacle shapes and obstacle clearance shapes

                let connections = &problem
                    .nets
                    .get(net_name)
                    .expect(format!("NetID {:?} not found in nets", net_name).as_str())
                    .connections;
                // only consider connections with probabilistic traces
                let connections: Vec<(ConnectionID, Rc<Connection>)> = connections
                    .iter()
                    .filter(|(connection_id, _)| {
                        let traces = self.connection_to_traces.get(connection_id).expect(
                            format!(
                                "ConnectionID {:?} not found in connection_to_traces",
                                connection_id
                            )
                            .as_str(),
                        );
                        if let Traces::Probabilistic(_) = traces {
                            true // Only consider connections with probabilistic traces
                        } else {
                            false // Skip fixed traces
                        }
                    })
                    .map(|(connection_id, connection)| (*connection_id, connection.clone()))
                    .collect();

                for (connection_id, connection) in connections.iter() {
                    let connection_num_generated_traces =
                        *num_generated_traces.get(connection_id).expect(
                            format!(
                                "ConnectionID {:?} not found in num_generated_traces",
                                connection_id
                            )
                            .as_str(),
                        );
                    if connection_num_generated_traces >= max_num_traces {
                        println!(
                            "ConnectionID {:?} already has enough traces, skipping",
                            connection_id
                        );
                        continue; // Skip this connection if it already has enough traces
                    }
                    // first check if any of the traces in connection_to_visited_traces satisfy the constraints
                    let current_connection_visited_traces = connection_to_visited_traces
                        .entry(*connection_id)
                        .or_insert_with(Vec::new);
                    let mut found_satisfying_trace = false;
                    for trace_path in current_connection_visited_traces.iter() {                        
                        let astar_check = AStarCheck{
                            border_colliders: border_colliders.clone(),
                            obstacle_colliders: obstacle_colliders.clone(),
                            obstacle_clearance_colliders: obstacle_clearance_colliders.clone(),
                            solution_trace: trace_path.clone(),
                            num_layers: problem.num_layers,
                        };
                        if astar_check.check() {
                            found_satisfying_trace = true; // the trace satisfies the constraints
                            break; // we found a trace that satisfies the constraints, no need to generate a new one
                        }
                    }

                    if found_satisfying_trace {
                        println!(
                            "Found a satisfying trace for ConnectionID {:?}, skipping A*",
                            connection_id
                        );
                        continue; // Skip this connection if a satisfying trace is found
                    }

                    // prepare for the a star model
                    // let start = net_info.source.position.to_fixed().to_nearest_even_even();
                    // let end = connection.sink.position.to_fixed().to_nearest_even_even();
                    let start_pad = net_info.pads[&connection.start_pad];
                    let end_pad = net_info.pads[&connection.end_pad];
                    let start = start_pad.position.to_fixed().to_nearest_even_even();
                    let end = end_pad.position.to_fixed().to_nearest_even_even();
                    let start_layers = start_pad.pad_layer;
                    let end_layers = end_pad.pad_layer;
                    let astar_model = AStarModel {
                        width: problem.width,
                        height: problem.height,
                        center: problem.center,
                        obstacle_shapes: obstacle_shapes.clone(),
                        obstacle_clearance_shapes: obstacle_clearance_shapes.clone(),
                        obstacle_colliders: obstacle_colliders.clone(),
                        obstacle_clearance_colliders: obstacle_clearance_colliders.clone(),
                        start,
                        end,
                        start_layers,
                        end_layers,
                        num_layers: problem.num_layers,
                        trace_width: net_info.trace_width,
                        trace_clearance: net_info.trace_clearance,
                        via_diameter: net_info.via_diameter,
                        border_colliders_cache: RefCell::new(None), // Cache for border points, initialized to None
                        border_shapes_cache: RefCell::new(None), // Cache for border shapes, initialized to None
                    };
                    sample_cnt += 1;
                    // run A* algorithm to find a path
                    let astar_result = astar_model.run(pcb_render_model.clone());
                    let astar_result = match astar_result {
                        Ok(result) => result,
                        Err(err) => {
                            println!("A* algorithm failed: {}", err);
                            continue; // Skip this connection if A* fails
                        }
                    };
                    let trace_path = astar_result.trace_path;

                    assert!(!visited_traces.contains(&trace_path.anchors), "Trace path is supposed to be a new one generated by A*");       

                    visited_traces.insert(trace_path.anchors.clone());
                    connection_to_visited_traces
                        .entry(*connection_id)
                        .or_insert_with(Vec::new)
                        .push(trace_path.clone());

                    let proba_trace_id = self
                        .trace_id_generator
                        .next()
                        .expect("TraceID generator exhausted");
                    let proba_trace = ProbaTrace {
                        net_name: net_name.clone(),
                        connection_id: *connection_id,
                        proba_trace_id,
                        trace_path,
                        iteration: self.next_iteration,
                        posterior: RefCell::new(None), // Initialize with None, will be updated later
                        temp_posterior: RefCell::new(None), // Temporary posterior for simultaneous updates
                    };
                    new_proba_traces.push(Rc::new(proba_trace));
                    let num = num_generated_traces.get_mut(connection_id).expect(
                        format!(
                            "ConnectionID {:?} not found in num_generated_traces",
                            connection_id
                        )
                        .as_str(),
                    );
                    *num += 1;
                }
            }
        }
        println!("sample_cnt = {}", sample_cnt);

        // add the new traces to the model
        for proba_trace in new_proba_traces {
            let proba_trace_id = proba_trace.proba_trace_id;
            let connection_id = proba_trace.connection_id;
            let traces = self.connection_to_traces.get_mut(&connection_id).expect(
                format!(
                    "ConnectionID {:?} not found in connection_to_traces",
                    connection_id
                )
                .as_str(),
            );
            // traces can only be probabilistic, if it is fixed, we panic
            let traces = if let Traces::Probabilistic(trace_map) = traces {
                trace_map
            } else {
                panic!(
                    "ConnectionID {:?} has fixed traces, cannot add probabilistic trace",
                    connection_id
                );
            };
            let old = traces.insert(proba_trace_id, proba_trace.clone());
            assert!(
                old.is_none(),
                "ProbaTraceID {:?} already exists for ConnectionID {:?}",
                proba_trace_id,
                connection_id
            );
        }

        // update proba_traces to include the new traces
        let mut proba_traces: HashMap<ProbaTraceID, Rc<ProbaTrace>> = HashMap::new();
        for traces in self.connection_to_traces.values() {
            if let Traces::Probabilistic(trace_map) = traces {
                for (proba_trace_id, proba_trace) in trace_map.iter() {
                    proba_traces.insert(*proba_trace_id, proba_trace.clone());
                }
            }
        }
        // update the collision adjacency
        let mut collision_adjacency: HashMap<ProbaTraceID, HashSet<ProbaTraceID>> = proba_traces
            .iter()
            .map(|(proba_trace_id, _)| (*proba_trace_id, HashSet::new()))
            .collect();
        // update net_to_proba_traces to include the new traces
        let mut net_to_proba_traces: HashMap<NetName, Vec<ProbaTraceID>> = problem
            .nets
            .keys()
            .map(|net_name| (net_name.clone(), Vec::new()))
            .collect();
        for (connection_id, traces) in self.connection_to_traces.iter() {
            if let Traces::Probabilistic(trace_ids) = traces {
                let net_id = connection_to_net.get(connection_id).expect(
                    format!(
                        "ConnectionID {:?} not found in connection_id_to_net_id",
                        connection_id
                    )
                    .as_str(),
                );
                net_to_proba_traces
                    .get_mut(net_id)
                    .expect(format!("NetID {:?} not found in net_to_proba_traces", net_id).as_str())
                    .extend(trace_ids.keys().cloned());
            }
        }
        // calculate the collisions between traces
        let proba_traces_vec: Vec<Vec<ProbaTraceID>> = net_to_proba_traces.into_values().collect();
        for i in 0..proba_traces_vec.len() {
            for j in (i + 1)..proba_traces_vec.len() {
                let net_i = &proba_traces_vec[i];
                let net_j = &proba_traces_vec[j];
                for trace_i in net_i.iter() {
                    for trace_j in net_j.iter() {
                        // check if the traces collide
                        let proba_trace_i = proba_traces.get(trace_i).expect(
                            format!("ProbaTraceID {:?} not found in proba_traces", trace_i)
                                .as_str(),
                        );
                        let proba_trace_j = proba_traces.get(trace_j).expect(
                            format!("ProbaTraceID {:?} not found in proba_traces", trace_j)
                                .as_str(),
                        );
                        if proba_trace_i
                            .trace_path
                            .collides_with(&proba_trace_j.trace_path)
                        {
                            // add the collision to the adjacency
                            collision_adjacency
                                .get_mut(trace_i)
                                .expect(
                                    format!(
                                        "ProbaTraceID {:?} not found in collision_adjacency",
                                        trace_i
                                    )
                                    .as_str(),
                                )
                                .insert(*trace_j);
                            collision_adjacency
                                .get_mut(trace_j)
                                .expect(
                                    format!(
                                        "ProbaTraceID {:?} not found in collision_adjacency",
                                        trace_j
                                    )
                                    .as_str(),
                                )
                                .insert(*trace_i);
                        }
                    }
                }
            }
        }
        self.collision_adjacency = collision_adjacency;
        // update next_iteration
        self.next_iteration = NonZeroUsize::new(self.next_iteration.get() + 1).unwrap();
    }
    pub fn to_pcb_render_model(&self, problem: &PcbProblem) -> PcbRenderModel {
        let mut trace_shape_renderables: Vec<RenderableBatch> = Vec::new();
        let mut pad_shape_renderables: Vec<ShapeRenderable> = Vec::new();
        let mut other_shape_renderables: Vec<ShapeRenderable> = Vec::new();
        for (_, net_info) in problem.nets.iter() {
            // add all pads in a net
            let net_color = net_info.color.to_float4(1.0);
            for pad in net_info.pads.values(){
                let pad_renderables = pad.to_renderables(net_color);
                let pad_clearance_renderables = pad.to_clearance_renderables(net_color);
                pad_shape_renderables.extend(pad_renderables);
                pad_shape_renderables.extend(pad_clearance_renderables);
            }            
            for (_, connection) in net_info.connections.iter() {
                // Add fixed traces
                if let Some(Traces::Fixed(fixed_trace)) =
                    self.connection_to_traces.get(&connection.connection_id)
                {
                    let color = net_info.color.to_float4(1.0);
                    let renderable_batches = fixed_trace.trace_path.to_renderables(color);
                    trace_shape_renderables.extend(renderable_batches);
                }
                // Add probabilistic traces
                if let Some(Traces::Probabilistic(trace_map)) =
                    self.connection_to_traces.get(&connection.connection_id)
                {
                    for proba_trace in trace_map.values() {
                        let posterior = proba_trace.get_posterior_with_fallback();
                        let posterior = posterior.clamp(0.0, 1.0); // Ensure posterior is between 0 and 1
                        let color = net_info.color.to_float4(posterior as f32);
                        let renderable_batches = proba_trace.trace_path.to_renderables(color);
                        trace_shape_renderables.extend(renderable_batches);
                    }
                }                
            }
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

    pub fn update_posterior(&mut self) {
        let proba_traces: HashMap<ProbaTraceID, Rc<ProbaTrace>> = self
            .connection_to_traces
            .values()
            .filter_map(|traces| {
                if let Traces::Probabilistic(trace_map) = traces {
                    Some(trace_map.clone())
                } else {
                    None // Skip fixed traces
                }
            })
            .flatten()
            .collect();
        // Update the posterior probabilities for all traces in the model
        for (proba_trace_id, proba_trace) in proba_traces.iter() {
            let adjacent_traces = self.collision_adjacency.get(proba_trace_id).expect(
                format!("No adjacent traces for ProbaTraceID {:?}", proba_trace_id).as_str(),
            );
            let mut proba_product = 1.0;
            for adjacent_trace_id in adjacent_traces.iter() {
                let adjacent_trace = proba_traces.get(adjacent_trace_id).expect(
                    format!(
                        "No ProbaTraceID {:?} found in proba_traces",
                        adjacent_trace_id
                    )
                    .as_str(),
                );
                let posterior = adjacent_trace.get_posterior_with_fallback();
                // to do: update this
                let one_minus_posterior = (1.0 - posterior).max(0.0); // Ensure non-negative
                proba_product *= one_minus_posterior;
            }
            let target_posterior = proba_product;
            assert!(
                target_posterior >= 0.0 && target_posterior <= 1.0,
                "Target posterior must be between 0 and 1"
            );
            // get num traces in the same iteration
            let current_posterior = proba_trace.get_posterior_with_fallback();
            let opportunity_cost = target_posterior / current_posterior;
            let score = proba_trace.trace_path.get_score();
            let score_weight = *SCORE_WEIGHT.lock().unwrap();
            let opportunity_cost_weight = *OPPORTUNITY_COST_WEIGHT.lock().unwrap();
            let target_posterior_unnormalized = 1.0
                * f64::powf(score, score_weight)
                * f64::powf(opportunity_cost, opportunity_cost_weight);
            let target_posterior_normalized =
                proba_trace.get_normalized_prior() * target_posterior_unnormalized;
            let mut temp_posterior = proba_trace.temp_posterior.borrow_mut();
            let target_greater_than_current = target_posterior_normalized > current_posterior;
            let constant_offset = if target_greater_than_current {
                CONSTANT_LEARNING_RATE
            } else {
                -CONSTANT_LEARNING_RATE
            };
            let new_posterior = current_posterior
                + (target_posterior_normalized - current_posterior) * LINEAR_LEARNING_RATE
                + constant_offset;
            let new_posterior = if target_greater_than_current {
                new_posterior.max(target_posterior_normalized)
            } else {
                new_posterior.min(target_posterior_normalized)
            };
            *temp_posterior = Some(new_posterior);
        }
        // move temp_posterior to posterior
        for (_, proba_trace) in proba_traces.iter() {
            let mut posterior = proba_trace.posterior.borrow_mut();
            let mut temp_posterior = proba_trace.temp_posterior.borrow_mut();
            let temp_posterior_val = temp_posterior.unwrap();
            *posterior = Some(temp_posterior_val);
            // reset temp_posterior
            *temp_posterior = None;
        }
    }
}
