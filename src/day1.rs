use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader},
    ops::{Add, Sub},
    path::Path,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Calories(i32);

impl Display for Calories {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Add for Calories {
    type Output = Calories;

    fn add(self, other: Calories) -> Calories {
        Calories(self.0 + other.0)
    }
}

impl Sub for Calories {
    type Output = Calories;

    fn sub(self, other: Calories) -> Calories {
        Calories(self.0 - other.0)
    }
}

#[derive(Clone, Debug)]
struct Elf {
    pub calories: Vec<Calories>,
}

impl Elf {
    pub fn new(calories: Vec<Calories>) -> Self {
        Elf { calories }
    }

    pub fn total_calories(&self) -> Calories {
        self.calories.iter().fold(Calories(0), |acc, c| acc + *c)
    }
}

fn load_elves_calories_from_reader(reader: impl BufRead) -> anyhow::Result<Vec<Elf>> {
    let mut elves = Vec::<Elf>::new();
    let mut current_calories = Vec::<Calories>::new();

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            if !current_calories.is_empty() {
                elves.push(Elf::new(current_calories));
                current_calories = Vec::new();
            }
        } else {
            let calories_int = line.parse::<i32>()?;
            let calories = Calories(calories_int);
            current_calories.push(calories);
        }
    }

    if !current_calories.is_empty() {
        elves.push(Elf::new(current_calories));
    }

    Ok(elves)
}

fn load_elves_calories_from_file(path: impl AsRef<Path>) -> anyhow::Result<Vec<Elf>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    load_elves_calories_from_reader(reader)
}

// --------------------------------------------------------------------

pub fn day1() -> anyhow::Result<()> {
    let elves = load_elves_calories_from_file("data/day1.txt")?;

    let mut sorted_elves = elves.clone();
    sorted_elves.sort_by(|a, b| a.total_calories().cmp(&b.total_calories()));

    let max_elve = sorted_elves.last()
        .ok_or(anyhow::anyhow!("No elves found"))?;

    println!("Day 1.1: {}", max_elve.total_calories());

    let max_3_elves = sorted_elves.iter()
        .rev()
        .take(3).collect::<Vec<_>>();

    let max_3_elves_calories = max_3_elves.iter()
        .fold(Calories(0), |acc, e| acc + e.total_calories());

    println!("Day 1.2: {}", max_3_elves_calories);

    Ok(())
}

// --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn elf_total_calories() {
        let elf = Elf::new(vec![Calories(1), Calories(2), Calories(3)]);
        assert_eq!(elf.total_calories(), Calories(6));
    }

    #[test]
    fn load_elves_calories_from_reader_data() {
        let input = r#"1
2
3

4
5"#;
        let reader = Cursor::new(input);
        let elves = load_elves_calories_from_reader(reader).unwrap();
        assert_eq!(elves.len(), 2);
        assert_eq!(elves[0].total_calories(), Calories(6));
        assert_eq!(elves[1].total_calories(), Calories(9));
    }

    #[test]
    fn load_elves_calories_from_reader_weird() {
        let input = r#"


1
2
3



4
5


"#;
        let reader = Cursor::new(input);
        let elves = load_elves_calories_from_reader(reader).unwrap();
        assert_eq!(elves.len(), 2);
        assert_eq!(elves[0].total_calories(), Calories(6));
        assert_eq!(elves[1].total_calories(), Calories(9));
    }

    #[test]
    fn load_elves_calories_from_reader_empty() {
        let input = "";
        let reader = Cursor::new(input);
        let elves = load_elves_calories_from_reader(reader).unwrap();
        assert_eq!(elves.len(), 0);
    }
}