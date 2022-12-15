use nom::character::complete;
use nom::{bytes::complete::tag, combinator::map, sequence::tuple, IResult};
use std::fmt::Debug;
use std::{
    fmt::Formatter,
    ops::{Add, Sub},
};

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn parser<'a>(separator: &'a str) -> impl Fn(&str) -> IResult<&str, Self> + 'a {
        move |input| {
            let mut parser = map(
                tuple((complete::i32, tag(separator), complete::i32)),
                |(x, _, y)| Self::new(x, y),
            );
            parser(input)
        }
    }

    pub fn parse(input: &str) -> IResult<&str, Self> {
        Self::parser(",")(input)
    }

    pub fn abs(&self) -> Point {
        Point::new(self.x.abs(), self.y.abs())
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
