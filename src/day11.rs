use std::{cmp::Reverse, str::FromStr, time::Instant};

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, newline},
    combinator::{map, value},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::utils::{nom_finish, parse_i64, parse_usize};

fn gcd(a: i64, b: i64) -> i64 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

fn lcm(a: i64, b: i64) -> i64 {
    a * b / gcd(a, b)
}

fn lcm_iter(iter: impl Iterator<Item = i64>) -> i64 {
    iter.fold(1, lcm)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    Add(i64),
    Multiply(i64),
    Square,
}

impl Operation {
    fn apply(&self, value: i64) -> i64 {
        match self {
            Operation::Add(x) => value + x,
            Operation::Multiply(x) => value * x,
            Operation::Square => value * value,
        }
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            map(preceded(tag("old + "), parse_i64), Operation::Add),
            map(preceded(tag("old * "), parse_i64), Operation::Multiply),
            value(Operation::Square, tag("old * old")),
        ))(input)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Test {
    DivisibleBy(i64),
}

impl Test {
    fn test(self, value: i64) -> bool {
        match self {
            Test::DivisibleBy(x) => value % x == 0,
        }
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        alt((map(
            preceded(tag("divisible by "), parse_i64),
            Test::DivisibleBy,
        ),))(input)
    }

    fn value(self) -> i64 {
        match self {
            Test::DivisibleBy(x) => x,
        }
    }
}

#[derive(Debug, Clone)]
struct Monkey {
    starting_items: Vec<i64>,
    operation: Operation,
    test: Test,
    if_true: usize,
    if_false: usize,
}

impl Monkey {
    fn parse(input: &str) -> IResult<&str, Self> {
        // Monkey 0:
        let (input, _index) = delimited(tag("Monkey "), parse_usize, tag(":"))(input)?;
        let (input, _) = newline(input)?;

        //   Starting items: 79, 98
        let (input, starting_items) = preceded(
            tag("  Starting items: "),
            nom::multi::separated_list1(tag(", "), parse_i64),
        )(input)?;
        let (input, _) = newline(input)?;

        //   Operation: new = old * 19
        let (input, operation) = preceded(tag("  Operation: new = "), Operation::parse)(input)?;
        let (input, _) = newline(input)?;

        //   Test: divisible by 23
        let (input, test) = preceded(tag("  Test: "), Test::parse)(input)?;
        let (input, _) = newline(input)?;

        //     If true: throw to monkey 2
        let (input, if_true) = preceded(tag("    If true: throw to monkey "), parse_usize)(input)?;
        let (input, _) = newline(input)?;

        //     If false: throw to monkey 3
        let (input, if_false) =
            preceded(tag("    If false: throw to monkey "), parse_usize)(input)?;

        Ok((
            input,
            Self {
                starting_items,
                operation,
                test,
                if_true,
                if_false,
            },
        ))
    }
}

#[derive(Debug, Clone)]
struct Input {
    monkeys: Vec<Monkey>,
}

impl Input {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, monkeys) =
            nom::multi::separated_list0(tuple((newline, newline)), Monkey::parse)(input)?;
        let (input, _) = multispace0(input)?;
        Ok((input, Self { monkeys }))
    }
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        nom_finish(Input::parse, s)
    }
}

#[derive(Debug, Clone)]
struct MonkeyState {
    items: Vec<i64>,
    definition: Monkey,
    inspected_items: i64,
}

