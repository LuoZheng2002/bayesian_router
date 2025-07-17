use std::sync::{Arc, Mutex};

use cgmath::Deg;

use parser::parse_end_to_end::parse_end_to_end;
use router::{
    pcb_problem_solve::solve_pcb_problem, test_pcb_problem::pcb_problem1,
    test_pcb_problem::pcb_problem2,
};
use shared::pcb_render_model::PcbRenderModel;

pub fn working_thread_fn(pcb_render_model: Arc<Mutex<PcbRenderModel>>) {
    println!("Working thread started");
    let pcb_problem = pcb_problem1();

    // let dsn_file_content = std::fs::read_to_string("specctra_test.dsn").unwrap();
    // let pcb_problem = match parse_end_to_end(dsn_file_content) {
    //     Ok(problem) => problem,
    //     Err(e) => {
    //         println!("Failed to parse DSN file: {}", e);
    //         return;
    //     }
    // };
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
