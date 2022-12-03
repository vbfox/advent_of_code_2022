mod day1;
mod day2;
mod day3;

#[allow(dead_code)]
fn previous_days() -> anyhow::Result<()> {
    day1::day1()?;
    day2::day2()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    day3::day3()?;
    Ok(())
}
