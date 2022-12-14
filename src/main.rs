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

mod day1;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day2;
mod day3;
mod day4;
mod day5;
mod day6;
mod day7;
mod day8;
mod day9;

struct Day {
    number: u8,
    func: fn(DayParams) -> Result<()>,
}

impl Day {
    fn new(index: u8, func: fn(DayParams) -> Result<()>) -> Self {
        Self {
            number: index,
            func,
        }
    }

    fn run(&self, params: DayParams) -> Result<()> {
        (self.func)(params)
    }
}

static DAYS: Lazy<Vec<Day>> = Lazy::new(|| {
    let mut result = Vec::new();
    result.push(Day::new(14, day14::day14));
    result
});

#[allow(dead_code)]
fn previous_days() -> Result<()> {
    day1::day1()?;
    day2::day2()?;
    day3::day3()?;
    day4::day4()?;
    day5::day5()?;
    day6::day6()?;
    day7::day7()?;
    day8::day8()?;
    day9::day9()?;
    day10::day10()?;
    day11::day11()?;
    day12::day12()?;
    day13::day13()?;

    Ok(())
}

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
    #[arg(long)]
    debug: Option<bool>,
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
    day.run(DayParams {
        number: day.number,
        part,
        test: args.test,
        debug: args.debug.unwrap_or(false),
    })?;
    // previous_days()?;

    // day14::day14()?;

    Ok(())
}
