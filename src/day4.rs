use eyre::eyre;
use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    ops::RangeInclusive,
    path::Path,
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Section(u32);

impl FromStr for Section {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>().map(Self)
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
struct SectionRange(RangeInclusive<Section>);

impl SectionRange {
    fn contains_range(&self, other: &SectionRange) -> bool {
        self.0.contains(other.0.start()) && self.0.contains(other.0.end())
    }

    fn overlaps(&self, other: &SectionRange) -> bool {
        self.0.contains(other.0.start())
            || self.0.contains(other.0.end())
            || other.0.contains(self.0.start())
            || other.0.contains(self.0.end())
    }
}

impl FromStr for SectionRange {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');

        match (parts.next(), parts.next(), parts.next()) {
            (Some(from), Some(to), None) => {
                let from: Section = from.parse()?;
                let to: Section = to.parse()?;
                Ok(Self(from..=to))
            }
            _ => Err(eyre!("Not a range: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
struct Pair {
    first: SectionRange,
    second: SectionRange,
}

impl Pair {
    fn fully_contains(&self) -> bool {
        self.first.contains_range(&self.second) || self.second.contains_range(&self.first)
    }

    fn overlaps(&self) -> bool {
        self.first.overlaps(&self.second)
    }
}

impl FromStr for Pair {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(',');

        match (parts.next(), parts.next(), parts.next()) {
            (Some(first), Some(second), None) => Ok(Self {
                first: first.parse()?,
                second: second.parse()?,
            }),
            _ => Err(eyre!("Not a pair: {}", s)),
        }
    }
}

fn load_from_reader(reader: impl BufRead) -> eyre::Result<Vec<Pair>> {
    reader.lines().map(|line| line?.parse()).collect()
}

fn load_from_file(path: impl AsRef<Path>) -> eyre::Result<Vec<Pair>> {
    let file = File::open(path)?;
    load_from_reader(BufReader::new(file))
}

pub fn day4() -> eyre::Result<()> {
    let lines = load_from_file("data/day4.txt")?;

    {
        let count = lines.iter().filter(|l| l.fully_contains()).count();
        println!("Day 4.1: {}", count);
    }

    {
        let count = lines.iter().filter(|l| l.overlaps()).count();
        println!("Day 4.2: {}", count);
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use std::io::Cursor;

    use super::*;

    static TEST_VECTOR: &str = r#"2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8
"#;

    fn load_from_string(s: impl AsRef<str>) -> eyre::Result<Vec<Pair>> {
        let reader = Cursor::new(s.as_ref());
        load_from_reader(reader)
    }

    #[test]
    fn sample() {
        let lines = load_from_string(TEST_VECTOR).unwrap();

        println!("{:?}", lines);
        assert_eq!(lines.len(), 6);
        assert_eq!(lines[0].fully_contains(), false);
        assert_eq!(lines[1].fully_contains(), false);
        assert_eq!(lines[2].fully_contains(), false);
        assert_eq!(lines[3].fully_contains(), true);
        assert_eq!(lines[4].fully_contains(), true);
        assert_eq!(lines[5].fully_contains(), false);

        let count = lines.iter().filter(|l| l.fully_contains()).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn sample2() {
        let lines = load_from_string(TEST_VECTOR).unwrap();

        println!("{:?}", lines);
        assert_eq!(lines.len(), 6);
        assert_eq!(lines[0].overlaps(), false);
        assert_eq!(lines[1].overlaps(), false);
        assert_eq!(lines[2].overlaps(), true);
        assert_eq!(lines[3].overlaps(), true);
        assert_eq!(lines[4].overlaps(), true);
        assert_eq!(lines[5].overlaps(), true);

        let count = lines.iter().filter(|l| l.overlaps()).count();
        assert_eq!(count, 4);
    }

    #[test]
    fn overlaps() {
        assert_eq!("20-22,1-20".parse::<Pair>().unwrap().overlaps(), true);
        assert_eq!("20-22,22-30".parse::<Pair>().unwrap().overlaps(), true);

        assert_eq!("20-20,1-20".parse::<Pair>().unwrap().overlaps(), true);
        assert_eq!("20-20,20-30".parse::<Pair>().unwrap().overlaps(), true);

        assert_eq!("20-22,1-100".parse::<Pair>().unwrap().overlaps(), true);
        assert_eq!("20-22,21-21".parse::<Pair>().unwrap().overlaps(), true);
    }
}
