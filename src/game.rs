use crate::heuristics::manhattan_distance;
use crate::search::{astar, State};
use serde::de::{MapAccess, Visitor};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
struct Block {
    position: Position2D,
    direction: Direction,
}

#[derive(Debug)]
pub struct Game {
    goals: HashMap<Color, Position2D>,
    arrows: HashMap<Position2D, Direction>,
    initial_state: HashMap<Color, Block>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            goals: HashMap::new(),
            arrows: HashMap::new(),
            initial_state: HashMap::new(),
        }
    }

    pub fn add_block(
        &mut self,
        color: Color,
        direction: Direction,
        starting_position: Position2D,
        goal_position: Option<Position2D>,
    ) {
        self.initial_state.insert(
            color.clone(),
            Block {
                position: starting_position,
                direction,
            },
        );
        if let Some(goal_position) = goal_position {
            self.goals.insert(color, goal_position);
        }
    }

    pub fn add_arrow(&mut self, direction: Direction, position: Position2D) {
        self.arrows.insert(position, direction);
    }

    pub fn solve(&self, max_moves: i32) -> Option<Vec<Color>> {
        let board_state = BoardState {
            game: self,
            cost: 0,
            squares: self.initial_state.clone(),
            move_history: vec![],
        };

        if let Some(state) = astar(board_state, max_moves) {
            Some(state.move_history)
        } else {
            None
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
    squares: HashMap<Color, Block>,
    move_history: Vec<Color>,
}

impl<'a> BoardState<'a> {
    fn move_square(&self, color: &Color) -> Self {
        let mut new_state = BoardState {
            game: self.game,
            cost: self.cost + 1,
            squares: self.squares.clone(),
            move_history: self.move_history.clone(),
        };
        new_state.move_history.push(color.clone());

        new_state.push_square(color, &self.squares.get(color).unwrap().direction);

        new_state
    }

    fn find_collision_with(&self, color: Color) -> Option<Color> {
        let block = self.squares.get(&color).unwrap();

        for (other_color, other_block) in self.squares.iter() {
            if other_color != &color && other_block.position == block.position {
                return Some(other_color.clone());
            }
        }

        None
    }

    fn push_square(&mut self, color: &Color, direction: &Direction) {
        let block = self.squares.get_mut(color).unwrap();

        block.position = match direction {
            Direction::Up => [block.position[0], block.position[1] + 1],
            Direction::Down => [block.position[0], block.position[1] - 1],
            Direction::Left => [block.position[0] - 1, block.position[1]],
            Direction::Right => [block.position[0] + 1, block.position[1]],
        };

        if let Some(new_direction) = self.game.arrows.get(&block.position) {
            block.direction = new_direction.clone();
        }

        if let Some(collided_block) = self.find_collision_with(color.clone()) {
            self.push_square(&collided_block, direction);
        }
    }
}

impl<'a> Ord for BoardState<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_cost = self.cost + self.distance_to_goal();
        let other_cost = other.cost + other.distance_to_goal();

        self_cost.cmp(&other_cost)
    }
}

impl<'a> PartialOrd for BoardState<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for BoardState<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.fingerprint() == other.fingerprint()
    }
}

impl<'a> Eq for BoardState<'a> {}

impl<'a> State<i32> for BoardState<'a> {
    fn successors(&self) -> Vec<Self> {
        self.squares.keys().map(|k| self.move_square(k)).collect()
    }

    fn is_goal(&self) -> bool {
        self.distance_to_goal() == 0
    }

    fn distance_to_goal(&self) -> i32 {
        self.game
            .goals
            .iter()
            .map(|(color, position)| {
                let block = self.squares.get(color).unwrap();
                manhattan_distance(&block.position, position)
            })
            .sum()
    }

    fn cost(&self) -> i32 {
        self.cost
    }

    fn fingerprint(&self) -> String {
        let mut keys: Vec<&Color> = self.squares.keys().collect();
        keys.sort();

        let mut fingerprint = String::new();

        for key in keys {
            let block = self.squares.get(key).unwrap();
            fingerprint.push_str(&format!(
                "{},{},{},{}\t",
                key, block.position[0], block.position[1], block.direction
            ));
        }

        fingerprint
    }
}
