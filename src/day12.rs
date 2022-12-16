use crate::utils::{a_start, dijkstra, DayParams, Vec2D};
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    hash::Hash,
    str::FromStr,
    time::Instant,
};

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

    fn shortest_path_dijkstra<FGetNeighbors>(
        &self,
        start: Point,
        end: Option<Point>,
        get_neighbors: FGetNeighbors,
    ) -> (Option<i32>, HashMap<Point, i32>)
    where
        FGetNeighbors: Fn(&Point) -> Vec<Point>,
    {
        let result = dijkstra(start, end, get_neighbors, |_a, _b| 1, vec![]);

        (result.distance_to_end, result.distances)
    }

    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn shortest_path_a_star(&self, start: Point, end: Point) -> Option<i32> {
        let end_col = end.col as f64;
        let end_row = end.row as f64;
        let path = a_start(
            start,
            end,
            |p| ((p.col as f64 - end_col).powi(2) + (p.row as f64 - end_row).powi(2)).sqrt() as i32,
            |p| self.movable_neighbors(*p),
            |_a, _b| 1,
        );

        path.map(|p| p.len() as i32)
    }

    #[allow(dead_code)]
    fn shortest_path_from_start_dijkstra(&self) -> Option<i32> {
        // Use the reverse function as it's faster to run
        let (_, shortest_from_end) =
            self.shortest_path_dijkstra(self.end, None, |p| self.movable_neighbors_rev(*p));

        shortest_from_end.get(&self.start).copied()
    }

    fn shortest_path_from_start_a_star(&self) -> Option<i32> {
        self.shortest_path_a_star(self.start, self.end)
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

    #[allow(dead_code)]
    fn shortest_path_from_sea_rayon(&self) -> Option<i32> {
        self.sea_level_points()
            .par_iter()
            .filter_map(|p| {
                let (dist, _) =
                    self.shortest_path_dijkstra(*p, Some(self.end), |p| self.movable_neighbors(*p));
                dist
            })
            .min()
    }

    #[allow(dead_code)]
    fn shortest_path_from_sea_smart(&self) -> Option<i32> {
        let (_, shortest_from_end) =
            self.shortest_path_dijkstra(self.end, None, |p| self.movable_neighbors_rev(*p));

        self.sea_level_points()
            .par_iter()
            .filter_map(|p| {
                let dist = shortest_from_end.get(p);
                dist.copied()
            })
            .min()
    }

    #[allow(dead_code)]
    fn shortest_path_from_sea_a_star_rayon(&self) -> Option<i32> {
        self.sea_level_points()
            .par_iter()
            .filter_map(|p| self.shortest_path_a_star(*p, self.end))
            .min()
    }
}

pub fn day12(p: &DayParams) -> eyre::Result<()> {
    let height_map: HeightMap = p.read_input()?.parse()?;

    if p.debug {
        height_map.map.paint_color();
    }

    p.part_1_raw(|| {
        let start = Instant::now();
        let shortest_path = height_map
            .shortest_path_from_start_a_star()
            .ok_or_else(|| eyre::eyre!("No path found"))?;

        let elapsed = start.elapsed();
        let result = shortest_path;
        println!("Day 12.1 [A*]: {result} ({elapsed:?})");

        if p.debug {
            let start = Instant::now();
            let shortest_path = height_map
                .shortest_path_from_start_dijkstra()
                .ok_or_else(|| eyre::eyre!("No path found"))?;

            let elapsed = start.elapsed();
            let result = shortest_path;
            println!("Day 12.1 [Dijkstra]: {result} ({elapsed:?})");
        }

        Ok(())
    })?;

    p.part_2_raw(|| {
        let start = Instant::now();
        let result = height_map
            .shortest_path_from_sea_a_star_rayon()
            .ok_or_else(|| eyre::eyre!("No path found"))?;

        let elapsed = start.elapsed();
        println!("Day 12.2 [A*]: {result} ({elapsed:?})");

        if p.debug {
            let start = Instant::now();
            let result = height_map
                .shortest_path_from_sea_smart()
                .ok_or_else(|| eyre::eyre!("No path found"))?;

            let elapsed = start.elapsed();
            println!("Day 12.2 [Dijkstra]: {result} ({elapsed:?})");
        }

        Ok(())
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use pretty_assertions::assert_eq;

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
        let shortest_path = height_map.shortest_path_from_start_dijkstra().unwrap();

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
