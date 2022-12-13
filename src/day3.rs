use crate::utils::SingleExt;
use crate::utils::{find_common_items, CharSliceExt};
use eyre::eyre;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::str::FromStr;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Item(char);

impl Item {
    fn priority(self) -> u32 {
        match self.0 {
            c @ 'a'..='z' => c as u32 - 'a' as u32 + 1,
            c @ 'A'..='Z' => c as u32 - 'A' as u32 + 27,
            _ => panic!("Invalid Item value: {}", self.0),
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ItemParseError {
    #[error("Invalid item value: '{0}'")]
    InvalidValue(String),
}

impl TryFrom<char> for Item {
    type Error = ItemParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            c @ ('A'..='Z' | 'a'..='z') => Ok(Item(c)),
            _ => Err(ItemParseError::InvalidValue(c.to_string())),
        }
    }
}

impl FromStr for Item {
    type Err = ItemParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(c.try_into()?),
            _ => Err(ItemParseError::InvalidValue(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
struct Compartment {
    items: Vec<Item>,
}

impl Display for Compartment {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        for item in &self.items {
            write!(f, "{item}")?;
        }
        Ok(())
    }
}

impl FromStr for Compartment {
    type Err = ItemParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s
            .char_slices()
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Compartment { items })
    }
}

#[derive(Debug, Clone)]
struct RuckSack(Compartment, Compartment);

impl Display for RuckSack {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} {}", self.0, self.1)
    }
}

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum PriorityError {
    #[error("No duplicate found for priority: '{0}'")]
    NoDuplicateFound(String),
}

impl RuckSack {
    pub fn find_duplicate(&self) -> Option<Item> {
        for item0 in &self.0.items {
            for item1 in &self.1.items {
                if item0 == item1 {
                    return Some(*item0);
                }
            }
        }

        None
    }

    pub fn priority(&self) -> eyre::Result<u32> {
        let duplicate = self
            .find_duplicate()
            .ok_or_else(|| PriorityError::NoDuplicateFound(self.to_string()))?;
        Ok(duplicate.priority())
    }

    pub fn all_items(&self) -> Vec<Item> {
        let mut items = self.0.items.clone();
        items.extend(self.1.items.clone());
        items
    }
}

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum RuckSackParseError {
    #[error("Invalid input length {1} in '{0}'")]
    InvalidLength(String, usize),
    #[error("Unable to parse compartment {1}: '{0}'")]
    CompartmentParseError(#[source] ItemParseError, usize),
}

impl FromStr for RuckSack {
    type Err = RuckSackParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() % 2 != 0 {
            return Err(RuckSackParseError::InvalidLength(s.to_string(), s.len()));
        }

        let (compartment1, compartment2) = s.split_at(s.len() / 2);
        let compartment1 = compartment1
            .parse()
            .map_err(|e| RuckSackParseError::CompartmentParseError(e, 1))?;
        let compartment2 = compartment2
            .parse()
            .map_err(|e| RuckSackParseError::CompartmentParseError(e, 2))?;
        Ok(RuckSack(compartment1, compartment2))
    }
}

struct Group(RuckSack, RuckSack, RuckSack);

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum BadgePriorityError {
    #[error("No badge found for priority: '{0}'")]
    NoDuplicateFound(String),
}

impl Group {
    pub fn find_badge(&self) -> Option<Item> {
        let slices = vec![self.0.all_items(), self.1.all_items(), self.2.all_items()];
        let common_items = find_common_items(&slices);
        common_items.into_iter().single().copied()
    }

    pub fn priority(&self) -> Result<u32, BadgePriorityError> {
        let badge = self
            .find_badge()
            .ok_or_else(|| BadgePriorityError::NoDuplicateFound(self.to_string()))?;
        Ok(badge.priority())
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}\n{}\n{}", self.0, self.1, self.2)
    }
}

fn get_groups(vec: &Vec<RuckSack>) -> eyre::Result<Vec<Group>> {
    if vec.len() % 3 != 0 {
        return Err(eyre!("Invalid RuckSacks length: {}", vec.len()));
    }

    let mut groups = Vec::new();
    for i in 0..vec.len() / 3 {
        let a = vec[i * 3].clone();
        let b = vec[i * 3 + 1].clone();
        let c = vec[i * 3 + 2].clone();
        groups.push(Group(a, b, c));
    }

    Ok(groups)
}

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum LoadError {
    #[error("Unable to parse RuckSack")]
    RuckSackParseError(#[from] RuckSackParseError),
    #[error("Unable to read line")]
    LineReadError(#[from] io::Error),
}

fn load_from_reader(reader: impl BufRead) -> Result<Vec<RuckSack>, LoadError> {
    reader.lines().map(|line| Ok(line?.parse()?)).collect()
}

fn load_from_file(path: impl AsRef<Path>) -> Result<Vec<RuckSack>, LoadError> {
    let file = File::open(path)?;
    load_from_reader(BufReader::new(file))
}

pub fn day3() -> eyre::Result<()> {
    let rucksacks = load_from_file("data/day3.txt")?;

    {
        let priorities = rucksacks
            .iter()
            .map(RuckSack::priority)
            .collect::<Result<Vec<_>, _>>()?;
        let total = priorities.iter().sum::<u32>();
        println!("Day 3.1: {total}");
    }
    {
        let groups = get_groups(&rucksacks)?;

        let priorities = groups
            .iter()
            .map(Group::priority)
            .collect::<Result<Vec<_>, _>>()?;
        let total = priorities.iter().sum::<u32>();
        println!("Day 3.2: {total}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::io::Cursor;

    fn load_from_string(s: impl AsRef<str>) -> Result<Vec<RuckSack>, LoadError> {
        let reader = Cursor::new(s.as_ref());
        load_from_reader(reader)
    }

    #[test]
    fn sample() {
        let lines = load_from_string(
            r#"vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw"#,
        )
        .unwrap();

        assert_eq!(lines.len(), 6);
        assert_eq!(lines[0].priority().unwrap(), 16);
        assert_eq!(lines[1].priority().unwrap(), 38);
        assert_eq!(lines[2].priority().unwrap(), 42);
        assert_eq!(lines[3].priority().unwrap(), 22);
        assert_eq!(lines[4].priority().unwrap(), 20);
        assert_eq!(lines[5].priority().unwrap(), 19);

        let total = lines
            .iter()
            .map(|line| line.priority().unwrap())
            .sum::<u32>();
        assert_eq!(total, 157);
    }

    #[test]
    fn sample2() {
        let lines = load_from_string(
            r#"vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw"#,
        )
        .unwrap();

        let groups = get_groups(&lines).unwrap();

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].priority().unwrap(), 18);
        assert_eq!(groups[1].priority().unwrap(), 52);

        let total = groups
            .iter()
            .map(|group| group.priority().unwrap())
            .sum::<u32>();
        assert_eq!(total, 70);
    }
}
