#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result as AnyResult;

mod day1;
mod day2;
mod day4;

#[allow(dead_code)]
fn previous_days() -> AnyResult<()> {
    day1::day1()?;
    day2::day2()?;

    Ok(())
}

fn main() -> AnyResult<()> {
    day4::day4()?;

    Ok(())
}
