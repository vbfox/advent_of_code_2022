use std::{collections::HashSet, fmt::Display, str::FromStr, time::Instant};

use eyre::eyre;
use itertools::Itertools;

use crate::utils::Vec2D;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Up => write!(f, "U"),
            Direction::Down => write!(f, "D"),
            Direction::Left => write!(f, "L"),
            Direction::Right => write!(f, "R"),
        }
    }
}

impl FromStr for Direction {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "U" => Ok(Direction::Up),
            "D" => Ok(Direction::Down),
            "L" => Ok(Direction::Left),
            "R" => Ok(Direction::Right),
            _ => Err(eyre!("Invalid direction: {}", s)),
        }
    }
}

#[derive(Clone, Debug)]
struct Motion {
    direction: Direction,
    distance: usize,
}

impl Motion {
    fn unit_motions(&self) -> impl Iterator<Item = Direction> + '_ {
        std::iter::from_fn(move || Some(self.direction)).take(self.distance)
    }
}

impl Display for Motion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.direction, self.distance)
    }
}

impl FromStr for Motion {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (direction, distance) = s
            .split_whitespace()
            .collect_tuple()
            .ok_or_else(|| eyre!("Invalid motion: {}", s))?;
        let direction = direction.parse()?;
        let distance = distance.parse()?;
        Ok(Motion {
            direction,
            distance,
        })
    }
}

struct Motions(Vec<Motion>);

impl FromStr for Motions {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let motions = s.lines().map(str::parse).collect::<Result<Vec<_>, _>>()?;
        Ok(Motions(motions))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn do_move(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.y += 1,
            Direction::Down => self.y -= 1,
            Direction::Left => self.x -= 1,
            Direction::Right => self.x += 1,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Debug)]
struct Part {
    name: char,
    position: Position,
    visited: Option<HashSet<Position>>,
}

impl Part {
    fn new(name: char, position: Position, save_visited: bool) -> Self {
        let visited = if save_visited {
            let mut visited = HashSet::new();
            visited.insert(position);
            Some(visited)
        } else {
            None
        };

        Self {
            name,
            position,
            visited,
        }
    }

    fn insert_visited(&mut self, position: Position) {
        if let Some(visited) = &mut self.visited {
            visited.insert(position);
        }
    }

    #[cfg(test)]
    pub fn has_visited(&self, position: Position) -> bool {
        self.visited
            .as_ref()
            .map_or(false, |visited| visited.contains(&position))
    }

    fn do_move(&mut self, direction: Direction) {
        self.position.do_move(direction);
        self.insert_visited(self.position);
    }

    fn follow(&mut self, other: Position) {
        // We know that the head moved only one step
        let mut dx = other.x - self.position.x;
        let mut dy = other.y - self.position.y;

        while dx.abs() > 1 || dy.abs() > 1 {
            if dx.abs() > 1 && dy == 0 {
                self.position.x += dx.signum();
                dx -= dx.signum();
            } else if dy.abs() > 1 && dx == 0 {
                self.position.y += dy.signum();
                dy -= dy.signum();
            } else {
                self.position.x += dx.signum();
                self.position.y += dy.signum();
                dx -= dx.signum();
                dy -= dy.signum();
            }
            self.insert_visited(self.position);
        }
    }
}

#[derive(Clone, Debug)]
struct BoardState {
    head: Part,
    tails: Vec<Part>,
}

impl BoardState {
    const TAIL_NAMES: &'static str = "193456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    pub fn new(tail_count: usize) -> Self {
        let mut tails = Vec::new();
        for i in 0..tail_count {
            let name = BoardState::TAIL_NAMES.chars().nth(i).unwrap();
            let tail = Part::new(name, Position::new(0, 0), i == tail_count - 1);
            tails.push(tail);
        }

        Self {
            head: Part::new('H', Position::new(0, 0), false),
            tails,
        }
    }

    fn adjust_tail_after_one_step(&mut self) {
        let mut current = &self.head;
        for tail in &mut self.tails {
            tail.follow(current.position);
            current = tail;
        }
    }

    pub fn do_move(&mut self, motion: &Motion) {
        for direction in motion.unit_motions() {
            self.head.do_move(direction);
            self.adjust_tail_after_one_step();
        }
    }

    pub fn do_moves(&mut self, motions: &Motions) {
        for motion in &motions.0 {
            self.do_move(motion);
        }
    }

    pub fn visited_positions(&self) -> usize {
        let last = self.tails.last().unwrap();
        last.visited.as_ref().unwrap().len()
    }

    #[allow(dead_code)]
    pub fn paint(&self) {
        let mut positions = self
            .tails
            .iter()
            .flat_map(|t| t.visited.iter().flat_map(std::collections::HashSet::iter))
            .copied()
            .collect::<Vec<_>>();
        positions.push(self.head.position);

        let min_x = positions.iter().map(|p| p.x).min().unwrap();
        let max_x = positions.iter().map(|p| p.x).max().unwrap();
        let min_y = positions.iter().map(|p| p.y).min().unwrap();
        let max_y = positions.iter().map(|p| p.y).max().unwrap();

        let mut vec_2d = Vec2D::new(
            (max_x - min_x + 1).try_into().unwrap(),
            (max_y - min_y + 1).try_into().unwrap(),
            '.',
        );

        for p in positions {
            vec_2d.set(
                (p.x - min_x).try_into().unwrap(),
                (p.y - min_y).try_into().unwrap(),
                '#',
            );
        }

        vec_2d.set(
            (self.head.position.x - min_x).try_into().unwrap(),
            (self.head.position.y - min_y).try_into().unwrap(),
            'H',
        );
        for tail in &self.tails {
            vec_2d.set(
                (tail.position.x - min_x).try_into().unwrap(),
                (tail.position.y - min_y).try_into().unwrap(),
                tail.name,
            );
        }

        vec_2d.paint_color_map(
            |c| match *c {
                '.' => 0,
                '#' => 1,
                _ => 2,
            },
            ToString::to_string,
        );
    }
}

