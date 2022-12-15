#![allow(dead_code)]

use std::fmt::{Debug, Display};
use std::{env, fmt};
use std::{fs, path::PathBuf, time::Instant};

use eyre::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayPart {
    One,
    Two,
    Both,
}

impl Display for DayPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DayPart::One => write!(f, "1"),
            DayPart::Two => write!(f, "2"),
            DayPart::Both => write!(f, "*"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DayParams {
    pub number: u8,
    pub part: DayPart,
    pub test: bool,
    pub debug: bool,
}

impl DayParams {
    pub fn input_path(&self) -> PathBuf {
        let file_name = if self.test {
            format!("day{:02}_test.txt", self.number)
        } else {
            format!("day{:02}.txt", self.number)
        };
        let path = PathBuf::from_iter(&["data", &file_name]);
        path
    }

    pub fn read_input(&self) -> eyre::Result<String> {
        let path = self.input_path();

        fs::read_to_string(path.clone())
            .wrap_err_with(|| format!("Failed to read {:?} from {:?}", path, env::current_dir()))
    }

    fn run_part(&self, part: DayPart) -> bool {
        match (part, self.part) {
            (DayPart::One, DayPart::One) => true,
            (DayPart::Two, DayPart::Two) => true,
            (DayPart::One, DayPart::Both) => true,
            (DayPart::Two, DayPart::Both) => true,
            (DayPart::Both, _) => true,
            _ => false,
        }
    }

    fn part_raw<F>(&self, f: F, part: DayPart) -> eyre::Result<()>
    where
        F: Fn() -> eyre::Result<()>,
    {
        if self.run_part(part) {
            f()
        } else {
            Ok(())
        }
    }

    pub fn part_1_raw<F>(&self, f: F) -> eyre::Result<()>
    where
        F: Fn() -> eyre::Result<()>,
    {
        self.part_raw(f, DayPart::One)
    }

    pub fn part_2_raw<F>(&self, f: F) -> eyre::Result<()>
    where
        F: Fn() -> eyre::Result<()>,
    {
        self.part_raw(f, DayPart::Two)
    }

    fn part<T, F>(&self, f: F, part: DayPart) -> eyre::Result<()>
    where
        F: Fn() -> eyre::Result<T>,
        T: Debug,
    {
        self.part_raw(
            || {
                let start = Instant::now();
                let result = f()?;
                let elapsed = start.elapsed();
                let number = self.number;
                println!("Day {number}.{part}: {result:?} ({elapsed:?})");
                Ok(())
            },
            part,
        )
    }

    pub fn part_1<T, F>(&self, f: F) -> eyre::Result<()>
    where
        F: Fn() -> eyre::Result<T>,
        T: Debug,
    {
        self.part(f, DayPart::One)
    }

    pub fn part_2<T, F>(&self, f: F) -> eyre::Result<()>
    where
        F: Fn() -> eyre::Result<T>,
        T: Debug,
    {
        self.part(f, DayPart::Two)
    }
}
