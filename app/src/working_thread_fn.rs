use std::sync::{Arc, Mutex};

use cgmath::Deg;

use router::{pcb_problem_solve::solve_pcb_problem, test_pcb_problem::pcb_problem1};
use shared::pcb_render_model::PcbRenderModel;

pub fn working_thread_fn(pcb_render_model: Arc<Mutex<PcbRenderModel>>) {
    println!("Working thread started");
    let pcb_problem = pcb_problem1();
    let result = solve_pcb_problem(&pcb_problem, pcb_render_model.clone());
    match result {
        Ok(_) => {
            println!("PCB problem solved successfully");
        }
        Err(e) => {
            println!("Failed to solve PCB problem: {}", e);
        }
    }
}
