use eyre::eyre;
use std::str::FromStr;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

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

impl FromStr for Item {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 1 {
            return Err(eyre!("Invalid Item value: {}", s));
        }

        match s.chars().next() {
            Some(c @ ('A'..='Z' | 'a'..='z')) => Ok(Item(c)),
            _ => Err(eyre!("Invalid Item value: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
struct Compartment {
    items: Vec<Item>,
}

impl FromStr for Compartment {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s
            .chars()
            .map(|c| c.to_string().parse())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Compartment { items })
    }
}

#[derive(Debug, Clone)]
struct RuckSack(Compartment, Compartment);

impl RuckSack {
    pub fn find_duplicate(&self) -> eyre::Result<Item> {
        for item0 in &self.0.items {
            for item1 in &self.1.items {
                if item0 == item1 {
                    return Ok(*item0);
                }
            }
        }

        Err(eyre!("No duplicate found"))
    }

    pub fn priority(&self) -> eyre::Result<u32> {
        let duplicate = self.find_duplicate()?;
        Ok(duplicate.priority())
    }

    pub fn all_items(&self) -> Vec<Item> {
        let mut items = self.0.items.clone();
        items.extend(self.1.items.clone());
        items
    }
}

impl FromStr for RuckSack {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() % 2 != 0 {
            return Err(eyre!("Invalid RuckSack length {}: {}", s.len(), s));
        }

        let (a, b) = s.split_at(s.len() / 2);
        Ok(RuckSack(a.parse()?, b.parse()?))
    }
}

struct Group(RuckSack, RuckSack, RuckSack);

impl Group {
    pub fn find_badge(&self) -> eyre::Result<Item> {
        let items0 = self.0.all_items();
        let items1 = self.1.all_items();
        let items2 = self.2.all_items();

        for item0 in &items0 {
            for item1 in &items1 {
                if *item0 == *item1 {
                    for item2 in &items2 {
                        if *item0 == *item2 {
                            return Ok(*item0);
                        }
                    }
                }
            }
        }

        Err(eyre!("No badge found"))
    }

    pub fn priority(&self) -> eyre::Result<u32> {
        let badge = self.find_badge()?;
        Ok(badge.priority())
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

fn load_from_reader(reader: impl BufRead) -> eyre::Result<Vec<RuckSack>> {
    reader.lines().map(|line| line?.parse()).collect()
}

fn load_from_file(path: impl AsRef<Path>) -> eyre::Result<Vec<RuckSack>> {
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
        println!("Day 3.1: {}", total);
    }
    {
        let groups = get_groups(&rucksacks)?;

        let priorities = groups
            .iter()
            .map(Group::priority)
            .collect::<Result<Vec<_>, _>>()?;
        let total = priorities.iter().sum::<u32>();
        println!("Day 3.2: {}", total);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn load_from_string(s: impl AsRef<str>) -> eyre::Result<Vec<RuckSack>> {
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
