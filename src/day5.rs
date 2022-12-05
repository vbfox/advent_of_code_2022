use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    str::FromStr,
};

use eyre::eyre;
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{digit1, newline},
    combinator::{self, map_res},
    multi::{many0, many1, separated_list0, separated_list1},
    IResult,
};

#[derive(Debug, Clone)]
struct Instruction {
    amount: usize,
    from: usize,
    to: usize,
}

fn parse_usize(input: &str) -> IResult<&str, usize> {
    map_res(digit1, str::parse)(input)
}

fn parse_instruction(input: &str) -> IResult<&str, Instruction> {
    let (input, _) = tag("move ")(input)?;
    let (input, amount) = parse_usize(input)?;
    let (input, _) = tag(" from ")(input)?;
    let (input, from) = parse_usize(input)?;
    let (input, _) = tag(" to ")(input)?;
    let (input, to) = parse_usize(input)?;
    Ok((input, Instruction { amount, from, to }))
}

fn parse_instruction_lines(input: &str) -> IResult<&str, Vec<Instruction>> {
    separated_list0(newline, parse_instruction)(input)
}

impl FromStr for Instruction {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, instruction) = parse_instruction(s).map_err(|e| eyre!(e.to_owned()))?;
        Ok(instruction)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "move {} from {} to {}", self.amount, self.from, self.to)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Crate(char);

impl Display for Crate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0)
    }
}

fn parse_crate(input: &str) -> IResult<&str, Crate> {
    let (input, _) = tag("[")(input)?;
    let (input, c) = take(1usize)(input)?;
    let (input, _) = tag("]")(input)?;
    let c = c.chars().next().unwrap();
    Ok((input, Crate(c)))
}

#[derive(Debug, Clone)]
struct CrateRow(Vec<Option<Crate>>);

impl Display for CrateRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for c in &self.0 {
            if first {
                first = false;
            } else {
                write!(f, " ")?;
            }

            match c {
                Some(c) => write!(f, "{}", c)?,
                None => write!(f, "   ")?,
            }
        }

        Ok(())
    }
}

fn parse_crate_option(input: &str) -> IResult<&str, Option<Crate>> {
    let success = combinator::map(parse_crate, Some);
    let failure = combinator::map(tag("   "), |_| None);
    alt((success, failure))(input)
}

fn parse_crate_row(input: &str) -> IResult<&str, CrateRow> {
    let (input, row) = separated_list0(tag(" "), parse_crate_option)(input)?;
    Ok((input, CrateRow(row)))
}

fn parse_crate_row_lines(input: &str) -> IResult<&str, Vec<CrateRow>> {
    separated_list0(newline, parse_crate_row)(input)
}

fn parse_digits_row(input: &str) -> IResult<&str, ()> {
    let (input, _) = many0(tag(" "))(input)?;
    let (input, _) = separated_list1(many1(tag(" ")), parse_usize)(input)?;
    let (input, _) = many0(tag(" "))(input)?;

    Ok((input, ()))
}

impl FromStr for CrateRow {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, row) = parse_crate_row(s).map_err(|e| eyre!(e.to_owned()))?;
        Ok(row)
    }
}

#[derive(Debug, Clone)]
struct Crates(Vec<Vec<Crate>>);

impl Crates {
    fn apply_instruction(&mut self, i: &Instruction, is_9001: bool) -> eyre::Result<()> {
        // Find the source stack
        let crates_from = self
            .0
            .get_mut(i.from - 1)
            .ok_or_else(|| eyre!("Invalid from index"))?;

        // Take some crates from the source
        let mut crates_to_insert = Vec::new();
        for _ in 0..i.amount {
            let c = crates_from.pop().ok_or_else(|| eyre!("Invalid amount"))?;
            crates_to_insert.push(c);
        }

        if is_9001 {
            crates_to_insert.reverse();
        }

        // Find the destination stack
        let crates_to = self
            .0
            .get_mut(i.to - 1)
            .ok_or_else(|| eyre!("Invalid to index"))?;

        // Insert the crates into the destination
        crates_to.append(&mut crates_to_insert);

        Ok(())
    }

    fn tops(&self) -> String {
        let mut s = String::new();
        for row in &self.0 {
            if let Some(c) = row.last() {
                s.push(c.0);
            } else {
                s.push('_');
            }
        }
        s
    }
}

impl From<Vec<CrateRow>> for Crates {
    fn from(rows: Vec<CrateRow>) -> Self {
        // Transpose the rows into columns
        let mut crates = Vec::new();
        for row in rows {
            for (i, c) in row.0.iter().enumerate() {
                if let Some(c) = c {
                    while crates.len() < i + 1 {
                        crates.push(Vec::new());
                    }
                    crates[i].push(c.clone());
                }
            }
        }
        for col in &mut crates {
            col.reverse();
        }
        Crates(crates)
    }
}

struct Input {
    crates: Crates,
    instructions: Vec<Instruction>,
}

impl Input {
    fn apply_instructions(&self, is_9001: bool) -> eyre::Result<Crates> {
        let mut crates = self.crates.clone();

        for instruction in &self.instructions {
            crates.apply_instruction(instruction, is_9001)?;
        }

        Ok(crates)
    }
}

