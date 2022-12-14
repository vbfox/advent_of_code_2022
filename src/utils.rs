#![allow(dead_code)]

use eyre::{bail, eyre};
use nom::{
    character::complete::digit1, combinator::map_res, error::ParseError, IResult, InputLength,
    Parser,
};
use std::{
    fmt::{self, Display},
    ops::{Add, Div, Mul, Sub},
};

mod aoc;
mod shortest_path;
mod vec2d;

pub use aoc::{DayParams, DayPart};
pub use shortest_path::{a_start, dijkstra, DijkstraResult};
pub use vec2d::Vec2D;

pub struct CharSliceIterator<'a> {
    s: &'a str,
    index: usize,
}

impl<'a> CharSliceIterator<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, index: 0 }
    }
}

impl<'a> Iterator for CharSliceIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.s.len() {
            let slice = &self.s[self.index..=self.index];
            self.index += 1;
            Some(slice)
        } else {
            None
        }
    }
}

pub trait CharSliceExt {
    fn char_slices(&self) -> CharSliceIterator;
}

impl<T: AsRef<str>> CharSliceExt for T {
    fn char_slices(&self) -> CharSliceIterator {
        CharSliceIterator::new(self.as_ref())
    }
}

// --------------------------------------------------------------------------

fn single<T>(mut iterator: impl Iterator<Item = T>) -> Option<T> {
    match (iterator.next(), iterator.next()) {
        (Some(item), None) => Some(item),
        _ => None,
    }
}

pub trait SingleExt<T> {
    fn single(self) -> Option<T>;
}

impl<Item, T: Iterator<Item = Item>> SingleExt<Item> for T {
    fn single(self) -> Option<Item> {
        single(self)
    }
}

#[cfg(test)]
mod single_tests {
    use super::*;

    #[test]
    fn single_test() {
        assert_eq!(single("a".chars()), Some('a'));
        assert_eq!(single("ab".chars()), None);
        assert_eq!(single("".chars()), None);
    }
}

// --------------------------------------------------------------------------

pub fn find_common_items<'a, T: Eq>(items: &'a [Vec<T>]) -> Vec<&'a T> {
    let mut common_items: Vec<&'a T> = Vec::new();

    for item in &items[0] {
        if common_items.contains(&item) {
            continue;
        }

        if items[1..].iter().all(|i| i.contains(item)) {
            common_items.push(item);
        }
    }

    common_items
}

#[cfg(test)]
mod find_common_items_tests {
    use super::*;

    #[test]
    fn all() {
        let items = vec![
            vec![1, 2, 3, 4, 5],
            vec![1, 2, 3, 4, 5],
            vec![1, 2, 3, 4, 5],
        ];

        let common_items = find_common_items(&items);

        assert_eq!(common_items, vec![&1, &2, &3, &4, &5]);
    }

    #[test]
    fn some() {
        let items = vec![vec![2, 3, 4, 5], vec![1, 2, 3, 4, 5], vec![1, 2, 4, 5]];

        let common_items = find_common_items(&items);

        assert_eq!(common_items, vec![&2, &4, &5]);
    }

    #[test]
    fn none() {
        let items = vec![vec![2, 4, 5], vec![1, 3], vec![5]];

        let common_items = find_common_items(&items);

        assert_eq!(common_items, Vec::<&i32>::default());
    }
}

// --------------------------------------------------------------------------

/// Finishes a nom parser and returns a Result with [eyre] used for errors.
pub fn nom_finish<I, O, E: ParseError<I>, F>(mut f: F, input: I) -> eyre::Result<O>
where
    I: InputLength + fmt::Debug,
    F: Parser<I, O, E>,
    E: Display,
{
    match f.parse(input) {
        Ok((s, result)) => match s.input_len() {
            0 => Ok(result),
            _ => bail!("Input not fully consumed, remains: {:?}", s),
        },
        Err(nom::Err::Error(e) | nom::Err::Failure(e)) => Err(eyre!(e.to_string())),
        Err(nom::Err::Incomplete(_)) => Err(eyre!("Incomplete input")),
    }
}

// --------------------------------------------------------------------------

pub fn parse_usize(input: &str) -> IResult<&str, usize> {
    map_res(digit1, str::parse)(input)
}

pub fn parse_i32(input: &str) -> IResult<&str, i32> {
    map_res(digit1, str::parse)(input)
}

pub fn parse_i64(input: &str) -> IResult<&str, i64> {
    map_res(digit1, str::parse)(input)
}

// --------------------------------------------------------------------------

pub fn scale<T>(value: T, min: T, max: T, a: T, b: T) -> T
where
    T: Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Add<Output = T> + Copy,
{
    (b - a) * (value - min) / (max - min) + a
}
