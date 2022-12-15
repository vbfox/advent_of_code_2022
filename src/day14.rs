use crate::utils::{nom_finish, parse_i32, DayParams, Vec2D};
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{char, line_ending},
    combinator::map,
    multi::{many0, separated_list1},
    sequence::{terminated, tuple},
    IResult,
};
use std::iter::Extend;
use std::{
    collections::HashMap,
    fmt::Formatter,
    ops::{Add, Sub},
};
use std::{collections::HashSet, fmt::Debug};

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        let mut parser = map(tuple((parse_i32, char(','), parse_i32)), |(x, _, y)| {
            Self::new(x, y)
        });
        parser(input)
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct PathLine {
    points: Vec<Point>,
}

impl PathLine {
    fn parse(input: &str) -> IResult<&str, Self> {
        let mut parser = map(separated_list1(tag(" -> "), Point::parse), |points| Self {
            points,
        });
        parser(input)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Scan {
    lines: Vec<PathLine>,
}

impl Scan {
    fn parse(input: &str) -> IResult<&str, Self> {
        let mut parser = terminated(
            map(separated_list1(line_ending, PathLine::parse), |lines| {
                Self { lines }
            }),
            many0(line_ending),
        );
        parser(input)
    }

    fn all_points(&self) -> impl Iterator<Item = &Point> {
        self.lines.iter().flat_map(|line| line.points.iter())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum CavePosition {
    Air,
    Rock,
    Sand,
    Source,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Cave {
    floor_y: i32,
    floor_is_rock: bool,
    structure: HashMap<Point, CavePosition>,
    source: Point,
}

impl Cave {
    fn set(&mut self, point: Point, value: CavePosition) {
        debug_assert_ne!(value, CavePosition::Air);
        self.structure.insert(point, value);
    }

    fn get(&self, point: Point) -> Option<CavePosition> {
        if point.y >= self.floor_y {
            if self.floor_is_rock {
                Some(CavePosition::Rock)
            } else {
                None
            }
        } else {
            Some(
                self.structure
                    .get(&point)
                    .copied()
                    .unwrap_or(CavePosition::Air),
            )
        }
    }

    fn draw_path_line(&mut self, line: &PathLine) {
        let mut start = line.points[0];

        for end in line.points.iter().skip(1) {
            let delta = *end - start;
            let offset = match delta {
                Point { x: 0, y } => Point::new(0, y.signum()),
                Point { x, y: 0 } => Point::new(x.signum(), 0),
                _ => panic!("Invalid delta between {start:?} and {end:?}: {delta:?}"),
            };

            let mut current = start;
            loop {
                self.set(current, CavePosition::Rock);

                if current == *end {
                    break;
                }

                current = current + offset;
            }

            start = *end;
        }
    }

    fn draw_scan(&mut self, scan: &Scan) {
        for line in &scan.lines {
            self.draw_path_line(line);
        }
    }

    fn from_scan(scan: &Scan, source: Point, floor_is_rock: bool) -> Self {
        let mut all_points = scan.all_points().dedup().collect_vec();
        all_points.extend_one(&source);

        let max_y = all_points.iter().map(|p| p.y).max().unwrap();

        let max_y = max_y + 2;

        let mut result = Self {
            floor_y: max_y,
            floor_is_rock,
            structure: HashMap::new(),
            source,
        };

        result.draw_scan(scan);
        result.set(source, CavePosition::Source);

        result
    }

    fn emit_sand(&mut self) -> (bool, Vec<Point>) {
        if self.get(self.source) == Some(CavePosition::Sand) {
            // Source is blocked, can't emit anymore
            return (false, Vec::new());
        }

        // The sand is pouring into the cave from point 500,0
        let mut visited = Vec::new();
        let mut current = self.source;
        loop {
            visited.push(current);
            let below = current + Point::new(0, 1);
            let down_left = current + Point::new(-1, 1);
            let down_right = current + Point::new(1, 1);

            match (self.get(below), self.get(down_left), self.get(down_right)) {
                // A unit of sand always falls down one step if possible
                (Some(CavePosition::Air), _, _) => {
                    current = below;
                }

                // If the tile immediately below is blocked (by rock or sand), the unit of sand
                // attempts to instead move diagonally one step down and to the left
                (_, Some(CavePosition::Air), _) => {
                    current = down_left;
                }

                // If that tile is blocked, the unit of sand attempts to instead move diagonally one
                // step down and to the right
                (_, _, Some(CavePosition::Air)) => {
                    current = down_right;
                }

                (None, _, _) | (_, None, _) | (_, _, None) => {
                    return (false, visited);
                }

                _ => {
                    break;
                }
            }
        }

        self.set(current, CavePosition::Sand);
        (true, visited)
    }

    fn emit_sand_util_filled(&mut self) {
        loop {
            let (result, _) = self.emit_sand();
            if !result {
                break;
            }
        }
    }

    fn count_sand(&self) -> usize {
        self.structure
            .iter()
            .filter(|(_, p)| **p == CavePosition::Sand)
            .count()
    }

    #[allow(dead_code)]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    fn paint(&self) {
        let mut cloned = self.clone();
        let (_, points) = cloned.emit_sand();
        let points = points.into_iter().collect::<HashSet<_>>();

        let (min_x, max_x) = cloned
            .structure
            .keys()
            .map(|p| p.x)
            .minmax()
            .into_option()
            .unwrap();

        let rows = if self.floor_is_rock {
            self.floor_y + 1
        } else {
            self.floor_y - 1
        };
        let mut structure = Vec2D::new(
            (rows) as usize,
            (max_x - min_x + 1) as usize,
            CavePosition::Air,
        );

        let delta = Point::new(min_x, 0);
        for (p, v) in cloned.structure {
            let p = p - delta;
            structure.set(p.y as usize, p.x as usize, v);
        }

        if self.floor_is_rock {
            for x in 0..structure.cols {
                structure.set(self.floor_y as usize, x, CavePosition::Rock);
            }
        }

        let structure = structure.map(|p, y, x| match p {
            CavePosition::Rock => '#',
            CavePosition::Sand => {
                let point = Point::new(x as i32, y as i32) + delta;
                if points.contains(&point) {
                    'x'
                } else {
                    'o'
                }
            }
            CavePosition::Source => '+',
            CavePosition::Air => {
                let point = Point::new(x as i32, y as i32) + delta;
                if points.contains(&point) {
                    '~'
                } else {
                    '.'
                }
            }
        });

        structure.paint_color_map(
            |p| match p {
                '#' => 1,
                '+' => 2,
                'o' => 3,
                '~' => 4,
                'x' => 5,
                _ => 0,
            },
            std::string::ToString::to_string,
        );
    }
}

pub fn day14(p: &DayParams) -> eyre::Result<()> {
    let input = &p.read_input()?;

    let scan = nom_finish(Scan::parse, input)?;

    p.part_1(|| {
        let mut cave = Cave::from_scan(&scan, Point::new(500, 0), false);
        cave.emit_sand_util_filled();
        if p.debug {
            cave.paint();
        }
        Ok(cave.count_sand())
    })?;

    p.part_2(|| {
        let mut cave = Cave::from_scan(&scan, Point::new(500, 0), true);
        cave.emit_sand_util_filled();
        if p.debug && p.test {
            cave.paint();
        }
        Ok(cave.count_sand())
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    static TEST_VECTOR: &str = r#"498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9"#;

    #[test]
    fn from_scan() {
        let scan = nom_finish(Scan::parse, TEST_VECTOR).unwrap();
        let cave = Cave::from_scan(&scan, Point::new(500, 0), false);
        assert_eq!(cave.get(Point::new(500, 0)), Some(CavePosition::Source));
        assert_eq!(cave.get(Point::new(494, 9)), Some(CavePosition::Rock));

        assert_eq!(cave.get(Point::new(495, 6)), Some(CavePosition::Air));
        assert_eq!(cave.get(Point::new(496, 6)), Some(CavePosition::Rock));
        assert_eq!(cave.get(Point::new(498, 6)), Some(CavePosition::Rock));
        assert_eq!(cave.get(Point::new(499, 6)), Some(CavePosition::Air));
        assert_eq!(cave.get(Point::new(498, 4)), Some(CavePosition::Rock));
        assert_eq!(cave.get(Point::new(498, 3)), Some(CavePosition::Air));
    }

    #[test]
    fn part1() {
        let scan = nom_finish(Scan::parse, TEST_VECTOR).unwrap();
        let mut cave = Cave::from_scan(&scan, Point::new(500, 0), false);
        cave.emit_sand_util_filled();
        let count = cave.count_sand();
        assert_eq!(count, 24);
    }

    #[test]
    fn part2() {
        let scan = nom_finish(Scan::parse, TEST_VECTOR).unwrap();
        let mut cave = Cave::from_scan(&scan, Point::new(500, 0), true);
        cave.emit_sand_util_filled();
        let count = cave.count_sand();
        assert_eq!(count, 93);
    }
}
