use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet},
    f32::INFINITY,
};

use ordered_float::OrderedFloat;
use shared::{octile_distance, pad::PadName, vec2::FloatVec2};

pub fn prim_mst(pad_positions: HashMap<PadName, FloatVec2>) -> Vec<(PadName, PadName)> {
    if pad_positions.is_empty() {
        return Vec::new();
    }
    let mut mst_edges = Vec::new();
    let mut visited = HashSet::new();
    let mut remaining_pads: HashSet<PadName> = pad_positions.keys().cloned().collect();

    // Start with an arbitrary node
    let start_pad = remaining_pads.iter().next().unwrap().clone();
    visited.insert(start_pad.clone());
    remaining_pads.remove(&start_pad);

    while !remaining_pads.is_empty() {
        let mut min_edge: Option<(PadName, PadName)> = None;
        let mut min_distance = INFINITY;

        // For each visited node, find the closest unvisited neighbor
        for visited_pad in &visited {
            let visited_pos = pad_positions[visited_pad];
            for candidate_pad in &remaining_pads {
                let candidate_pos = pad_positions[candidate_pad];
                let distance = octile_distance::octile_distance_float(visited_pos, candidate_pos);

                if distance < min_distance {
                    min_distance = distance;
                    min_edge = Some((visited_pad.clone(), candidate_pad.clone()));
                }
            }
        }

        if let Some((from_pad, to_pad)) = min_edge {
            mst_edges.push((from_pad.clone(), to_pad.clone()));
            visited.insert(to_pad.clone());
            remaining_pads.remove(&to_pad);
        } else {
            // This case shouldn't happen if the graph is connected
            break;
        }
    }

    mst_edges
}