use anyhow::anyhow;
use std::str::FromStr;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

#[derive(Debug)]
enum Player1 {
    Rock,
    Paper,
    Scissors,
}

impl Player1 {
    pub fn score(&self) -> i32 {
        match &self {
            Player1::Rock => 1,
            Player1::Paper => 2,
            Player1::Scissors => 3,
        }
    }
}

impl FromStr for Player1 {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "X" => Ok(Player1::Rock),
            "Y" => Ok(Player1::Paper),
            "Z" => Ok(Player1::Scissors),
            _ => Err(anyhow!("Invalid Player1 value: {}", s)),
        }
    }
}

#[derive(Debug)]
enum Player2 {
    Rock,
    Paper,
    Scissors,
}

impl FromStr for Player2 {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(Player2::Rock),
            "B" => Ok(Player2::Paper),
            "C" => Ok(Player2::Scissors),
            _ => Err(anyhow!("Invalid Player1 value: {}", s)),
        }
    }
}

#[derive(Debug)]
enum Winner {
    Player1,
    Player2,
    Draw,
}

impl Winner {
    pub fn score(&self) -> i32 {
        match &self {
            Winner::Player1 => 6,
            Winner::Player2 => 0,
            Winner::Draw => 3,
        }
    }
}

impl FromStr for Winner {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "X" => Ok(Winner::Player2),
            "Y" => Ok(Winner::Draw),
            "Z" => Ok(Winner::Player1),
            _ => Err(anyhow!("Invalid Winner value: {}", s)),
        }
    }
}

#[derive(Debug)]
struct StrategyLine {
    player1: Player1,
    player2: Player2,
}

impl StrategyLine {
    pub fn winner(&self) -> Winner {
        match (&self.player1, &self.player2) {
            (Player1::Rock, Player2::Rock) => Winner::Draw,
            (Player1::Rock, Player2::Paper) => Winner::Player2,
            (Player1::Rock, Player2::Scissors) => Winner::Player1,
            (Player1::Paper, Player2::Rock) => Winner::Player1,
            (Player1::Paper, Player2::Paper) => Winner::Draw,
            (Player1::Paper, Player2::Scissors) => Winner::Player2,
            (Player1::Scissors, Player2::Rock) => Winner::Player2,
            (Player1::Scissors, Player2::Paper) => Winner::Player1,
            (Player1::Scissors, Player2::Scissors) => Winner::Draw,
        }
    }

    pub fn score(&self) -> i32 {
        let play_score = self.player1.score();
        let win_score = self.winner().score();

        play_score + win_score
    }
}

impl FromStr for StrategyLine {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace();
        let player2 = iter
            .next()
            .ok_or_else(|| anyhow!("Can't read player1 value"))?
            .parse()?;
        let player1 = iter
            .next()
            .ok_or_else(|| anyhow!("Can't read player2 value"))?
            .parse()?;
        Ok(StrategyLine { player1, player2 })
    }
}

#[derive(Debug)]
struct StrategyLineV2 {
    expected_end: Winner,
    player2: Player2,
}

impl StrategyLineV2 {
    pub fn player1(&self) -> Player1 {
        match (&self.player2, &self.expected_end) {
            (Player2::Rock, Winner::Player1) => Player1::Paper,
            (Player2::Rock, Winner::Player2) => Player1::Scissors,
            (Player2::Rock, Winner::Draw) => Player1::Rock,
            (Player2::Paper, Winner::Player1) => Player1::Scissors,
            (Player2::Paper, Winner::Player2) => Player1::Rock,
            (Player2::Paper, Winner::Draw) => Player1::Paper,
            (Player2::Scissors, Winner::Player1) => Player1::Rock,
            (Player2::Scissors, Winner::Player2) => Player1::Paper,
            (Player2::Scissors, Winner::Draw) => Player1::Scissors,
        }
    }

    pub fn score(&self) -> i32 {
        let play_score = self.player1().score();
        let win_score = self.expected_end.score();

        play_score + win_score
    }
}

impl FromStr for StrategyLineV2 {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace();
        let player2 = iter
            .next()
            .ok_or_else(|| anyhow!("Can't read player1 value"))?
            .parse()?;
        let expected_end = iter
            .next()
            .ok_or_else(|| anyhow!("Can't read expected_end value"))?
            .parse()?;
        Ok(StrategyLineV2 {
            expected_end,
            player2,
        })
    }
}

fn load_from_reader(reader: impl BufRead) -> anyhow::Result<Vec<StrategyLine>> {
    reader.lines().map(|line| line?.parse()).collect()
}

fn load_from_file(path: impl AsRef<Path>) -> anyhow::Result<Vec<StrategyLine>> {
    let file = File::open(path)?;
    load_from_reader(BufReader::new(file))
}

fn load_from_reader_v2(reader: impl BufRead) -> anyhow::Result<Vec<StrategyLineV2>> {
    reader.lines().map(|line| line?.parse()).collect()
}

fn load_from_file_v2(path: impl AsRef<Path>) -> anyhow::Result<Vec<StrategyLineV2>> {
    let file = File::open(path)?;
    load_from_reader_v2(BufReader::new(file))
}

pub fn day2() -> anyhow::Result<()> {
    {
        let lines = load_from_file("data/day2.txt")?;
        let scores = lines.iter().map(|line| line.score()).collect::<Vec<_>>();
        let total_score = scores.iter().sum::<i32>();

        println!("Day 2.1: {}", total_score);
    }

    {
        let lines = load_from_file_v2("data/day2.txt")?;
        let scores = lines.iter().map(|line| line.score()).collect::<Vec<_>>();
        let total_score = scores.iter().sum::<i32>();

        println!("Day 2.2: {}", total_score);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn load_from_string(s: impl AsRef<str>) -> anyhow::Result<Vec<StrategyLine>> {
        let reader = Cursor::new(s.as_ref());
        load_from_reader(reader)
    }

    fn load_from_string_v2(s: impl AsRef<str>) -> anyhow::Result<Vec<StrategyLineV2>> {
        let reader = Cursor::new(s.as_ref());
        load_from_reader_v2(reader)
    }

    #[test]
    fn sample() {
        let lines = load_from_string(
            r#"A Y
            B X
            C Z"#,
        )
        .unwrap();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].score(), 8);
        assert_eq!(lines[1].score(), 1);
        assert_eq!(lines[2].score(), 6);

        let total = lines.iter().map(|line| line.score()).sum::<i32>();
        assert_eq!(total, 15);
    }

    #[test]
    fn sample_v2() {
        let lines = load_from_string_v2(
            r#"A Y
            B X
            C Z"#,
        )
        .unwrap();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].score(), 4);
        assert_eq!(lines[1].score(), 1);
        assert_eq!(lines[2].score(), 7);

        let total = lines.iter().map(|line| line.score()).sum::<i32>();
        assert_eq!(total, 12);
    }
}
