use std::sync::atomic::Ordering;

use router::command_flags::{COMMAND_CVS, COMMAND_LEVEL, COMMAND_MUTEXES};

pub fn command_thread_fn() {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "" => {
                for cv in COMMAND_CVS.iter() {
                    cv.notify_all(); // Notify all command flags to proceed
                }
            }
            "i" => {
                let result = COMMAND_LEVEL.fetch_sub(1, Ordering::SeqCst);
                println!(
                    "Command level decremented to {}",
                    result.checked_sub(1).unwrap_or(0)
                );
                if result == 0 {
                    println!("Warning: command level below 0, resetting to 0");
                    COMMAND_LEVEL.store(0, Ordering::SeqCst);
                }
            }
            "o" => {
                let max_level = COMMAND_MUTEXES.len() as u8 - 1;
                let result = COMMAND_LEVEL.fetch_add(1, Ordering::SeqCst);
                println!("Command level incremented to {}", result + 1);
                if result >= max_level {
                    println!(
                        "Warning: command level above {}, resetting to {}",
                        max_level, max_level
                    );
                    COMMAND_LEVEL.store(max_level, Ordering::SeqCst);
                }
            }
            _ => {
                println!("Unknown command");
            }
        }
    }
}
