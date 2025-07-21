use std::sync::Mutex;

use lazy_static::lazy_static;
use rand::{SeedableRng, rngs::StdRng};

pub fn create_deterministic_rng() -> StdRng {
    let seed = 42; // Fixed seed for reproducibility
    StdRng::seed_from_u64(seed)
}
