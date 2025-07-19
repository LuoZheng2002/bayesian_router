use std::{cell::RefCell, collections::{BTreeSet, HashMap, HashSet}, num::NonZeroUsize, rc::Rc, sync::{Arc, Mutex}};

use rand::distr::{weighted::WeightedIndex, Distribution};
use shared::{hyperparameters::{CONSTANT_LEARNING_RATE, ITERATION_TO_NUM_TRACES, ITERATION_TO_PRIOR_PROBABILITY, LINEAR_LEARNING_RATE, MAX_GENERATION_ATTEMPTS, NEXT_ITERATION_TO_REMAINING_PROBABILITY, OPPORTUNITY_COST_WEIGHT, SCORE_WEIGHT}, pcb_problem::{Connection, ConnectionID, FixedTrace, NetName, PcbProblem}, pcb_render_model::{PcbRenderModel, RenderableBatch, ShapeRenderable, UpdatePcbRenderModel}, prim_shape::PrimShape, trace_path::{TraceAnchors, TracePath}, vec2::{FixedPoint, FixedVec2}};

use crate::{astar::AStarModel, block_or_sleep::{block_or_sleep, block_thread}};



#[derive(Copy, Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct ProbaTraceID(pub usize);

#[derive(Debug)]
pub struct ProbaTrace {
    pub net_name: NetName,                        // The net that the trace belongs to
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
        pcb_render_model: Arc<Mutex<PcbRenderModel>>,
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
        let display_and_block = |proba_model: &ProbaModel| {
            let render_model = proba_model.to_pcb_render_model(problem);
            pcb_render_model.update_pcb_render_model(render_model);
            block_or_sleep();
        };
        display_and_block(&proba_model);
        block_thread();

