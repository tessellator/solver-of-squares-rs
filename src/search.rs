use num::Num;
use std::cmp::Reverse;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BinaryHeap, HashSet};
use std::hash::{Hash, Hasher};

pub trait State<T: Num>: Sized + Ord + Hash {
    fn successors(&self) -> Vec<Self>;
    fn is_goal(&self) -> bool;
    fn distance_to_goal(&self) -> T;
    fn cost(&self) -> T;
}

fn hash<T: Num, S: State<T>>(state: &S) -> u64 {
    let mut hasher = DefaultHasher::new();
    state.hash(&mut hasher);
    hasher.finish()
}

pub fn astar<T: State<N>, N: Num + PartialOrd>(initial_state: T, max_cost: N) -> Option<T> {
    let mut open_set = BinaryHeap::new();
    open_set.push(Reverse(initial_state));
    let mut seen = HashSet::new();

    while let Some(reversed_state) = open_set.pop() {
        let state = reversed_state.0;

        if state.is_goal() {
            return Some(state);
        }

        if state.cost() < max_cost {
            for successor in state.successors() {
                let fingerprint = hash(&successor);

                if !seen.contains(&fingerprint) {
                    open_set.push(Reverse(successor));
                    seen.insert(fingerprint);
                }
            }
        }
    }

    None
}
