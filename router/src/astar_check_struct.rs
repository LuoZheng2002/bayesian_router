use std::{collections::HashMap, rc::Rc};

use shared::{collider::Collider, trace_path::TracePath};

use crate::quad_tree::QuadTreeNode;





pub struct AStarCheck{
    pub border_colliders: Rc<Vec<Collider>>,
    pub obstacle_colliders: Rc<HashMap<usize, QuadTreeNode>>,
    pub obstacle_clearance_colliders: Rc<HashMap<usize, QuadTreeNode>>,
    pub solution_trace: TracePath,    
    pub num_layers: usize,
}

impl AStarCheck{
    pub fn check(&self) -> bool {
        // check with border colliders
        let trace_colliders = self.solution_trace.to_colliders(self.num_layers);
        let trace_clearance_colliders = self.solution_trace.to_clearance_colliders(self.num_layers);
        for border_collider in &*self.border_colliders {
            for (_, trace_colliders) in &trace_colliders {
                for trace_collider in trace_colliders {
                    if border_collider.collides_with(trace_collider) {
                        println!("Collision with border collider: {:?}", border_collider);
                        return false; // Collision with border collider
                    }
                }
            }
        }
        for (layer, obstacle_colliders) in &*self.obstacle_colliders{
            let trace_clearance_colliders = trace_clearance_colliders.get(&layer).unwrap();
            if obstacle_colliders.collides_with_set(trace_clearance_colliders.iter()){
                println!("Collision between obstacle colliders and trace clearance colliders on layer {}", layer);
                return false; // Collision with obstacle colliders
            }
        }
        for (layer, obstacle_clearance_colliders) in &*self.obstacle_clearance_colliders {
            let trace_colliders = trace_colliders.get(&layer).unwrap();
            if obstacle_clearance_colliders.collides_with_set(trace_colliders.iter()) {
                println!("Collision between obstacle clearance colliders and trace colliders on layer {}", layer);
                return false; // Collision with obstacle clearance colliders
            }
        }
        true
    }
}