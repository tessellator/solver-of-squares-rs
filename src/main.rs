mod game;
mod heuristics;
mod search;

use std::env;
use std::fs::File;

use game::Game;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];
    let file = File::open(path).expect("could not open file");
    let game: Game = serde_yaml::from_reader(file).expect("could not parse input file");

    if let Some(moves) = game.solve(50) {
        println!("Solution found with {} moves", moves.len());
        println!("Moves: {:?}", moves);
    } else {
        println!("No solution found");
    }
}
