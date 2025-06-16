use crate::heuristics::manhattan_distance;
use crate::search::{astar, State};
use serde::de::{MapAccess, Visitor};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Direction::Up => write!(f, "up"),
            Direction::Down => write!(f, "down"),
            Direction::Left => write!(f, "left"),
            Direction::Right => write!(f, "right"),
        }
    }
}

pub type Position2D = [i32; 2];

pub type Color = String;

#[derive(Clone, Debug, Deserialize, Hash)]
struct Block {
    position: Position2D,
    direction: Direction,
}

#[derive(Debug)]
pub struct Game {
    goals: Vec<Option<Position2D>>,
    arrows: HashMap<Position2D, Direction>,
    colors: Vec<Color>,
    color_idx_map: HashMap<Color, usize>,
    initial_state: Vec<Block>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            goals: Vec::new(),
            arrows: HashMap::new(),
            color_idx_map: HashMap::new(),
            colors: Vec::new(),
            initial_state: Vec::new(),
        }
    }

    pub fn add_block(
        &mut self,
        color: Color,
        direction: Direction,
        starting_position: Position2D,
        goal_position: Option<Position2D>,
    ) {
        if self.color_idx_map.get(&color).is_none() {
            self.color_idx_map.insert(color.clone(), self.colors.len());
            self.colors.push(color.clone());
            self.initial_state.push(Block {
                position: starting_position,
                direction: direction.clone(),
            });
            self.goals.push(goal_position);
        } else {
            let idx = self.color_idx_map.get(&color).unwrap();
            self.initial_state[*idx] = Block {
                position: starting_position,
                direction: direction.clone(),
            };
            self.goals[*idx] = goal_position;
        }
    }

    pub fn add_arrow(&mut self, direction: Direction, position: Position2D) {
        self.arrows.insert(position, direction);
    }

    pub fn solve(&self, max_moves: i32) -> Option<Vec<Color>> {
        let board_state = BoardState {
            game: self,
            cost: 0,
            previous_move: None,
            squares: self.initial_state.clone(),
        };

        match astar(board_state, max_moves) {
            Some(states) => {
                Some(states
                    .filter_map(|state| state.previous_move)
                    .map(|idx| self.colors[idx].clone())
                    .collect())
            },
            None => None,
        }
    }
}

impl<'de> Deserialize<'de> for Game {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct GameVisitor;

        #[derive(Deserialize)]
        struct SerializedBlock {
            color: Color,
            direction: Direction,
            position: Position2D,
            goal: Option<Position2D>,
        }

        #[derive(Deserialize)]
        struct SerializedArrow {
            direction: Direction,
            position: Position2D,
        }

        impl<'de> Visitor<'de> for GameVisitor {
            type Value = Game;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a game with values for blocks and (optionally) arrows")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Game, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut game = Game::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "blocks" => {
                            let blocks: Vec<SerializedBlock> = map.next_value()?;
                            for block in blocks {
                                game.add_block(
                                    block.color,
                                    block.direction,
                                    block.position,
                                    block.goal,
                                );
                            }
                        }
                        "arrows" => {
                            let arrows: Vec<SerializedArrow> = map.next_value()?;
                            for arrow in arrows {
                                game.add_arrow(arrow.direction, arrow.position);
                            }
                        }
                        _ => {
                            return Err(serde::de::Error::unknown_field(
                                &key,
                                &["blocks", "arrows"],
                            ));
                        }
                    }
                }

                Ok(game)
            }
        }

        deserializer.deserialize_map(GameVisitor)
    }
}

#[derive(Clone, Debug)]
struct BoardState<'a> {
    game: &'a Game,
    cost: i32,
    previous_move: Option<usize>,
    squares: Vec<Block>,
}

impl<'a> BoardState<'a> {
    fn move_square(&self, color_idx: usize) -> Self {
        let mut new_state = Self {
            game: self.game,
            cost: self.cost + 1,
            previous_move: Some(color_idx),
            squares: self.squares.clone(),
        };
        let direction = new_state.squares[color_idx].direction.clone();
        new_state.push_square(color_idx, &direction);

        new_state
    }

    fn find_collision_with(&self, color_idx: usize) -> Option<usize> {
        let block = &self.squares[color_idx];

        for idx in 0..self.squares.len() {
            if idx != color_idx {
                let other_block = &self.squares[idx];
                if other_block.position == block.position {
                    return Some(idx);
                }
            }
        }

        None
    }

    fn push_square(&mut self, color_idx: usize, direction: &Direction) {
        let block = &mut self.squares[color_idx];

        block.position = match direction {
            Direction::Up => [block.position[0], block.position[1] + 1],
            Direction::Down => [block.position[0], block.position[1] - 1],
            Direction::Left => [block.position[0] - 1, block.position[1]],
            Direction::Right => [block.position[0] + 1, block.position[1]],
        };

        if let Some(new_direction) = self.game.arrows.get(&block.position) {
            block.direction = new_direction.clone();
        }

        if let Some(collided_idx) = self.find_collision_with(color_idx) {
            self.push_square(collided_idx, direction);
        }
    }
}

impl<'a> Hash for BoardState<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.squares.hash(state);
    }
}

impl<'a> State for BoardState<'a> {
    type Cost = i32;

    fn successors(&self) -> impl Iterator<Item = Self> {
        let count = self.game.colors.len();
        let mut children = Vec::with_capacity(count);

        for color_idx in 0..count {
            children.push(self.move_square(color_idx));
        }

        children.into_iter()
    }

    fn is_goal(&self) -> bool {
        self.distance_to_goal() == 0
    }

    fn distance_to_goal(&self) -> Self::Cost {
        let mut sum = 0;

        for idx in 0..self.game.colors.len() {
            let block = &self.squares[idx];
            if let Some(goal_position) = &self.game.goals[idx] {
                sum += manhattan_distance(&block.position, goal_position);
            }
        }

        sum
    }

    fn cost(&self) -> Self::Cost {
        self.cost
    }
}
