use std::sync::Mutex;

use lazy_static::lazy_static;
use rand::{SeedableRng, rngs::StdRng};
use rand_chacha::ChaCha12Rng;

pub fn create_deterministic_rng() -> ChaCha12Rng {
    let seed = 42; // Fixed seed for reproducibility
    let rng = ChaCha12Rng::from_seed([seed as u8; 32]);
    rng
}
