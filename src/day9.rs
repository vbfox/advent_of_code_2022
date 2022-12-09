use std::{collections::HashSet, fmt::Display, str::FromStr};

use eyre::eyre;
use itertools::Itertools;

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
            .ok_or_else(|| eyre!("Invalid motion: {}", s))?; // TODO: use split_once when stable
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
        let motions = s
            .lines()
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Motions(motions))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    fn do_move(&self, direction: Direction) -> Self {
        match direction {
            Direction::Up => Self {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Down => Self {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Left => Self {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Self {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Debug)]
struct BoardState {
    visited_by_tail: HashSet<Position>,
    head: Position,
    tail: Position,
}

impl BoardState {
    pub fn new() -> Self {
        let tail = Position { x: 0, y: 0 };
        let mut visited_by_tail = HashSet::new();
        visited_by_tail.insert(tail.clone());

        Self {
            visited_by_tail,
            head: Position { x: 0, y: 0 },
            tail,
        }
    }

    fn adjust_tail_after_one_step(&mut self) {
        // We know that the head moved only one step
        let mut dx = self.head.x - self.tail.x;
        let mut dy = self.head.y - self.tail.y;

        while dx.abs() > 1 || dy.abs() > 1 {
            if dx.abs() > 1 && dy == 0 {
                self.tail.x += dx.signum();
                dx -= dx.signum();
            } else if dy.abs() > 1 && dx == 0 {
                self.tail.y += dy.signum();
                dy -= dy.signum();
            } else {
                self.tail.x += dx.signum();
                self.tail.y += dy.signum();
                dx -= dx.signum();
                dy -= dy.signum();
            }
            self.visited_by_tail.insert(self.tail.clone());
        }
    }

    pub fn do_move(&mut self, motion: &Motion) {
        // println!("------");
        // println!("Before {}: {:#?}", motion, self);
        for direction in motion.unit_motions() {
            self.head = self.head.do_move(direction);
            // println!("Moved head to {}", self.head);
            self.adjust_tail_after_one_step();
            // println!("Moved tail to {}", self.tail);
            self.visited_by_tail.insert(self.tail.clone());
        }
        // println!("After: {:#?}", self);
    }

    pub fn do_moves(&mut self, motions: &Motions) {
        for motion in &motions.0 {
            self.do_move(motion);
        }
    }

    pub fn visited_positions(&self) -> usize {
        self.visited_by_tail.len()
    }
}

pub fn day9() -> eyre::Result<()> {
    let motions: Motions = include_str!("../data/day9.txt").parse()?;
    {
        let mut s = BoardState::new();
        s.do_moves(&motions);
        println!("Day 9.1: {}", s.visited_positions());
    }
    // {
    //     let distance = ViewingDistance::compute(&forest)?;
    //     println!(
    //         "Day 8.2: {}",
    //         distance.get_max().ok_or_else(|| eyre!("No max ?"))?
    //     );
    // }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use super::*;

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
        let mut s = BoardState::new();
        s.head = Position { x: 1, y: 0 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tail, Position { x: 0, y: 0 });

        s.head = Position { x: 2, y: 0 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tail, Position { x: 1, y: 0 });

        s.head = Position { x: -5, y: 0 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tail, Position { x: -4, y: 0 });

        assert_eq!(s.visited_by_tail.len(), 6);
        assert!(s.visited_by_tail.contains(&Position { x: 0, y: 0 }));
        assert!(s.visited_by_tail.contains(&Position { x: 1, y: 0 }));
        assert!(s.visited_by_tail.contains(&Position { x: -1, y: 0 }));
        assert!(s.visited_by_tail.contains(&Position { x: -2, y: 0 }));
        assert!(s.visited_by_tail.contains(&Position { x: -3, y: 0 }));
        assert!(s.visited_by_tail.contains(&Position { x: -4, y: 0 }));
    }

    #[test]
    fn adjust_tail_v() {
        let mut s = BoardState::new();
        s.head = Position { x: 0, y: 1 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tail, Position { x: 0, y: 0 });

        s.head = Position { x: 0, y: 2 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tail, Position { x: 0, y: 1 });

        s.head = Position { x: 0, y: -5 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tail, Position { x: 0, y: -4 });

        assert_eq!(s.visited_by_tail.len(), 6);
        assert!(s.visited_by_tail.contains(&Position { x: 0, y: 0 }));
        assert!(s.visited_by_tail.contains(&Position { x: 0, y: 1 }));
        assert!(s.visited_by_tail.contains(&Position { x: 0, y: -1 }));
        assert!(s.visited_by_tail.contains(&Position { x: 0, y: -2 }));
        assert!(s.visited_by_tail.contains(&Position { x: 0, y: -3 }));
        assert!(s.visited_by_tail.contains(&Position { x: 0, y: -4 }));
    }

    #[test]
    fn adjust_tail_d() {
        let mut s = BoardState::new();
        s.head = Position { x: 1, y: 1 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tail, Position { x: 0, y: 0 });

        s.head = Position { x: 2, y: 2 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tail, Position { x: 1, y: 1 });

        s.head = Position { x: -5, y: -5 };
        s.adjust_tail_after_one_step();
        assert_eq!(s.tail, Position { x: -4, y: -4 });

        assert_eq!(s.visited_by_tail.len(), 6);
        assert!(s.visited_by_tail.contains(&Position { x: 0, y: 0 }));
        assert!(s.visited_by_tail.contains(&Position { x: 1, y: 1 }));
        assert!(s.visited_by_tail.contains(&Position { x: -1, y: -1 }));
        assert!(s.visited_by_tail.contains(&Position { x: -2, y: -2 }));
        assert!(s.visited_by_tail.contains(&Position { x: -3, y: -3 }));
        assert!(s.visited_by_tail.contains(&Position { x: -4, y: -4 }));
    }

    #[test]
    fn adjust_tail_d2() {
        let mut s = BoardState::new();
        s.head = Position { x: 2, y: 2 };
        s.tail = Position { x: 1, y: 1 };
        s.do_move(&Motion {
            direction: Direction::Up,
            distance: 1,
        });
        s.adjust_tail_after_one_step();
        assert_eq!(s.tail, Position { x: 2, y: 2 });
    }

    #[test]
    fn adjust_tail_d3() {
        let mut s = BoardState::new();
        s.head = Position { x: 2, y: 2 };
        s.tail = Position { x: 1, y: 1 };
        s.do_move(&Motion {
            direction: Direction::Right,
            distance: 1,
        });
        s.adjust_tail_after_one_step();
        assert_eq!(s.tail, Position { x: 2, y: 2 });
    }

    #[test]
    fn part_1() {
        let motions: Motions = TEST_VECTOR.parse().unwrap();
        let mut s = BoardState::new();
        s.do_moves(&motions);

        assert_eq!(s.visited_positions(), 13);
    }
}
