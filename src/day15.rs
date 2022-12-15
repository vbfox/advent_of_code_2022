use crate::utils::{nom_finish, DayParams, Point, Vec2D};
use eyre::eyre;
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::map,
    multi::{many0, separated_list0},
    sequence::{preceded, terminated, tuple},
    IResult,
};
use range_ranger::ContinuousRange;
use rayon::prelude::*;
use std::{collections::HashSet, ops::Bound};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Box {
    top_left: Point,
    bottom_right: Point,
}

impl Box {
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.top_left.x
            && point.x <= self.bottom_right.x
            && point.y >= self.top_left.y
            && point.y <= self.bottom_right.y
    }
}

impl std::ops::Add for Box {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let min_x = self.top_left.x.min(other.top_left.x);
        let max_x = self.bottom_right.x.max(other.bottom_right.x);
        let min_y = self.top_left.y.min(other.top_left.y);
        let max_y = self.bottom_right.y.max(other.bottom_right.y);

        Self {
            top_left: Point::new(min_x, min_y),
            bottom_right: Point::new(max_x, max_y),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sensor {
    position: Point,
    closest_beacon: Point,
    hit_box: Box,
    beacon_distance: i32,
}

impl Sensor {
    pub fn new(position: Point, closest_beacon: Point) -> Self {
        Self {
            position,
            closest_beacon,
            hit_box: Self::get_box(position, closest_beacon),
            beacon_distance: Self::manhattan_distance(position, closest_beacon),
        }
    }

    pub fn parse(input: &str) -> IResult<&str, Self> {
        let mut parser = map(
            tuple((
                preceded(tag("Sensor at x="), Point::parser(", y=")),
                preceded(tag(": closest beacon is at x="), Point::parser(", y=")),
            )),
            |(position, closest_beacon)| Self::new(position, closest_beacon),
        );

        parser(input)
    }

    fn manhattan_distance(a: Point, b: Point) -> i32 {
        let p = (a - b).abs();
        p.x + p.y
    }

    fn get_box(position: Point, closest_beacon: Point) -> Box {
        let beacon_distance = Self::manhattan_distance(position, closest_beacon);
        let min_x = position.x - beacon_distance;
        let max_x = position.x + beacon_distance;
        let min_y = position.y - beacon_distance;
        let max_y = position.y + beacon_distance;

        Box {
            top_left: Point::new(min_x, min_y),
            bottom_right: Point::new(max_x, max_y),
        }
    }

    pub fn in_zone(&self, point: Point) -> bool {
        if !self.hit_box.contains(point) {
            return false;
        }

        let distance = Self::manhattan_distance(self.position, point);

        distance <= self.beacon_distance
    }

    pub fn zone_points(&self) -> impl Iterator<Item = Point> + '_ {
        let beacon_distance = Self::manhattan_distance(self.position, self.closest_beacon);
        let min_x = self.position.x - beacon_distance;
        let max_x = self.position.x + beacon_distance;
        let min_y = self.position.y - beacon_distance;
        let max_y = self.position.y + beacon_distance;

        (min_x..=max_x)
            .cartesian_product(min_y..=max_y)
            .map(|(x, y)| Point::new(x, y))
            .filter(move |point| self.in_zone(*point))
    }

    pub fn range_on_y(&self, y: i32) -> ContinuousRange<i32> {
        if y < self.hit_box.top_left.y || y > self.hit_box.bottom_right.y {
            return ContinuousRange::empty();
        }

        let y_dist = (self.position.y - y).abs();

        let min = self.position.x - self.beacon_distance + y_dist;
        let max = self.position.x + self.beacon_distance - y_dist;

        ContinuousRange::inclusive(min, max)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sensors {
    sensors: Vec<Sensor>,
}

impl Sensors {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let mut parser = map(
            terminated(
                separated_list0(line_ending, Sensor::parse),
                many0(line_ending),
            ),
            |sensors| Self { sensors },
        );
        parser(input)
    }

    fn paint(&self) {
        let (min_x, max_x) = self
            .sensors
            .iter()
            .flat_map(|s| {
                let mut v = vec![s.position.x, s.closest_beacon.x];
                v.extend(s.zone_points().map(|p| p.x));
                v
            })
            .minmax()
            .into_option()
            .unwrap();
        let (min_y, max_y) = self
            .sensors
            .iter()
            .flat_map(|s| {
                let mut v = vec![s.position.y, s.closest_beacon.y];
                v.extend(s.zone_points().map(|p| p.y));
                v
            })
            .minmax()
            .into_option()
            .unwrap();

        let rows = max_y - min_y + 1;
        let cols = max_x - min_x + 1;
        let delta = Point::new(-min_x, -min_y);

        let mut canvas = Vec2D::new(rows as usize, cols as usize, '.');

        for sensor in &self.sensors {
            for p in sensor.zone_points() {
                let position = p + delta;
                canvas.set(position.y as usize, position.x as usize, '#');
            }
        }

        for sensor in &self.sensors {
            let position = sensor.position + delta;
            canvas.set(position.y as usize, position.x as usize, 'S');

            let closest_beacon = sensor.closest_beacon + delta;
            canvas.set(closest_beacon.y as usize, closest_beacon.x as usize, 'B');
        }

        canvas.paint_color_map(
            |c| match c {
                '#' => 1,
                'B' => 2,
                'S' => 3,
                _ => 0,
            },
            |c| c.to_string(),
        );
    }

    fn hit_box(&self) -> Box {
        self.sensors
            .iter()
            .map(|s| s.hit_box.clone())
            .reduce(|a, b| a + b)
            .unwrap()
    }

    fn occupied(&self) -> impl Iterator<Item = Point> + '_ {
        self.sensors
            .iter()
            .flat_map(|s| vec![s.closest_beacon, s.position])
    }

    fn count_cannot_contain_beacon(&self, y: i32) -> i32 {
        let sensors_box = self.hit_box();
        let occupied = self.occupied().collect::<HashSet<_>>();

        // If the hit box doesn't touch our y coordinate we don't care
        let filtered_sensors = self
            .sensors
            .iter()
            .filter(|s| y >= s.hit_box.top_left.y && y <= s.hit_box.bottom_right.y)
            .collect::<Vec<_>>();

        let x_range = sensors_box.top_left.x..=sensors_box.bottom_right.x;

        //Let's launch rayon and test all X coordinates
        x_range
            .into_par_iter()
            .map(|x| Point::new(x, y))
            .filter(|point| !occupied.contains(point))
            .map(|point| {
                for sensor in &filtered_sensors {
                    if sensor.position == point || sensor.in_zone(point) {
                        return 1;
                    }
                }
                return 0;
            })
            .sum()
    }

    fn range_start(range: &ContinuousRange<i32>) -> i32 {
        match range
            .start()
            .ok_or_else(|| eyre!("No start value of {:?}", range))
            .unwrap()
        {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x,
            Bound::Unbounded => panic!("Unbounded range"),
        }
    }

    fn range_end(range: &ContinuousRange<i32>) -> i32 {
        match range
            .end()
            .ok_or_else(|| eyre!("No end value of {:?}", range))
            .unwrap()
        {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x,
            Bound::Unbounded => panic!("Unbounded range"),
        }
    }

    // TODO: Work on range_ranger, it's what it is made for
    fn simplify_ranges(ranges: Vec<ContinuousRange<i32>>) -> Vec<ContinuousRange<i32>> {
        let mut ranges = ranges;

        ranges.sort_by(|a, b| Self::range_start(&a).cmp(&Self::range_start(&b)).reverse());

        // let mut ranges = VecDeque::from(ranges);
        let mut simplified = vec![];
        let mut current = ranges.pop();
        while let Some(r) = ranges.pop() {
            match current {
                Some(c) => match c.union(&r) {
                    Some(u) => current = Some(u),
                    None => {
                        simplified.push(c);
                        current = Some(r);
                    }
                },
                None => current = Some(r.clone()),
            }
        }

        if let Some(c) = current {
            simplified.push(c);
        }

        simplified
    }

    fn first_cannot_contain_beacon(&self, min: i32, max: i32) -> Option<Point> {
        let test_range: ContinuousRange<_> = (min..=max).into();

        (min..=max).par_bridge().find_map_any(|y| {
            let y_ranges = self
                .sensors
                .iter()
                .map(|s| s.range_on_y(y).intersection(&test_range))
                .filter(|r| !r.is_empty());
            let simplified = Self::simplify_ranges(y_ranges.collect());
            if simplified.len() != 1 {
                let x = Self::range_end(&simplified[0]) + 1;
                let point = Point::new(x, y);
                Some(point)
            } else {
                None
            }
        })
    }

    fn tuning_frequency(&self, min: i32, max: i32) -> Option<i64> {
        let p = self.first_cannot_contain_beacon(min, max)?;

        Some((p.x as i64) * 4000000 + (p.y as i64))
    }
}

pub fn day(p: &DayParams) -> eyre::Result<()> {
    let input = &p.read_input()?;

    let sensors = nom_finish(Sensors::parse, input)?;

    if p.debug {
        sensors.paint();
    }

    p.part_1(|| {
        let y = if p.test { 10 } else { 2_000_000 };

        Ok(sensors.count_cannot_contain_beacon(y))
    })?;

    p.part_2(|| {
        let min = 0;
        let max = if p.test { 20 } else { 4_000_000 };

        let freq = sensors
            .tuning_frequency(min, max)
            .ok_or_else(|| eyre!("No tuning frequency found"))?;
        Ok(freq)
    })?;

    Ok(())
}
