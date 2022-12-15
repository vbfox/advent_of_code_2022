#![warn(clippy::all, clippy::pedantic)]
#![feature(let_chains)]
#![feature(iter_advance_by)]
#![feature(extend_one)]

use clap::Parser;
use color_eyre::eyre::Result;
use once_cell::sync::Lazy;
use utils::DayParams;
use yansi::Paint;

mod utils;

mod day01;
mod day02;
mod day03;
mod day04;
mod day05;
mod day06;
mod day07;
mod day08;
mod day09;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;

struct Day {
    number: u8,
    func: fn(&DayParams) -> Result<()>,
}

impl Day {
    fn new(index: u8, func: fn(&DayParams) -> Result<()>) -> Self {
        Self {
            number: index,
            func,
        }
    }

    fn run(&self, params: &DayParams) -> Result<()> {
        (self.func)(params)
    }
}

static DAYS: Lazy<Vec<Day>> = Lazy::new(|| {
    vec![
        Day::new(1, day01::day01),
        Day::new(2, day02::day02),
        Day::new(3, day03::day03),
        Day::new(4, day04::day04),
        Day::new(5, day05::day05),
        Day::new(6, day06::day06),
        Day::new(7, day07::day07),
        Day::new(8, day08::day08),
        Day::new(9, day09::day09),
        Day::new(10, day10::day10),
        Day::new(11, day11::day11),
        Day::new(12, day12::day12),
        Day::new(13, day13::day13),
        Day::new(14, day14::day14),
        Day::new(15, day15::day),
    ]
});

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Day to run, defaults to the latest
    #[arg(short, long)]
    day: Option<u8>,

    /// Part to run, defaults to both
    #[arg(short, long)]
    part: Option<u8>,

    /// Use the dayXX_test.txt file instead of dayXX.txt
    #[arg(short, long, default_value_t = false)]
    test: bool,

    /// Enable debug output
    #[arg(long, default_value_t = false)]
    debug: bool,
}

fn setup() -> Result<()> {
    color_eyre::install()?;

    if cfg!(windows) && !Paint::enable_windows_ascii() {
        Paint::disable();
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    setup()?;

    let day = args
        .day
        .and_then(|number| DAYS.iter().find(|d| d.number == number))
        .unwrap_or(DAYS.iter().max_by_key(|d| d.number).unwrap());

    let part = match args.part {
        Some(1) => utils::DayPart::One,
        Some(2) => utils::DayPart::Two,
        _ => utils::DayPart::Both,
    };
    day.run(&DayParams {
        number: day.number,
        part,
        test: args.test,
        debug: args.debug,
    })?;
    // previous_days()?;

    // day14::day14()?;

    Ok(())
}
