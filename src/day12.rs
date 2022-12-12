use crate::utils::Vec2D;
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::time::Instant;
use std::{fmt::Formatter, str::FromStr};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Point {
    row: usize,
    col: usize,
}

impl Point {
    fn new(row: usize, col: usize) -> Self {
        Point { row, col }
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct HeightMap {
    map: Vec2D<i32>,
    start: Point,
    end: Point,
}

fn parse_elevation(c: char) -> eyre::Result<i32> {
    match c {
        c @ 'a'..='z' => Ok(c as i32 - 'a' as i32 + 1),
        _ => Err(eyre::eyre!("Invalid elevation: {}", c)),
    }
}

impl FromStr for HeightMap {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s.lines().count();
        let cols = match s.lines().next() {
            Some(line) => line.chars().count(),
            None => 0,
        };
        let mut map = Vec2D::new(rows, cols, 0);
        let mut start = None;
        let mut end = None;

        for (row, line) in s.lines().enumerate() {
            for (col, c) in line.chars().enumerate() {
                match c {
                    'S' => {
                        start = Some(Point { row, col });
                        map.set(row, col, parse_elevation('a').expect("Hardcoded elevation"));
                    }
                    'E' => {
                        end = Some(Point { row, col });
                        map.set(row, col, parse_elevation('z').expect("Hardcoded elevation"));
                    }
                    c => {
                        map.set(row, col, parse_elevation(c)?);
                    }
                }
            }
        }

        Ok(HeightMap {
            map,
            start: start.ok_or_else(|| eyre::eyre!("No start point found"))?,
            end: end.ok_or_else(|| eyre::eyre!("No end point found"))?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SearchState {
    current: Point,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SearchResult {
    distance: i32,
    path: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GlobalSearchState {
    cache: Vec2D<Option<Option<SearchResult>>>,
}

impl HeightMap {
    fn neighbors(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        let mut neighbors = Vec::new();

        if point.row > 0 {
            neighbors.push(Point::new(point.row - 1, point.col));
        }
        if point.row < self.map.rows - 1 {
            neighbors.push(Point::new(point.row + 1, point.col));
        }
        if point.col > 0 {
            neighbors.push(Point::new(point.row, point.col - 1));
        }
        if point.col < self.map.cols - 1 {
            neighbors.push(Point::new(point.row, point.col + 1));
        }

        neighbors.into_iter()
    }

    fn movable_neighbors(&self, point: Point) -> Vec<Point> {
        let height_at_point = self.map.get(point.row, point.col).unwrap();

        self.neighbors(point)
            .filter(|p| {
                let height_at_neighbor = self.map.get(p.row, p.col).unwrap();
                *height_at_neighbor <= *height_at_point + 1
            })
            .collect()
    }

    fn movable_neighbors_rev(&self, point: Point) -> Vec<Point> {
        let height_at_point = self.map.get(point.row, point.col).unwrap();

        self.neighbors(point)
            .filter(|p| {
                let height_at_neighbor = self.map.get(p.row, p.col).unwrap();
                *height_at_point <= *height_at_neighbor + 1
            })
            .collect()
    }

    // https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm
    fn shortest_path_dijkstra<FGetNeighbors>(
        &self,
        start: &Point,
        end: Option<&Point>,
        get_neighbors: FGetNeighbors,
    ) -> (Option<i32>, Vec2D<Option<i32>>)
    where
        FGetNeighbors: Fn(Point) -> Vec<Point>,
    {
        // Mark all nodes unvisited. Create a set of all the unvisited nodes called the unvisited set.
        let mut unvisited = BTreeSet::<Point>::new();
        for row in 0..self.map.rows {
            for col in 0..self.map.cols {
                unvisited.insert(Point::new(row, col));
            }
        }

        // Assign to every node a tentative distance value: set it to zero for our initial node and to infinity
        // for all other nodes.
        let mut tentative_distances = Vec2D::<Option<i32>>::new(self.map.rows, self.map.cols, None);
        tentative_distances.set(start.row, start.col, Some(0));

        // Set the initial node as current
        let mut current = *start;

        loop {
            let tentative_distance = tentative_distances
                .get(current.row, current.col)
                .unwrap()
                .unwrap();

            // For the current node, consider all of its unvisited neighbors and calculate their tentative distances
            // through the current node.
            for neighbor in get_neighbors(current)
                .iter()
                .filter(|p| unvisited.contains(p))
            {
                let new_tentative_distance = tentative_distance + 1;
                let current_tentative_distance =
                    tentative_distances.get(neighbor.row, neighbor.col).unwrap();

                // Compare the newly calculated tentative distance to the one currently assigned to the neighbor and
                // assign it the smaller one.
                if current_tentative_distance.is_none()
                    || new_tentative_distance < current_tentative_distance.unwrap()
                {
                    tentative_distances.set(
                        neighbor.row,
                        neighbor.col,
                        Some(new_tentative_distance),
                    );
                }
            }

            // When we are done considering all of the unvisited neighbors of the current node, mark the current node
            // as visited and remove it from the unvisited set
            unvisited.remove(&current);

            // If the destination node has been marked visited
            if let Some(end) = end && current == *end {
                // We are done
                return (*tentative_distances.get(end.row, end.col).unwrap(), tentative_distances);
            }

            let next = unvisited
                .iter()
                .filter_map(|p| {
                    tentative_distances
                        .get(p.row, p.col)
                        .unwrap()
                        .map(|d| (d, p))
                })
                .min_by_key(|(d, _)| *d)
                .map(|(_, p)| p);

            match next {
                // Otherwise, select the unvisited node that is marked with the smallest tentative distance, set it as
                // the new current node
                Some(next) => current = *next,
                // if the smallest tentative distance among the nodes in the unvisited set is infinity (when planning
                // a complete traversal; occurs when there is no connection between the initial node and remaining
                // unvisited nodes)
                None => return (None, tentative_distances),
            }
        }
    }

    fn shortest_path_from_start(&self) -> Option<i32> {
        let (dist, _) = self
            .shortest_path_dijkstra(&self.start, Some(&self.end), |p| self.movable_neighbors(p));
        dist
    }

    fn sea_level_points(&self) -> Vec<Point> {
        let mut seal_level_points = Vec::new();

        for row in 0..self.map.rows {
            for col in 0..self.map.cols {
                if *self.map.get(row, col).unwrap() == 1 {
                    seal_level_points.push(Point::new(row, col));
                }
            }
        }

        seal_level_points
    }

    fn shortest_path_from_sea_rayon(&self) -> Option<i32> {
        self.sea_level_points()
            .par_iter()
            .filter_map(|p| {
                let (dist, _) =
                    self.shortest_path_dijkstra(p, Some(&self.end), |p| self.movable_neighbors(p));
                dist
            })
            .min()
    }

    fn shortest_path_from_sea_smart(&self) -> Option<i32> {
        let (_, shortest_from_end) =
            self.shortest_path_dijkstra(&self.end, None, |p| self.movable_neighbors_rev(p));

        self.sea_level_points()
            .par_iter()
            .filter_map(|p| {
                let dist = shortest_from_end.get(p.row, p.col).unwrap();
                *dist
            })
            .min()
    }
}

pub fn day12() -> eyre::Result<()> {
    let height_map: HeightMap = include_str!("../data/day12.txt").parse()?;

    height_map.map.paint_color();

    {
        let start = Instant::now();
        let shortest_path = height_map
            .clone()
            .shortest_path_from_start()
            .ok_or_else(|| eyre::eyre!("No path found"))?;

        let elapsed = start.elapsed();
        let result = shortest_path;
        println!("Day 12.1: {result} ({elapsed:?})");
    }
    {
        let start = Instant::now();
        let result = height_map
            .clone()
            .shortest_path_from_sea_smart()
            .ok_or_else(|| eyre::eyre!("No path found"))?;

        let elapsed = start.elapsed();
        println!("Day 12.2: {result} ({elapsed:?})");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    static TEST_VECTOR: &str = r#"Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi"#;

    #[test]
    fn parse() -> eyre::Result<()> {
        let height_map = TEST_VECTOR.parse::<HeightMap>()?;
        assert_eq!(height_map.start, Point { row: 0, col: 0 });
        assert_eq!(height_map.end, Point { row: 2, col: 5 });
        assert_eq!(height_map.map.get(0, 0), Some(&1));
        assert_eq!(height_map.map.get(0, 1), Some(&1));
        assert_eq!(height_map.map.get(0, 2), Some(&2));
        Ok(())
    }

    #[test]
    fn part1() -> eyre::Result<()> {
        let height_map = TEST_VECTOR.parse::<HeightMap>()?;
        let shortest_path = height_map.shortest_path_from_start().unwrap();

        assert_eq!(shortest_path, 31);
        Ok(())
    }

    #[test]
    fn part2() -> eyre::Result<()> {
        let height_map = TEST_VECTOR.parse::<HeightMap>()?;
        let shortest_path = height_map.shortest_path_from_sea_rayon().unwrap();

        assert_eq!(shortest_path, 29);
        Ok(())
    }

    #[test]
    fn sea_level_points() {
        let height_map = TEST_VECTOR.parse::<HeightMap>().unwrap();
        let sea_level = height_map.sea_level_points();
        assert_eq!(sea_level.len(), 6);
        assert_eq!(
            sea_level,
            vec![
                Point::new(0, 0),
                Point::new(0, 1),
                Point::new(1, 0),
                Point::new(2, 0),
                Point::new(3, 0),
                Point::new(4, 0)
            ]
        );
    }

    #[test]
    fn neighbors() {
        let height_map = TEST_VECTOR.parse::<HeightMap>().unwrap();
        {
            let neighbors = height_map.neighbors(Point::new(0, 0)).collect_vec();
            assert_eq!(neighbors, vec![Point::new(1, 0), Point::new(0, 1)]);
        }
        {
            let neighbors = height_map.neighbors(Point::new(1, 1)).collect_vec();
            assert_eq!(
                neighbors,
                vec![
                    Point::new(0, 1),
                    Point::new(2, 1),
                    Point::new(1, 0),
                    Point::new(1, 2),
                ]
            );
        }
    }

    #[test]
    fn movable_neighbors() {
        let height_map = r#"Sabqponm
abdryxxl
accszExk
acctuvwj
abdefghi"#
            .parse::<HeightMap>()
            .unwrap();
        {
            let neighbors = height_map.movable_neighbors(Point::new(0, 0));
            assert_eq!(neighbors, vec![Point::new(1, 0), Point::new(0, 1)]);
        }
        {
            let neighbors = height_map.movable_neighbors(Point::new(1, 1));
            assert_eq!(
                neighbors,
                vec![Point::new(0, 1), Point::new(2, 1), Point::new(1, 0),]
            );
        }
    }

    #[test]
    fn movable_neighbors_2() {
        let height_map = r#"Sabqponm
yxzryxxl
accszExk
acctuvwj
abdefghi"#
            .parse::<HeightMap>()
            .unwrap();
        {
            let neighbors = height_map.movable_neighbors(Point::new(1, 1));
            assert_eq!(
                neighbors,
                vec![Point::new(0, 1), Point::new(2, 1), Point::new(1, 0),]
            );
        }
    }
}
