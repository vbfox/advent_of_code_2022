use crate::utils::{nom_finish, DayParams, Point};
use nom::{bytes::complete::tag, combinator::map, sequence::tuple, IResult};

struct Sensor {
    position: Point,
    closest_beacon: Point,
}

struct Sensors {
    sensors: Vec<Sensor>,
}

impl Sensors {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        todo!();
    }
}

pub fn day(p: &DayParams) -> eyre::Result<()> {
    let input = &p.read_input()?;

    let sensors = nom_finish(Sensors::parse, input)?;

    p.part_1(|| Ok(()))?;

    p.part_1(|| Ok(()))?;

    Ok(())
}
