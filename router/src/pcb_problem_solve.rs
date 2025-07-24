use std::sync::{atomic::Ordering, Arc, Mutex};

use shared::{hyperparameters::SAMPLE_CNT, pcb_problem::{PcbProblem, PcbSolution}, pcb_render_model::PcbRenderModel};

use crate::{bayesian_backtrack_algo::bayesian_backtrack, naive_backtrack_algo::naive_backtrack};



/// this either calls naive backtrack or bayesian backtrack
pub fn solve_pcb_problem(
    pcb_problem: &PcbProblem,
    pcb_render_model: Arc<Mutex<Option<PcbRenderModel>>>,
    bayesian: bool,
) -> Result<PcbSolution, String> {
    let result = if bayesian {
        // Call the Bayesian backtrack function
        bayesian_backtrack(pcb_problem, pcb_render_model)
    } else {
        // Call the naive backtrack function
        naive_backtrack(pcb_problem, pcb_render_model)
    };
    match result{
        Ok(solution) => {
            println!("PCB problem solved successfully");
            println!("Sample Count: {}", SAMPLE_CNT.load(Ordering::SeqCst));
            Ok(solution)
        }
        Err(e) => {
            println!("Failed to solve PCB problem: {}", e);
            println!("Sample Count: {}", SAMPLE_CNT.load(Ordering::SeqCst));
            Err(e)
        }
    }
}
