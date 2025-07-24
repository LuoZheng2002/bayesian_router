use std::{
    process::exit,
    sync::{Arc, Mutex},
};

use cgmath::Deg;

use parser::{parse_end_to_end::{parse_end_to_end, parse_start_to_dsn_struct, parse_struct_to_end}, write_ses::write_ses};
use router::{
    naive_backtrack_algo::naive_backtrack, pcb_problem_solve::solve_pcb_problem
};
use shared::pcb_render_model::PcbRenderModel;

pub fn working_thread_fn(pcb_render_model: Arc<Mutex<Option<PcbRenderModel>>>) {
    println!("Working thread started");
    // let pcb_problem = pcb_problem2();

    let dsn_file_content = std::fs::read_to_string("Digistump_ATtiny.dsn").unwrap();
    let dsn_struct = match parse_start_to_dsn_struct(dsn_file_content.clone()) {
        Ok(structure) => structure,
        Err(e) => {
            println!("Failed to parse DSN file: {}", e);
            exit(-1);
        }
    };
    let pcb_problem = match parse_struct_to_end(&dsn_struct) {
        Ok(problem) => problem,
        Err(e) => {
            println!("Failed to parse DSN file: {}", e);
            exit(-1);
        }
    };
    let result = solve_pcb_problem(&pcb_problem, pcb_render_model.clone(), false);
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
    match write_ses(&dsn_struct, &result, "a"){
        Ok(_) => println!("SES file written successfully"),
        Err(e) => {
            println!("Failed to write SES file: {}", e);
            exit(-1);
        }
    }
}
