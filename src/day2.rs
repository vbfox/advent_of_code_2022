use std::io;
use std::str::FromStr;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};
use thiserror::Error;

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

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum MoveFromStrError {
    #[error("Unknown move for player 1: '{0}'")]
    UnknownPlayer1Move(String),
    #[error("Unknown move for player 2: '{0}'")]
    UnknownPlayer2Move(String),
    #[error("Unknown winning move: '{0}'")]
    UnknownWinnerMove(String),
}

impl FromStr for Player1 {
    type Err = MoveFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "X" => Ok(Player1::Rock),
            "Y" => Ok(Player1::Paper),
            "Z" => Ok(Player1::Scissors),
            _ => Err(MoveFromStrError::UnknownPlayer1Move(s.to_string())),
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
    type Err = MoveFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(Player2::Rock),
            "B" => Ok(Player2::Paper),
            "C" => Ok(Player2::Scissors),
            _ => Err(MoveFromStrError::UnknownPlayer2Move(s.to_string())),
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
    type Err = MoveFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "X" => Ok(Winner::Player2),
            "Y" => Ok(Winner::Draw),
            "Z" => Ok(Winner::Player1),
            _ => Err(MoveFromStrError::UnknownWinnerMove(s.to_string())),
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
            (Player1::Rock, Player2::Rock)
            | (Player1::Paper, Player2::Paper)
            | (Player1::Scissors, Player2::Scissors) => Winner::Draw,
            (Player1::Rock, Player2::Scissors)
            | (Player1::Paper, Player2::Rock)
            | (Player1::Scissors, Player2::Paper) => Winner::Player1,
            (Player1::Rock, Player2::Paper)
            | (Player1::Paper, Player2::Scissors)
            | (Player1::Scissors, Player2::Rock) => Winner::Player2,
        }
    }

    pub fn score(&self) -> i32 {
        let play_score = self.player1.score();
        let win_score = self.winner().score();

        play_score + win_score
    }
}

#[derive(Error, Debug)]
pub enum LineFromStrError {
    #[error("Invalid line format: {0}")]
    InvalidLine(String),
    #[error("Unable to parse move on line '{line}': {source}")]
    UnableToParseMove {
        line: String,
        source: MoveFromStrError,
    },
    #[error("Unable to read line")]
    UnableToRead(#[from] io::Error),
}

impl FromStr for StrategyLine {
    type Err = LineFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace();

        match (iter.next(), iter.next(), iter.next()) {
            (Some(player2), Some(player1), None) => Ok(StrategyLine {
                player1: player1
                    .parse()
                    .map_err(|source| LineFromStrError::UnableToParseMove {
                        line: s.to_string(),
                        source,
                    })?,
                player2: player2
                    .parse()
                    .map_err(|source| LineFromStrError::UnableToParseMove {
                        line: s.to_string(),
                        source,
                    })?,
            }),
            _ => Err(LineFromStrError::InvalidLine(s.to_string())),
        }
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
            (Player2::Rock, Winner::Player1)
            | (Player2::Paper, Winner::Draw)
            | (Player2::Scissors, Winner::Player2) => Player1::Paper,
            (Player2::Rock, Winner::Player2)
            | (Player2::Paper, Winner::Player1)
            | (Player2::Scissors, Winner::Draw) => Player1::Scissors,
            (Player2::Rock, Winner::Draw)
            | (Player2::Paper, Winner::Player2)
            | (Player2::Scissors, Winner::Player1) => Player1::Rock,
        }
    }

    pub fn score(&self) -> i32 {
        let play_score = self.player1().score();
        let win_score = self.expected_end.score();

        play_score + win_score
    }
}

impl FromStr for StrategyLineV2 {
    type Err = LineFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace();
        match (iter.next(), iter.next(), iter.next()) {
            (Some(player2), Some(expected_end), None) => Ok(StrategyLineV2 {
                expected_end: expected_end.parse().map_err(|source| {
                    LineFromStrError::UnableToParseMove {
                        line: s.to_string(),
                        source,
                    }
                })?,
                player2: player2
                    .parse()
                    .map_err(|source| LineFromStrError::UnableToParseMove {
                        line: s.to_string(),
                        source,
                    })?,
            }),
            _ => Err(LineFromStrError::InvalidLine(s.to_string())),
        }
    }
}

fn load_from_reader(reader: impl BufRead) -> Result<Vec<StrategyLine>, LineFromStrError> {
    reader.lines().map(|line| line?.parse()).collect()
}

fn load_from_file(path: impl AsRef<Path>) -> Result<Vec<StrategyLine>, LineFromStrError> {
    let file = File::open(path)?;
    load_from_reader(BufReader::new(file))
}

fn load_from_reader_v2(reader: impl BufRead) -> Result<Vec<StrategyLineV2>, LineFromStrError> {
    reader.lines().map(|line| line?.parse()).collect()
}

fn load_from_file_v2(path: impl AsRef<Path>) -> Result<Vec<StrategyLineV2>, LineFromStrError> {
    let file = File::open(path)?;
    load_from_reader_v2(BufReader::new(file))
}

pub fn day2() -> eyre::Result<()> {
    {
        let lines = load_from_file("data/day2.txt")?;
        let scores = lines.iter().map(StrategyLine::score).collect::<Vec<_>>();
        let total_score = scores.iter().sum::<i32>();

        println!("Day 2.1: {total_score}");
    }

    {
        let lines = load_from_file_v2("data/day2.txt")?;
        let scores = lines.iter().map(StrategyLineV2::score).collect::<Vec<_>>();
        let total_score = scores.iter().sum::<i32>();

        println!("Day 2.2: {total_score}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn load_from_string(s: impl AsRef<str>) -> Result<Vec<StrategyLine>, LineFromStrError> {
        let reader = Cursor::new(s.as_ref());
        load_from_reader(reader)
    }

    fn load_from_string_v2(s: impl AsRef<str>) -> Result<Vec<StrategyLineV2>, LineFromStrError> {
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

        let total = lines.iter().map(StrategyLine::score).sum::<i32>();
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

        let total = lines.iter().map(StrategyLineV2::score).sum::<i32>();
        assert_eq!(total, 12);
    }
}