pub fn day9() -> eyre::Result<()> {
    let motions: Motions = include_str!("../data/day9.txt").parse()?;
    {
        let mut s = BoardState::new(1);
        let start = Instant::now();
        s.do_moves(&motions);
        // s.paint();
        println!("Day 9.1: {} ({:?})", s.visited_positions(), start.elapsed());
    }
    {
        let mut s = BoardState::new(9);
        let start = Instant::now();
        s.do_moves(&motions);
        // s.paint();
        println!("Day 9.2: {} ({:?})", s.visited_positions(), start.elapsed());
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    static TEST_VECTOR: &str = r#"R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2"#;

    #[test]
    fn adjust_tail_h() {
        let mut s = BoardState::new(1);
        s.head.position = Position { x: 1, y: 0 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tails.last().unwrap().position, Position { x: 0, y: 0 });

        s.head.position = Position { x: 2, y: 0 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tails.last().unwrap().position, Position { x: 1, y: 0 });

        s.head.position = Position { x: -5, y: 0 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tails.last().unwrap().position, Position { x: -4, y: 0 });

        assert_eq!(s.tails.last().unwrap().visited.as_ref().unwrap().len(), 6);
        assert!(s.tails.last().unwrap().has_visited(Position { x: 0, y: 0 }));
        assert!(s.tails.last().unwrap().has_visited(Position { x: 1, y: 0 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: -1, y: 0 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: -2, y: 0 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: -3, y: 0 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: -4, y: 0 }));
    }

    #[test]
    fn adjust_tail_v() {
        let mut s = BoardState::new(1);
        s.head.position = Position { x: 0, y: 1 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tails.last().unwrap().position, Position { x: 0, y: 0 });

        s.head.position = Position { x: 0, y: 2 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tails.last().unwrap().position, Position { x: 0, y: 1 });

        s.head.position = Position { x: 0, y: -5 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tails.last().unwrap().position, Position { x: 0, y: -4 });

        assert_eq!(s.tails.last().unwrap().visited.as_ref().unwrap().len(), 6);
        assert!(s.tails.last().unwrap().has_visited(Position { x: 0, y: 0 }));
        assert!(s.tails.last().unwrap().has_visited(Position { x: 0, y: 1 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: 0, y: -1 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: 0, y: -2 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: 0, y: -3 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: 0, y: -4 }));
    }

    #[test]
    fn adjust_tail_d() {
        let mut s = BoardState::new(1);
        s.head.position = Position { x: 1, y: 1 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tails.last().unwrap().position, Position { x: 0, y: 0 });

        s.head.position = Position { x: 2, y: 2 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tails.last().unwrap().position, Position { x: 1, y: 1 });

        s.head.position = Position { x: -5, y: -5 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tails.last().unwrap().position, Position { x: -4, y: -4 });

        assert_eq!(s.tails.last().unwrap().visited.as_ref().unwrap().len(), 6);
        assert!(s.tails.last().unwrap().has_visited(Position { x: 0, y: 0 }));
        assert!(s.tails.last().unwrap().has_visited(Position { x: 1, y: 1 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: -1, y: -1 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: -2, y: -2 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: -3, y: -3 }));
        assert!(s
            .tails
            .last()
            .unwrap()
            .has_visited(Position { x: -4, y: -4 }));
    }

    #[test]
    fn adjust_tail_d2() {
        let mut s = BoardState::new(1);
        s.do_move(&Motion {
            direction: Direction::Right,
            distance: 1,
        });
        s.do_move(&Motion {
            direction: Direction::Up,
            distance: 1,
        });
        s.do_move(&Motion {
            direction: Direction::Right,
            distance: 1,
        });
        s.do_move(&Motion {
            direction: Direction::Up,
            distance: 1,
        });

        s.do_move(&Motion {
            direction: Direction::Up,
            distance: 1,
        });
        assert_eq!(s.tails.last().unwrap().position, Position { x: 2, y: 2 });
    }

    #[test]
    fn adjust_tail_d3() {
        let mut s = BoardState::new(1);
        s.head.position = Position { x: 2, y: 2 };
        s.tails.last_mut().unwrap().position = Position { x: 1, y: 1 };
        s.do_move(&Motion {
            direction: Direction::Right,
            distance: 1,
        });
        s.adjust_tail_after_one_step();
        assert_eq!(s.tails.last().unwrap().position, Position { x: 2, y: 2 });
    }

    #[test]
    fn part_1() {
        let motions: Motions = TEST_VECTOR.parse().unwrap();
        let mut s = BoardState::new(1);
        s.do_moves(&motions);

        assert_eq!(s.visited_positions(), 13);
    }

    #[test]
    fn part_2_1() {
        let motions: Motions = TEST_VECTOR.parse().unwrap();
        let mut s = BoardState::new(10);
        s.do_moves(&motions);

        assert_eq!(s.visited_positions(), 1);
    }

    static TEST_VECTOR_BIG: &str = r#"R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20"#;

    #[test]
    fn part_2_2() {
        let motions: Motions = TEST_VECTOR_BIG.parse().unwrap();
        let mut s = BoardState::new(9);
        s.do_moves(&motions);

        assert_eq!(s.visited_positions(), 36);
    }
}
