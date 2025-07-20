use std::sync::atomic::Ordering;

use router::command_flags::{COMMAND_CVS, COMMAND_LEVEL};





pub fn command_thread_fn() {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim(){
            "" =>{
                for cv in COMMAND_CVS.iter(){
                    cv.notify_all(); // Notify all command flags to proceed
                }
            },
            "i" =>{
                let result = COMMAND_LEVEL.fetch_sub(1, Ordering::SeqCst);
                if result == u8::MAX{
                    println!("Warning: command level below 0, resetting to 0");
                    COMMAND_LEVEL.store(0, Ordering::SeqCst);
                }
            },
            "o" =>{
                let result = COMMAND_LEVEL.fetch_add(1, Ordering::SeqCst);
                if result > 3{
                    println!("Warning: command level above 3, resetting to 3");
                    COMMAND_LEVEL.store(3, Ordering::SeqCst);
                }
            },
            _ =>{
                println!("Unknown command");
            }
        }
    }
}