use std::{
    collections::HashMap,
    sync::{Arc, Mutex, atomic::Ordering},
    thread,
    time::Duration,
};

use shared::{
    color_float3::ColorFloat3,
    hyperparameters::NUM_TOP_RANKED_TO_TRY,
    pcb_problem::{NetName, PcbProblem, PcbSolution},
    pcb_render_model::{PcbRenderModel, RenderableBatch, ShapeRenderable, UpdatePcbRenderModel},
    prim_shape::PrimShape,
};

use crate::{
    backtrack_node::BacktrackNode,
    block_or_sleep,
    command_flags::{COMMAND_CVS, COMMAND_LEVEL, COMMAND_MUTEXES, CommandFlag},
};

pub fn solve_pcb_problem(
    pcb_problem: &PcbProblem,
    pcb_render_model: Arc<Mutex<Option<PcbRenderModel>>>,
) -> Result<PcbSolution, String> {
    let mut node_stack: Vec<BacktrackNode> = Vec::new();

    fn last_updated_node_index(node_stack: &Vec<BacktrackNode>) -> usize {
        for (index, node) in node_stack.iter().enumerate().rev() {
            if node.prob_up_to_date {
                return index; // Return the index of the last updated node
            }
        }
        // because the first node is always up to date, it is impossible to reach here
        panic!("No updated node found in the stack");
    }

    fn print_current_stack(node_stack: &Vec<BacktrackNode>) {
        println!("Current stack: num_items: {}", node_stack.len());
        for (index, node) in node_stack.iter().enumerate() {
            println!(
                "\tNode {}: up_to_date: {}, num fixed traces: {}, num remaining trace candidates: {}, ",
                index,
                node.prob_up_to_date,
                node.fixed_traces.len(),
                node.remaining_trace_candidates.len()
            );
            for fixed_trace in node.fixed_traces.values() {
                println!(
                    "\t\tFixed trace: net_name: {}, connection_id: {}",
                    fixed_trace.net_name.0, fixed_trace.connection_id.0
                );
            }
        }
    }

    fn node_to_pcb_render_model(problem: &PcbProblem, node: &BacktrackNode) -> PcbRenderModel {
        let mut trace_shape_renderables: Vec<RenderableBatch> = Vec::new();
        let mut pad_shape_renderables: Vec<ShapeRenderable> = Vec::new();
        let mut other_shape_renderables: Vec<ShapeRenderable> = Vec::new();
        let mut net_name_to_color: HashMap<NetName, ColorFloat3> = HashMap::new();
        for (_, net_info) in problem.nets.iter() {
            net_name_to_color.insert(net_info.net_name.clone(), net_info.color);
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
        for fixed_trace in node.fixed_traces.values() {
            let renderable_batches = fixed_trace
                .trace_path
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
        node: &BacktrackNode,
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
            thread::sleep(Duration::from_millis(400));
        }
    }

    let first_node =
        BacktrackNode::from_fixed_traces(pcb_problem, &HashMap::new(), pcb_render_model.clone());
    // assume the first node has trace candidates
    node_stack.push(first_node);

    while node_stack.len() > 0 {
        print_current_stack(&node_stack);
        display_when_necessary(
            node_stack.last().unwrap(),
            pcb_problem,
            pcb_render_model.clone(),
        );
        let top_node = node_stack.last_mut().unwrap();
        if top_node.is_solution(pcb_problem) {
            println!("Found a solution!");
            // If the top node is a solution, we can return it
            let fixed_traces = top_node.fixed_traces.clone();
            let solution = PcbSolution {
                determined_traces: fixed_traces,
            };
            return Ok(solution);
        }
        let display_and_block_closure = |node: &BacktrackNode| {
            display_when_necessary(node, pcb_problem, pcb_render_model.clone());
        };
        let new_node =
            top_node.try_fix_top_k_ranked_trace(display_and_block_closure, NUM_TOP_RANKED_TO_TRY);
        match new_node {
            Some(new_node) => {
                // If we successfully fixed a trace, push the new node onto the stack
                println!(
                    "Successfully fixed the top ranked trace, pushing new node onto the stack"
                );
                // assert!(new_node.prob_up_to_date, "New node must be up to date");
                node_stack.push(new_node);
            }
            None => {
                // If we failed to fix the top-ranked trace, we update the node in the middle between the current position and the last updated node
                println!(
                    "Failed to fix the top ranked trace, trying to update the probabilistic model in the middle of the stack"
                );
                let current_node_index = node_stack.len() - 1;
                let last_updated_index = last_updated_node_index(&node_stack);
                // let target_index = (current_node_index + last_updated_index + 1) / 2; // bias to right for consistency
                let target_index = current_node_index;
                let new_node = node_stack[target_index]
                    .try_update_proba_model(pcb_problem, pcb_render_model.clone());
                match new_node {
                    Some(new_node) => {
                        // If we successfully updated the probabilistic model, replace the node at the target index with the new node
                        assert!(
                            target_index < node_stack.len(),
                            "Target index must be within the stack bounds"
                        );
                        // if target_index == node_stack.len() - 1 {
                        //     node_stack.push(new_node);
                        // } else {
                        //     node_stack[target_index + 1] = new_node;
                        //     node_stack.truncate(target_index + 2); // Remove all nodes above the target index
                        //     println!(
                        //         "Successfully updated the probabilistic model, replacing node at index {}",
                        //         target_index
                        //     );
                        // }
                        node_stack[target_index] = new_node;
                        node_stack.truncate(target_index + 1); // Remove all nodes above the target index
                        println!(
                            "Successfully updated the probabilistic model, replacing node at index {}",
                            target_index
                        );
                        print_current_stack(&node_stack);
                    }
                    None => {
                        // // If we failed to update the probabilistic model, we pop the current node from the stack
                        // assert!(
                        //     target_index == node_stack.len() - 1,
                        //     "target index must be the last node in the stack"
                        // );
                        // node_stack.pop();
                        // println!(
                        //     "Failed to update the probabilistic model, popping the current node from the stack"
                        // );
                        panic!("failed to find a solution")
                    }
                }
            }
        }
    }
    Err("No solution found".to_string())
}
