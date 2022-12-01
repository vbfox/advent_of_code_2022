use std::{
    cmp::Reverse,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader},
    iter::Sum,
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

impl Sum<Self> for Calories {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Calories(0), |acc, e| acc + e)
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
    let mut elves = load_elves_calories_from_file("data/day1.txt")?;

    elves.sort_by_key(|e| Reverse(e.total_calories()));

    let max_elve = elves
        .first()
        .ok_or_else(|| anyhow::anyhow!("No elves found"))?;

    println!("Day 1.1: {}", max_elve.total_calories());

    let max_3_elves_calories: Calories = elves.iter().take(3).map(|e| e.total_calories()).sum();

    println!("Day 1.2: {}", max_3_elves_calories);

    Ok(())
}

// --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn load_elves_calories_from_string(s: impl AsRef<str>) -> anyhow::Result<Vec<Elf>> {
        let reader = Cursor::new(s.as_ref());
        load_elves_calories_from_reader(reader)
    }

    #[test]
    fn elf_total_calories() {
        let elf = Elf::new(vec![Calories(1), Calories(2), Calories(3)]);
        assert_eq!(elf.total_calories(), Calories(6));
    }

    #[test]
    fn load_elves_calories_from_reader_data() {
        let elves = load_elves_calories_from_string(
            r#"1
2
3

4
5"#,
        )
        .unwrap();

        assert_eq!(elves.len(), 2);
        assert_eq!(elves[0].total_calories(), Calories(6));
        assert_eq!(elves[1].total_calories(), Calories(9));
    }

    #[test]
    fn load_elves_calories_from_reader_weird() {
        let elves = load_elves_calories_from_string(
            r#"


1
2
3



4
5


"#,
        )
        .unwrap();

        assert_eq!(elves.len(), 2);
        assert_eq!(elves[0].total_calories(), Calories(6));
        assert_eq!(elves[1].total_calories(), Calories(9));
    }

    #[test]
    fn load_elves_calories_from_reader_empty() {
        let elves = load_elves_calories_from_string("").unwrap();

        assert_eq!(elves.len(), 0);
    }
}
