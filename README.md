# Solver of Squares (in Rust)

This is a solver for [Game about squares](http://gameaboutsquares.com).

It implements A* with a non-admissable heuristic, so the outputs are not guaranteed to be optimal.
But it's fun anyway.

## Run

The executable takes a path to a YAML file describing the blocks and arrows for the puzzle.
It will calculate a solution and print the number of moves required and the ordering of the colors to complete the puzzle.

`cargo run -- ./levels/level_31.yaml`