        // sample and then update posterior
        // to do: specify iteration number
        for j in 0..2 {
            println!("Sampling new traces for iteration {}", j + 1);
            proba_model.sample_new_traces(problem, pcb_render_model.clone());
            display_and_block(&proba_model);
            block_thread();

            for i in 0..10 {
                println!("Updating posterior for the {}th time", i + 1);
                proba_model.update_posterior();
                display_and_block(&proba_model);
            }
            block_thread();
        }
        proba_model
    }

    fn sample_new_traces(
        &mut self,
        problem: &PcbProblem,
        pcb_render_model: Arc<Mutex<PcbRenderModel>>,
    ) {
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
        for traces in self.connection_to_traces.values() {
            if let Traces::Probabilistic(trace_map) = traces {
                for (proba_trace_id, proba_trace) in trace_map.iter() {
                    proba_traces.insert(*proba_trace_id, proba_trace.clone());
                    visited_traces.insert(proba_trace.trace_path.anchors.clone());
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
                sum_posterior += NEXT_ITERATION_TO_REMAINING_PROBABILITY
                    .get(&self.next_iteration)
                    .expect(
                        format!(
                            "No remaining probability for iteration {:?}",
                            self.next_iteration
                        )
                        .as_str(),
                    );
                // normalize the posterior for each trace
                // divide each posterior by the sum of all posteriors
                for (proba_trace_id, proba_trace) in trace_ids.iter() {
                    let posterior = proba_trace.get_posterior_with_fallback();
                    let normalized_posterior = posterior / sum_posterior;
                    temp_normalized_posteriors.insert(*proba_trace_id, normalized_posterior);
                }
            }
        }

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
            let mut obstacle_source_pad_shapes: HashMap<usize, Vec<PrimShape>> = (0..problem.num_layers)
                .map(|layer| (layer, Vec::new()))
                .collect();
            let mut obstacle_source_pad_clearance_shapes: HashMap<usize, Vec<PrimShape>> = (0..problem.num_layers)
                .map(|layer| (layer, Vec::new()))
                .collect();
            for (_, net_info) in problem.nets.iter().filter(|(other_net_id, _)| **other_net_id != *net_name) {
                let source_pad_layers = net_info.source.pad_layer.get_iter(problem.num_layers);
                for layer in source_pad_layers {
                    obstacle_source_pad_shapes.get_mut(&layer).unwrap().extend(net_info.source.to_shapes());
                    obstacle_source_pad_clearance_shapes.get_mut(&layer).unwrap().extend(net_info.source.to_clearance_shapes());
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
                    let mut rng = rand::rng();
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
                let mut obstacle_clearance_shapes: HashMap<usize, Vec<PrimShape>> = (0..problem.num_layers)
                    .map(|layer| (layer, Vec::new()))
                    .collect();
                for i in (0..problem.num_layers).into_iter() {
                    // add source pad shapes and clearance shapes
                    obstacle_shapes.get_mut(&i).unwrap().extend(
                        obstacle_source_pad_shapes.get(&i).unwrap().iter().cloned(),
                    );
                    obstacle_clearance_shapes.get_mut(&i).unwrap().extend(
                        obstacle_source_pad_clearance_shapes.get(&i).unwrap().iter().cloned(),
                    );
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
                        obstacle_shapes.get_mut(&layer).unwrap().extend(shapes);
                        obstacle_clearance_shapes.get_mut(&layer).unwrap().extend(clearance_shapes);
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
                        obstacle_shapes.get_mut(&layer).unwrap().extend(shapes);
                        obstacle_clearance_shapes.get_mut(&layer).unwrap().extend(clearance_shapes);
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
                        obstacle_shapes.get_mut(&layer).unwrap().extend(pad_shapes.clone());
                        obstacle_clearance_shapes.get_mut(&layer).unwrap().extend(pad_clearance_shapes.clone());
                    }
                }
                let obstacle_shapes = Rc::new(obstacle_shapes);
                let obstacle_clearance_shapes = Rc::new(obstacle_clearance_shapes);
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

                    // sample a trace for this connection
                    let start = net_info.source.position.to_fixed().to_nearest_even_even();
                    let end = connection.sink.position.to_fixed().to_nearest_even_even();
                    let start_layers = net_info.source.pad_layer;
                    let end_layers = connection.sink.pad_layer;
                    let astar_model = AStarModel {
                        width: problem.width,
                        height: problem.height,
                        center: problem.center,
                        obstacle_shapes: obstacle_shapes.clone(),
                        obstacle_clearance_shapes: obstacle_clearance_shapes.clone(),
                        start,
                        end,
                        start_layers,
                        end_layers,
                        num_layers: problem.num_layers,
                        trace_width: connection.sink_trace_width,            
                        trace_clearance: connection.sink_trace_clearance,    
                        via_diameter: net_info.via_diameter,
                        border_colliders_cache: RefCell::new(None), // Cache for border points, initialized to None
                        border_shapes_cache: RefCell::new(None), // Cache for border shapes, initialized to None
                    };                    
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
                    if visited_traces.contains(&trace_path.anchors) {
                        // println!(
                        //     "Trace path {:?} already visited, skipping",
                        //     trace_path.anchors
                        // );
                        continue;
                    }
                    visited_traces.insert(trace_path.anchors.clone());
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
        for (_, net_info) in problem.nets.iter() {
            // add source pad
            let source_renderables = net_info
                .source
                .to_renderables(net_info.color.to_float4(1.0));
            let source_clearance_renderables = net_info
                .source
                .to_clearance_renderables(net_info.color.to_float4(0.5));
            pad_shape_renderables.extend(source_renderables);
            pad_shape_renderables.extend(source_clearance_renderables);
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
                let sink_renderables = connection
                    .sink
                    .to_renderables(net_info.color.to_float4(1.0));
                let sink_clearance_renderables = connection
                    .sink
                    .to_clearance_renderables(net_info.color.to_float4(0.5));                
                pad_shape_renderables.extend(sink_renderables);
                pad_shape_renderables.extend(sink_clearance_renderables);
            }
        }
        PcbRenderModel {
            width: problem.width,
            height: problem.height,
            center: problem.center,
            trace_shape_renderables,
            pad_shape_renderables,
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