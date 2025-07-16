use std::{collections::HashMap, sync::{Arc, Mutex}};

use shared::{pcb_problem::{PcbProblem, PcbSolution}, pcb_render_model::PcbRenderModel};

use crate::backtrack_node::BacktrackNode;



pub fn solve_pcb_problem(
    pcb_problem: &PcbProblem,
    pcb_render_model: Arc<Mutex<PcbRenderModel>>,
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
        println!("Current stack:");
        for (index, node) in node_stack.iter().enumerate() {
            println!(
                "\tNode {}: up_to_date: {}, num fixed traces: {}, num remaining trace candidates: {}, ",
                index,
                node.prob_up_to_date,
                node.fixed_traces.len(),
                node.remaining_trace_candidates.len()
            );
        }
    }

    let first_node = BacktrackNode::from_fixed_traces(pcb_problem, &HashMap::new(), pcb_render_model.clone());
    // assume the first node has trace candidates
    node_stack.push(first_node);

    while node_stack.len() > 0 {
        print_current_stack(&node_stack);
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
        let new_node = top_node.try_fix_top_ranked_trace();
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
                let target_index = (current_node_index + last_updated_index + 1) / 2; // bias to right for consistency
                let new_node = node_stack[target_index]
                    .try_update_proba_model(pcb_problem, pcb_render_model.clone());
                match new_node {
                    Some(new_node) => {
                        // If we successfully updated the probabilistic model, replace the node at the target index with the new node
                        assert!(
                            target_index < node_stack.len(),
                            "Target index must be within the stack bounds"
                        );
                        if target_index == node_stack.len() - 1 {
                            node_stack.push(new_node);
                        } else {
                            node_stack[target_index + 1] = new_node;
                            node_stack.truncate(target_index + 2); // Remove all nodes above the target index
                            println!(
                                "Successfully updated the probabilistic model, replacing node at index {}",
                                target_index
                            );
                        }
                    }
                    None => {
                        // If we failed to update the probabilistic model, we pop the current node from the stack
                        assert!(
                            target_index == node_stack.len() - 1,
                            "target index must be the last node in the stack"
                        );
                        node_stack.pop();
                        println!(
                            "Failed to update the probabilistic model, popping the current node from the stack"
                        );
                    }
                }
            }
        }
    }
    Err("No solution found".to_string())
}