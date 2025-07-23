use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct BinaryHeapItem<T, U> {
    pub key: T,   // used for ordering
    pub value: U, // not used for ordering
}
impl<T: Ord, U> BinaryHeapItem<T, U> {
    pub fn new(key: T, value: U) -> Self {
        BinaryHeapItem { key, value }
    }
}

// Implement Eq and Ord based on key
impl<T: Ord, U> PartialEq for BinaryHeapItem<T, U> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: Ord, U> Eq for BinaryHeapItem<T, U> {}

impl<T: Ord, U> PartialOrd for BinaryHeapItem<T, U> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord, U> Ord for BinaryHeapItem<T, U> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}
