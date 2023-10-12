use num::Num;
use std::cmp::Reverse;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BinaryHeap, HashSet};
use std::hash::{Hash, Hasher};

pub trait State: Hash + Sized {
    type Cost: Num + PartialOrd;

    fn successors(&self) -> Vec<Self>;
    fn is_goal(&self) -> bool;
    fn distance_to_goal(&self) -> Self::Cost;
    fn cost(&self) -> Self::Cost;
}

fn hash(state: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    state.hash(&mut hasher);
    hasher.finish()
}

struct StateContainer<T: State> {
    state: T,
}

impl<T: State> StateContainer<T> {
    fn new(state: T) -> Self {
        Self { state }
    }
}

impl<T: State> PartialEq for StateContainer<T> {
    fn eq(&self, other: &Self) -> bool {
        hash(&self.state) == hash(&other.state)
    }
}

impl<T: State> Eq for StateContainer<T> {}

impl<T: State> PartialOrd for StateContainer<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: State> Ord for StateContainer<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_cost = self.state.cost() + self.state.distance_to_goal();
        let other_cost = other.state.cost() + other.state.distance_to_goal();

        self_cost.partial_cmp(&other_cost).unwrap()
    }
}

pub fn astar<T: State>(initial_state: T, max_cost: T::Cost) -> Option<T> {
    let mut open_set = BinaryHeap::new();
    open_set.push(Reverse(StateContainer::new(initial_state)));
    let mut seen = HashSet::new();

    while let Some(reversed_state) = open_set.pop() {
        let state = reversed_state.0.state;

        if state.is_goal() {
            return Some(state);
        }

        if state.cost() < max_cost {
            for successor in state.successors() {
                let fingerprint = hash(&successor);

                if !seen.contains(&fingerprint) {
                    open_set.push(Reverse(StateContainer::new(successor)));
                    seen.insert(fingerprint);
                }
            }
        }
    }

    None
}
