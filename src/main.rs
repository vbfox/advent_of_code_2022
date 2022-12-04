#![warn(clippy::all, clippy::pedantic)]

use color_eyre::eyre::Result;

mod utils;

mod day1;
mod day2;
mod day3;
mod day4;

#[allow(dead_code)]
fn previous_days() -> Result<()> {
    day1::day1()?;
    day2::day2()?;
    day3::day3()?;

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    previous_days()?;
    day4::day4()?;

    Ok(())
}
