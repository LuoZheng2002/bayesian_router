use std::{
    process::exit,
    sync::{Arc, Mutex},
};

use cgmath::Deg;

use parser::{parse_end_to_end::parse_end_to_end, write_ses::write_ses};
use router::{
    pcb_problem_solve::solve_pcb_problem, test_pcb_problem::pcb_problem1,
    test_pcb_problem::pcb_problem2,
};
use shared::pcb_render_model::PcbRenderModel;

pub fn working_thread_fn(pcb_render_model: Arc<Mutex<Option<PcbRenderModel>>>) {
    println!("Working thread started");
    let pcb_problem = pcb_problem2();

    // let dsn_file_content = std::fs::read_to_string("vimdrones_esc_development_board.dsn").unwrap();
    // let pcb_problem = match parse_end_to_end(dsn_file_content) {
    //     Ok(problem) => problem,
    //     Err(e) => {
    //         println!("Failed to parse DSN file: {}", e);
    //         exit(-1);
    //     }
    // };
    let result = solve_pcb_problem(&pcb_problem, pcb_render_model.clone());
    let result = match result {
        Ok(result) => {
            println!("PCB problem solved successfully");
            result
        }
        Err(e) => {
            println!("Failed to solve PCB problem: {}", e);
            exit(0);
        }
    };
    write_ses(dsn, solution, output)
}
