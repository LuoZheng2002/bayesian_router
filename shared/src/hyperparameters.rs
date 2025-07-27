use std::{collections::HashMap, num::NonZeroUsize, sync::{atomic::AtomicUsize, Mutex}};

use lazy_static::lazy_static;

use crate::{color_float3::ColorFloat3, vec2::FixedPoint};

pub const HALF_PROBABILITY_RAW_SCORE: f64 = 10.0;
pub const HALF_PROBABILITY_OPPORTUNITY_COST: f64 = 0.5;

// pub const MAX_TRACES_PER_ITERATION: usize = 4; // Maximum number of traces per iteration
pub const MAX_GENERATION_ATTEMPTS: usize = 4; // Maximum number of attempts to generate a trace

pub const FIRST_ITERATION_SUM_PROBABILITY: f64 = 0.5; // Probability for the first iteration
pub const SECOND_ITERATION_SUM_PROBABILITY: f64 = 0.25; //
pub const THIRD_ITERATION_SUM_PROBABILITY: f64 = 0.125; // Probability for the third iteration
pub const FOURTH_ITERATION_SUM_PROBABILITY: f64 = 0.0625; // Probability for the fourth iteration

pub const FIRST_ITERATION_NUM_TRACES: usize = 1;
pub const SECOND_ITERATION_NUM_TRACES: usize = 3;
pub const THIRD_ITERATION_NUM_TRACES: usize = 4;
pub const FOURTH_ITERATION_NUM_TRACES: usize = 2;

// pub const BLOCK_THREAD: bool = true; // Whether to block the thread when waiting for a trace to be generated
// pub const DISPLAY_ASTAR: bool = true; // Whether to display the A* search process
pub const DISPLAY_OPTIMIZATION: bool = false; // Whether to display the optimization process
pub const OPTIMIZATION_PRO: bool = true;
pub const DISPLAY_PERIOD_MILLIS: u64 = 10;

pub const MAX_ITERATION: NonZeroUsize =
    NonZeroUsize::new(4).expect("MAX_ITERATION must be non-zero");

pub const MAX_TRIALS: usize = 1000; // Maximum number of trials to find a trace

pub const LINEAR_LEARNING_RATE: f64 = 0.2;
pub const CONSTANT_LEARNING_RATE: f64 = 0.01;

pub const TURN_PENALTY: f64 = 1.0;

pub const ESTIMATE_COEFFICIENT: f64 = 1.0;

pub const VIA_COST: f64 = 5.0; // Cost of placing a via

pub const NUM_TOP_RANKED_TO_TRY: usize = 3; // Number of top-ranked traces to try fixing in each iteration

pub const SAMPLE_ITERATIONS: usize = 2;

pub const UPDATE_PROBA_SKIP_STRIDE: usize = 2;

pub const LAYER_TO_TRACE_COLOR: [ColorFloat3; 4] = [
    ColorFloat3::new(1.0, 0.0, 0.0), // Red for front layer
    ColorFloat3::new(0.0, 0.0, 1.0), // Blue for back layer
    ColorFloat3::new(1.0, 1.0, 0.0), // Yellow for top layer
    ColorFloat3::new(0.0, 1.0, 0.0), // Green for bottom layer
];

lazy_static! {
    pub static ref SAMPLE_CNT: AtomicUsize = AtomicUsize::new(0); // Global counter for the number of samples taken
    pub static ref ASTAR_STRIDE: FixedPoint = {
        let raw_stride: f32 = 2.00;
        let mut result = FixedPoint::from_num(raw_stride);
        let result_bits = result.to_bits();
        if result_bits & 1 == 1{
            result += FixedPoint::DELTA;
        }
        assert!(result.to_bits() & 1 == 0, "A* stride must be even");
        result
    }; // A* search stride
    pub static ref SCORE_WEIGHT: Mutex<f64> = Mutex::new(1.0);
    pub static ref OPPORTUNITY_COST_WEIGHT: Mutex<f64> = Mutex::new(0.0);
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
            1.0
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
