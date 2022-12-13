#![warn(clippy::all, clippy::pedantic)]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use color_eyre::eyre::Result;
use yansi::Paint;

mod utils;

mod day1;
mod day10;
mod day11;
mod day12;
mod day13;
mod day2;
mod day3;
mod day4;
mod day5;
mod day6;
mod day7;
mod day8;
mod day9;

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

fn setup() -> Result<()> {
    color_eyre::install()?;

    if cfg!(windows) && !Paint::enable_windows_ascii() {
        Paint::disable();
    }

    Ok(())
}

fn main() -> Result<()> {
    setup()?;
    // previous_days()?;

    day9::day9()?;

    Ok(())
}
