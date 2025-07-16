use std::{collections::HashMap, num::NonZeroUsize, sync::Mutex};

use lazy_static::lazy_static;

use crate::vec2::FixedPoint;

pub const HALF_PROBABILITY_RAW_SCORE: f64 = 10.0;

pub const MAX_TRACES_PER_ITERATION: usize = 4; // Maximum number of traces per iteration
pub const MAX_GENERATION_ATTEMPTS: usize = 10; // Maximum number of attempts to generate a trace

pub const FIRST_ITERATION_SUM_PROBABILITY: f64 = 0.5; // Probability for the first iteration
pub const SECOND_ITERATION_SUM_PROBABILITY: f64 = 0.25; //
pub const THIRD_ITERATION_SUM_PROBABILITY: f64 = 0.125; // Probability for the third iteration
pub const FOURTH_ITERATION_SUM_PROBABILITY: f64 = 0.0625; // Probability for the fourth iteration

pub const FIRST_ITERATION_NUM_TRACES: usize = 1;
pub const SECOND_ITERATION_NUM_TRACES: usize = 3;
pub const THIRD_ITERATION_NUM_TRACES: usize = 4;
pub const FOURTH_ITERATION_NUM_TRACES: usize = 2;

pub const BLOCK_THREAD: bool = false; // Whether to block the thread when waiting for a trace to be generated
pub const DISPLAY_ASTAR: bool = true; // Whether to display the A* search process
pub const DISPLAY_PERIOD_MILLIS: u64 = 10;

pub const MAX_ITERATION: NonZeroUsize =
    NonZeroUsize::new(4).expect("MAX_ITERATION must be non-zero");

pub const LINEAR_LEARNING_RATE: f64 = 0.2;
pub const CONSTANT_LEARNING_RATE: f64 = 0.01;

pub const TURN_PENALTY: f64 = 1.0;

pub const ESTIMATE_COEFFICIENT: f64 = 1.0;

lazy_static! {
    pub static ref ASTAR_STRIDE: FixedPoint = {
        let result = FixedPoint::from_num(1.27) + FixedPoint::DELTA;
        let result_bits = result.to_bits();
        if result_bits & 1 == 0{
            println!("A* search stride is even.");
        }else{
            println!("A* search stride is odd.");
        }
        result
    }; // A* search stride
    pub static ref SCORE_WEIGHT: Mutex<f64> = Mutex::new(0.3);
    pub static ref OPPORTUNITY_COST_WEIGHT: Mutex<f64> = Mutex::new(0.3);
    pub static ref ITERATION_TO_PRIOR_PROBABILITY: HashMap<NonZeroUsize, f64> = vec![
        (
            NonZeroUsize::new(1).unwrap(),
            FIRST_ITERATION_SUM_PROBABILITY / FIRST_ITERATION_NUM_TRACES as f64
        ),
        (
            NonZeroUsize::new(2).unwrap(),
            SECOND_ITERATION_SUM_PROBABILITY / SECOND_ITERATION_NUM_TRACES as f64
        ),
        (
            NonZeroUsize::new(3).unwrap(),
            THIRD_ITERATION_SUM_PROBABILITY / THIRD_ITERATION_NUM_TRACES as f64
        ),
        (
            NonZeroUsize::new(4).unwrap(),
            FOURTH_ITERATION_SUM_PROBABILITY / FOURTH_ITERATION_NUM_TRACES as f64
        ),
    ]
    .into_iter()
    .collect();
    pub static ref NEXT_ITERATION_TO_REMAINING_PROBABILITY: HashMap<NonZeroUsize, f64> = vec![
        (
            NonZeroUsize::new(1).unwrap(),
            1.0 - FIRST_ITERATION_SUM_PROBABILITY
        ),
        (
            NonZeroUsize::new(2).unwrap(),
            1.0 - FIRST_ITERATION_SUM_PROBABILITY
        ),
        (
            NonZeroUsize::new(3).unwrap(),
            1.0 - FIRST_ITERATION_SUM_PROBABILITY - SECOND_ITERATION_SUM_PROBABILITY
        ),
        (
            NonZeroUsize::new(4).unwrap(),
            1.0 - FIRST_ITERATION_SUM_PROBABILITY
                - SECOND_ITERATION_SUM_PROBABILITY
                - THIRD_ITERATION_SUM_PROBABILITY
        ),
        (
            NonZeroUsize::new(5).unwrap(),
            1.0 - FIRST_ITERATION_SUM_PROBABILITY
                - SECOND_ITERATION_SUM_PROBABILITY
                - THIRD_ITERATION_SUM_PROBABILITY
                - FOURTH_ITERATION_SUM_PROBABILITY
        ),
    ]
    .into_iter()
    .collect();
    pub static ref ITERATION_TO_NUM_TRACES: HashMap<NonZeroUsize, usize> = vec![
        (NonZeroUsize::new(1).unwrap(), FIRST_ITERATION_NUM_TRACES),
        (NonZeroUsize::new(2).unwrap(), SECOND_ITERATION_NUM_TRACES),
        (NonZeroUsize::new(3).unwrap(), THIRD_ITERATION_NUM_TRACES),
        (NonZeroUsize::new(4).unwrap(), FOURTH_ITERATION_NUM_TRACES),
    ]
    .into_iter()
    .collect();
}
