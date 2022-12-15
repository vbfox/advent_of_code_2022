use super::parse_i32;
use nom::{bytes::complete::tag, combinator::map, sequence::tuple, IResult};
use std::fmt::Debug;
use std::{
    fmt::Formatter,
    ops::{Add, Sub},
};

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn parse<'a>(input: &'a str, separator: &str) -> IResult<&'a str, Self> {
        let mut parser = map(
            tuple((parse_i32, tag(separator), parse_i32)),
            |(x, _, y)| Self::new(x, y),
        );
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