fn parse_input(input: &str) -> IResult<&str, Input> {
    let (input, crate_rows) = parse_crate_row_lines(input)?;
    let (input, _) = parse_digits_row(input)?;
    let (input, _) = newline(input)?;
    let (input, _) = newline(input)?;
    let (input, instructions) = parse_instruction_lines(input)?;
    let crates: Crates = crate_rows.into();

    Ok((
        input,
        Input {
            crates,
            instructions,
        },
    ))
}

fn load_from_reader(reader: impl BufRead) -> eyre::Result<Input> {
    let s = io::read_to_string(reader)?;
    let (_, input) = parse_input(&s).map_err(|e| eyre!(e.to_owned()))?;
    Ok(input)
}

fn load_from_file(path: impl AsRef<Path>) -> eyre::Result<Input> {
    let file = File::open(path)?;
    load_from_reader(BufReader::new(file))
}

pub fn day5() -> eyre::Result<()> {
    let lines = load_from_file("data/day5.txt")?;

    {
        let after = lines.apply_instructions(false)?;
        println!("Day 5.1: {}", after.tops());
    }
    {
        let after = lines.apply_instructions(true)?;
        println!("Day 5.2: {}", after.tops());
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use super::*;

    static TEST_VECTOR: &str = r#"    [D]
[N] [C]
[Z] [M] [P]
 1   2   3

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2"#;

    #[test]
    fn parse_instruction() {
        let s = "move 1 from 2 to 4";
        let i: Instruction = s.parse().unwrap();

        assert_eq!(i.amount, 1);
        assert_eq!(i.from, 2);
        assert_eq!(i.to, 4);
    }

    #[test]
    fn display_instruction() {
        let s = "move 1 from 2 to 4";
        let i: Instruction = s.parse().unwrap();

        assert_eq!(i.to_string(), s);
    }

    #[test]
    fn parse_crate_row() {
        let s = "    [B] [C]";
        let r: CrateRow = s.parse().unwrap();

        assert_eq!(r.0.len(), 3);
        assert_eq!(r.0[0], None);
        assert_eq!(r.0[1], Some(Crate('B')));
        assert_eq!(r.0[2], Some(Crate('C')));
    }

    #[test]
    fn display_crate_row() {
        let s = "    [B] [C]";
        let r: CrateRow = s.parse().unwrap();
        assert_eq!(r.to_string(), s);
    }

    #[test]
    fn crates() {
        let r: Vec<CrateRow> = vec![
            "    [B] [C]".parse().unwrap(),
            "[A] [D] [E]".parse().unwrap(),
        ];
        let c: Crates = r.into();

        println!("{:#?}", c);
        assert_eq!(c.0.len(), 3);
        assert_eq!(c.0[0], vec![Crate('A')]);
        assert_eq!(c.0[1], vec![Crate('D'), Crate('B')]);
        assert_eq!(c.0[2], vec![Crate('E'), Crate('C')]);
    }

    #[test]
    fn crates_instruction() {
        let r: Vec<CrateRow> = vec![
            "    [B] [C]".parse().unwrap(),
            "[A] [D] [E]".parse().unwrap(),
        ];
        let mut c: Crates = r.into();
        c.apply_instruction(&"move 2 from 2 to 1".parse().unwrap(), false)
            .unwrap();

        println!("{:#?}", c);
        assert_eq!(c.0.len(), 3);
        assert_eq!(c.0[0], vec![Crate('A'), Crate('B'), Crate('D')]);
        assert_eq!(c.0[1], vec![]);
        assert_eq!(c.0[2], vec![Crate('E'), Crate('C')]);
    }

    #[test]
    fn crates_instruction_9001() {
        let r: Vec<CrateRow> = vec![
            "    [B] [C]".parse().unwrap(),
            "[A] [D] [E]".parse().unwrap(),
        ];
        let mut c: Crates = r.into();
        c.apply_instruction(&"move 2 from 2 to 1".parse().unwrap(), true)
            .unwrap();

        println!("{:#?}", c);
        assert_eq!(c.0.len(), 3);
        assert_eq!(c.0[0], vec![Crate('A'), Crate('D'), Crate('B')]);
        assert_eq!(c.0[1], vec![]);
        assert_eq!(c.0[2], vec![Crate('E'), Crate('C')]);
    }

    #[test]
    fn can_parse_digits_row() {
        let (s, _) = parse_digits_row(" 1   2   3 ").unwrap();
        assert_eq!(s, "");
    }

    #[test]
    fn run_instructions() {
        let (_, input) = parse_input(TEST_VECTOR).unwrap();
        let after = input.apply_instructions(false).unwrap();

        assert_eq!(after.0.len(), 3);
        assert_eq!(after.0[0], vec![Crate('C')]);
        assert_eq!(after.0[1], vec![Crate('M')]);
        assert_eq!(
            after.0[2],
            vec![Crate('P'), Crate('D'), Crate('N'), Crate('Z')]
        );
        assert_eq!(after.tops(), "CMZ");
    }

    #[test]
    fn run_instructions_9001() {
        let (_, input) = parse_input(TEST_VECTOR).unwrap();
        let after = input.apply_instructions(true).unwrap();

        assert_eq!(after.0.len(), 3);
        assert_eq!(after.0[0], vec![Crate('M')]);
        assert_eq!(after.0[1], vec![Crate('C')]);
        assert_eq!(
            after.0[2],
            vec![Crate('P'), Crate('Z'), Crate('N'), Crate('D')]
        );
        assert_eq!(after.tops(), "MCD");
    }
}
