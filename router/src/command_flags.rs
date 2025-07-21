use std::sync::{Condvar, Mutex, atomic::AtomicU8};

use lazy_static::lazy_static;

pub enum CommandFlag {
    AstarFrontierOrUpdatePosterior,
    AstarInOut,
    UpdatePosteriorResult,
    ProbaModelResult,
    Auto,
}

impl CommandFlag {
    pub fn get_level(&self) -> u8 {
        match self {
            CommandFlag::AstarFrontierOrUpdatePosterior => 0,
            CommandFlag::AstarInOut => 1,
            CommandFlag::UpdatePosteriorResult => 2,
            CommandFlag::ProbaModelResult => 3,
            CommandFlag::Auto => 4,
        }
    }
}

lazy_static! {

    pub static ref COMMAND_MUTEXES: [Mutex<()>; 5] = [
        Mutex::new(()), // For CommandFlag::AstarFrontierOrUpdatePosterior
        Mutex::new(()), // For CommandFlag::AstarInOut
        Mutex::new(()), // For CommandFlag::UpdatePosteriorResult
        Mutex::new(()), // For CommandFlag::ProbaModelResult
        Mutex::new(()), // For CommandFlag::Auto
    ];
    pub static ref COMMAND_CVS: [Condvar; 5] = [
        Condvar::new(), // For CommandFlag::AstarFrontierOrUpdatePosterior
        Condvar::new(), // For CommandFlag::AstarInOut
        Condvar::new(), // For CommandFlag::UpdatePosteriorResult
        Condvar::new(), // For CommandFlag::ProbaModelResult
        Condvar::new(), // For CommandFlag::Auto
    ];
    pub static ref COMMAND_LEVEL: AtomicU8 = AtomicU8::new(0);
}
