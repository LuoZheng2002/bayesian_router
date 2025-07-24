use cgmath::Deg;
use shared::{
    collider::{BorderCollider, Collider, PolygonCollider},
    prim_shape::{Line, PrimShape, RectangleShape},
    vec2::FloatVec2,
};

const MAX_OBJECTS: usize = 4;
const MAX_DEPTH: usize = 10;

pub struct QuadTreeChildren {
    top_left: Box<QuadTreeNode>,
    top_right: Box<QuadTreeNode>,
    bottom_left: Box<QuadTreeNode>,
    bottom_right: Box<QuadTreeNode>,
}
impl QuadTreeChildren {
    pub fn new(
        parent_x_min: f32,
        parent_x_max: f32,
        parent_y_min: f32,
        parent_y_max: f32,
        parent_depth: usize,
    ) -> Self {
        let mid_x = (parent_x_min + parent_x_max) / 2.0;
        let mid_y = (parent_y_min + parent_y_max) / 2.0;

        Self {
            top_left: Box::new(QuadTreeNode::new(
                parent_x_min,
                mid_x,
                parent_y_min,
                mid_y,
                parent_depth + 1,
            )),
            top_right: Box::new(QuadTreeNode::new(
                mid_x,
                parent_x_max,
                parent_y_min,
                mid_y,
                parent_depth + 1,
            )),
            bottom_left: Box::new(QuadTreeNode::new(
                parent_x_min,
                mid_x,
                mid_y,
                parent_y_max,
                parent_depth + 1,
            )),
            bottom_right: Box::new(QuadTreeNode::new(
                mid_x,
                parent_x_max,
                mid_y,
                parent_y_max,
                parent_depth + 1,
            )),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &QuadTreeNode> {
        [
            &*self.top_left,
            &*self.top_right,
            &*self.bottom_left,
            &*self.bottom_right,
        ]
        .into_iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut QuadTreeNode> {
        [
            &mut *self.top_left,
            &mut *self.top_right,
            &mut *self.bottom_left,
            &mut *self.bottom_right,
        ]
        .into_iter()
    }
    pub fn insert(&mut self, collider: Collider) -> bool {
        for child in self.iter_mut() {
            if child.insert(collider.clone()) {
                return true;
            }
        }
        false
    }
}
pub struct QuadTreeNode {
    pub depth: usize,
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub objects: Vec<Collider>,
    pub children: Option<QuadTreeChildren>,
}
impl QuadTreeNode {
    pub fn new(x_min: f32, x_max: f32, y_min: f32, y_max: f32, depth: usize) -> Self {
        Self {
            depth,
            x_min,
            x_max,
            y_min,
            y_max,
            objects: Vec::new(),
            children: None,
        }
    }
    /// helper function that is called by insert and query
    fn fully_contained_in_boundary(&self, collider: &Collider) -> bool {
        let left_border = BorderCollider {
            point_on_border: FloatVec2::new(self.x_min, 0.0),
            normal: FloatVec2::new(-1.0, 0.0),
        };
        if collider.collides_with(&Collider::Border(left_border)) {
            return false; // collider collides with the left border, so it is not fully contained
        }
        let right_border = BorderCollider {
            point_on_border: FloatVec2::new(self.x_max, 0.0),
            normal: FloatVec2::new(1.0, 0.0),
        };
        if collider.collides_with(&Collider::Border(right_border)) {
            return false; // collider collides with the right border, so it is not fully contained
        }
        let top_border = BorderCollider {
            point_on_border: FloatVec2::new(0.0, self.y_max),
            normal: FloatVec2::new(0.0, 1.0),
        };
        if collider.collides_with(&Collider::Border(top_border)) {
            return false; // collider collides with the top border, so it is not fully contained
        }
        let bottom_border = BorderCollider {
            point_on_border: FloatVec2::new(0.0, self.y_min),
            normal: FloatVec2::new(0.0, -1.0),
        };
        if collider.collides_with(&Collider::Border(bottom_border)) {
            return false; // collider collides with the bottom border, so it is not fully contained
        }
        true
    }
    fn partially_contained_in_boundary(&self, collider: &Collider) -> bool {
        let polygon_collider = PolygonCollider(vec![
            FloatVec2::new(self.x_min, self.y_min),
            FloatVec2::new(self.x_max, self.y_min),
            FloatVec2::new(self.x_max, self.y_max),
            FloatVec2::new(self.x_min, self.y_max),
        ]);
        collider.collides_with(&Collider::Polygon(polygon_collider))
    }
    pub fn insert(&mut self, collider: Collider) -> bool {
        if !self.fully_contained_in_boundary(&collider) {
            return false; // shape is not fully contained in this node's boundary
        }
        let max_depth_reached = self.depth >= MAX_DEPTH;
        let has_children = self.children.is_some();
        let max_objects_reached = self.objects.len() >= MAX_OBJECTS;
        match (max_depth_reached, has_children, max_objects_reached) {
            (true, false, _) // max_depth_reached and assert no children, push to objects directly
            |(false, false, false) // else if no children and not reached max objects, push to objects directly
            => {
                // if we have reached the max depth, we can insert directly
                self.objects.push(collider);
            },
            (false, false, true) // if we have not reached the max depth, but have reached the max objects, we need to create children
                 => {
                assert!(self.children.is_none(), "A node that has not reached the max depth should not have children");
                self.children = Some(QuadTreeChildren::new(self.x_min, self.x_max, self.y_min, self.y_max, self.depth));
                let children = self.children.as_mut().unwrap();
                // try push all the existing objects into the children
                let existing_shapes = std::mem::take(&mut self.objects);
                for existing_shape in existing_shapes {
                    if !children.insert(existing_shape.clone()) {
                        // if the children could not insert, we push to objects
                        self.objects.push(existing_shape);
                    }
                }
                // insert the new shape into the children
                if !children.insert(collider.clone()){
                    self.objects.push(collider); // if the children could not insert, we push to objects
                }
            },
            (false, true, _) // if we have children, assert not reached max depth, and don't care about whether max objects reached
            =>{
                let children = self.children.as_mut().unwrap();
                if !children.insert(collider.clone()) {
                    // if the children could not insert, we push to objects
                    self.objects.push(collider);
                }
            }
            (true, true, _)=>{
                panic!("A node that has reached the max depth should not have children");
            }
        };
        true
    }
    pub fn extend(&mut self, colliders: impl Iterator<Item = Collider>) {
        for collider in colliders {
            self.insert(collider);
        }
    }

    pub fn collides_with(&self, collider: &Collider) -> bool {
        // query all the shapes that have a potential to collide with the given shape
        if !self.partially_contained_in_boundary(collider) {
            return false; // shape is not fully contained in this node's boundary
        }
        for object in &self.objects {
            if object.collides_with(collider) {
                return true; // found a collision
            }
        }
        if let Some(children) = &self.children {
            for child in children.iter() {
                if child.collides_with(collider) {
                    return true; // found a collision in the children
                }
            }
        }
        false
    }
    pub fn collides_with_set<'a>(&self, colliders: impl Iterator<Item = &'a Collider>) -> bool {
        // query all the shapes that have a potential to collide with the given set of shapes
        for collider in colliders {
            if self.collides_with(collider) {
                return true; // found a collision
            }
        }
        false
    }
    pub fn to_outline_shapes(&self) -> Vec<PrimShape> {
        let mut shapes = vec![
            PrimShape::Line(Line {
                start: FloatVec2::new(self.x_min, self.y_min),
                end: FloatVec2::new(self.x_max, self.y_min),
            }),
            PrimShape::Line(Line {
                start: FloatVec2::new(self.x_max, self.y_min),
                end: FloatVec2::new(self.x_max, self.y_max),
            }),
            PrimShape::Line(Line {
                start: FloatVec2::new(self.x_max, self.y_max),
                end: FloatVec2::new(self.x_min, self.y_max),
            }),
            PrimShape::Line(Line {
                start: FloatVec2::new(self.x_min, self.y_max),
                end: FloatVec2::new(self.x_min, self.y_min),
            }),
        ];
        if let Some(children) = &self.children {
            for child in children.iter() {
                shapes.extend(child.to_outline_shapes());
            }
        }
        shapes
    }
}