impl MonkeyState {
    fn new(definition: Monkey) -> Self {
        Self {
            items: definition.starting_items.clone(),
            definition,
            inspected_items: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct State {
    monkeys: Vec<MonkeyState>,
    lcm_tests: i64,
}

impl State {
    fn new(input: Input) -> Self {
        let lcm_tests = lcm_iter(input.monkeys.iter().map(|m| m.test.value()));
        Self {
            monkeys: input.monkeys.into_iter().map(MonkeyState::new).collect(),
            lcm_tests,
        }
    }

    fn round(&mut self, is_bored: bool) {
        let mut changes = vec![vec![]; self.monkeys.len()];

        for i in 0..self.monkeys.len() {
            let monkey = &mut self.monkeys[i];

            for item in &monkey.items {
                // Worry level operation
                let item = monkey.definition.operation.apply(*item);

                // Monkey gets bored with item
                let item = if is_bored { item / 3 } else { item };

                // Compute everything modulo the LCM to avoid overflow
                let item = item % self.lcm_tests;

                // Check current worry level
                if monkey.definition.test.test(item) {
                    changes[monkey.definition.if_true].push(item);
                } else {
                    changes[monkey.definition.if_false].push(item);
                }
            }

            monkey.inspected_items += monkey.items.len() as i64;
            monkey.items.clear();

            for (change_index, change) in changes.iter_mut().enumerate() {
                self.monkeys[change_index].items.extend(change.iter());
                change.clear();
            }
        }
    }

    fn rounds(&mut self, rounds: usize, is_bored: bool) {
        for _ in 0..rounds {
            self.round(is_bored);
        }
    }

    fn business_level(&self) -> i64 {
        let inspected = self
            .monkeys
            .iter()
            .map(|m| Reverse(m.inspected_items))
            .sorted()
            .collect_vec();

        let first = inspected[0].0;
        let second = inspected[1].0;

        first * second
    }
}

pub fn day11() -> eyre::Result<()> {
    let input: Input = include_str!("../data/day11.txt").parse()?;
    {
        let start = Instant::now();
        let mut state = State::new(input.clone());
        state.rounds(20, true);
        let result = state.business_level();
        let elapsed = start.elapsed();
        println!("Day 11.1: {result} ({elapsed:?})");
    }
    {
        let start = Instant::now();
        let mut state = State::new(input);
        state.rounds(10_000, false);
        let result = state.business_level();
        let elapsed = start.elapsed();
        println!("Day 11.2: {result} ({elapsed:?})");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    static TEST_MONKEY: &str = r#"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3"#;

    static TEST_INPUT: &str = r#"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1"#;

    #[test]
    fn parse_operation() -> eyre::Result<()> {
        let op = nom_finish(Operation::parse, "old * 19")?;
        assert_eq!(op, Operation::Multiply(19));
        Ok(())
    }

    #[test]
    fn parse_test() -> eyre::Result<()> {
        let op = nom_finish(Test::parse, "divisible by 17")?;
        assert_eq!(op, Test::DivisibleBy(17));
        Ok(())
    }

    #[test]
    fn parse_monkey() -> eyre::Result<()> {
        let monkey = nom_finish(Monkey::parse, TEST_MONKEY)?;
        assert_eq!(monkey.starting_items, vec![79, 98]);
        assert_eq!(monkey.operation, Operation::Multiply(19));
        assert_eq!(monkey.test, Test::DivisibleBy(23));
        assert_eq!(monkey.if_true, 2);
        assert_eq!(monkey.if_false, 3);
        Ok(())
    }

    #[test]
    fn parse() -> eyre::Result<()> {
        let input: Input = TEST_INPUT.parse()?;
        assert_eq!(input.monkeys.len(), 4);
        Ok(())
    }

    #[test]
    fn round() -> eyre::Result<()> {
        let input: Input = TEST_INPUT.parse()?;
        let mut state = State::new(input);
        state.round(true);
        assert_eq!(state.monkeys[0].items, vec![20, 23, 27, 26]);
        assert_eq!(state.monkeys[1].items, vec![2080, 25, 167, 207, 401, 1046]);
        assert_eq!(state.monkeys[2].items, vec![]);
        assert_eq!(state.monkeys[3].items, vec![]);
        Ok(())
    }

    #[test]
    fn part1() -> eyre::Result<()> {
        let input: Input = TEST_INPUT.parse()?;
        let mut state = State::new(input);
        state.rounds(20, true);
        assert_eq!(state.business_level(), 10_605);
        Ok(())
    }

    #[test]
    fn part2() -> eyre::Result<()> {
        let input: Input = TEST_INPUT.parse()?;
        let mut state = State::new(input);
        state.rounds(10_000, false);

        assert_eq!(state.business_level(), 2_713_310_158);
        Ok(())
    }
}
