use std::collections::{BinaryHeap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

fn hash(state: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    state.hash(&mut hasher);
    hasher.finish()
}

pub trait State: Hash {
    type Cost: num::Num + PartialOrd;

    fn successors(&self) -> impl Iterator<Item = Self>;
    fn is_goal(&self) -> bool;
    fn distance_to_goal(&self) -> Self::Cost;
    fn cost(&self) -> Self::Cost;
}

struct Node<T: State> {
    depth: usize,
    state: T,
    parent: Option<Rc<Node<T>>>,
}

impl<T: State> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        hash(&self.state) == hash(&other.state)
    }
}

impl<T: State> Eq for Node<T> {}

impl<T: State> PartialOrd for Node<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.cmp(self)) // Reverse order for min-heap
    }
}

impl<T: State> Ord for Node<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_cost = self.state.cost() + self.state.distance_to_goal();
        let other_cost = other.state.cost() + other.state.distance_to_goal();

        self_cost
            .partial_cmp(&other_cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn node_to_path<T: State>(node: Rc<Node<T>>) -> impl Iterator<Item = T> {
    let mut path = Vec::with_capacity(node.depth + 1);
    let mut current = Some(node);

    while let Some(n) = current {
        let inner = Rc::into_inner(n).unwrap();
        path.push(inner.state);
        current = inner.parent.clone();
    }

    path.into_iter().rev()
}

pub fn astar<T: State>(initial_state: T, max_cost: T::Cost) -> Option<impl Iterator<Item = T>> {
    let mut open_set = BinaryHeap::new();
    let mut seen = HashSet::new();

    open_set.push(Node {
        depth: 0,
        state: initial_state,
        parent: None,
    });

    while let Some(node) = open_set.pop() {
        if node.state.is_goal() {
            drop(open_set);
            return Some(node_to_path(Rc::new(node)).into_iter());
        }

        if node.state.cost() < max_cost {
            let new_depth = node.depth + 1;
            let parent = Rc::new(node);
            for successor in parent.state.successors() {
                let hash = hash(&successor);

                if !seen.contains(&hash) {
                    seen.insert(hash);
                    open_set.push(Node {
                        depth: new_depth,
                        state: successor,
                        parent: Some(parent.clone()),
                    });
                }
            }
        }
    }

    None
}
