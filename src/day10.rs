use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    time::Instant,
};
use yansi::Paint;

use crate::utils::DayParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Instruction {
    Noop,
    AddX(i32),
}

impl Instruction {
    fn cycles(self) -> usize {
        match self {
            Self::Noop => 1,
            Self::AddX(_) => 2,
        }
    }
}

impl FromStr for Instruction {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();

        match (parts.next(), parts.next(), parts.next()) {
            (Some(instr), None, None) => match instr {
                "noop" => Ok(Self::Noop),
                _ => return Err(eyre::eyre!("Unknown instruction: {}", instr)),
            },
            (Some(instr), Some(param), None) => match instr {
                "addx" => {
                    let value = param.parse::<i32>()?;
                    Ok(Self::AddX(value))
                }
                _ => return Err(eyre::eyre!("Unknown instruction: {}", instr)),
            },
            _ => return Err(eyre::eyre!("Invalid instruction format: {}", s)),
        }
    }
}

fn parse_instructions(input: &str) -> eyre::Result<Vec<Instruction>> {
    input.lines().map(str::parse).collect::<Result<Vec<_>, _>>()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CurrentInstruction {
    instruction: Instruction,
    end_cycle: usize,
}

impl CurrentInstruction {
    fn new(instruction: Instruction, cycle: usize) -> Self {
        Self {
            instruction,
            // An instrucion that start at Cycle X for 1 cycle will end at Cycle X
            end_cycle: cycle + instruction.cycles() - 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sample {
    cycle: usize,
    x: i32,
}

impl Sample {
    fn new(cycle: usize, x: i32) -> Self {
        Self { cycle, x }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn signal_strength(&self) -> i32 {
        self.cycle as i32 * self.x
    }
}

struct Signal(Vec<Sample>);

impl Signal {
    fn new(samples: Vec<Sample>) -> Self {
        Self(samples)
    }

    fn interesting(&self) -> Vec<&Sample> {
        let mut interesting = Vec::<&Sample>::new();

        let mut iter = self.0.iter();
        match (iter.advance_by(19), iter.next()) {
            (Ok(_), Some(value)) => {
                interesting.push(value);
            }
            _ => return interesting,
        }

        loop {
            match (iter.advance_by(39), iter.next()) {
                (Ok(_), Some(value)) => {
                    interesting.push(value);
                }
                _ => return interesting,
            }
        }
    }

    fn signal_strength(&self) -> i32 {
        let interesting = self.interesting();
        interesting
            .iter()
            .map(|sample| sample.signal_strength())
            .sum()
    }
}

const SCREEN_WIDTH: usize = 40;

struct Screen(Vec<Vec<char>>);

impl Screen {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn horizontal_position(&self) -> usize {
        let raw_pos = self.0.last().map_or(0, std::vec::Vec::len);
        if raw_pos >= SCREEN_WIDTH {
            0
        } else {
            raw_pos
        }
    }

    fn draw(&mut self, c: char) {
        let mut line: Option<&mut Vec<char>> = None;
        if let Some(last_line) = self.0.last_mut() {
            if last_line.len() < SCREEN_WIDTH {
                line = Some(last_line);
            }
        }

        if line.is_none() {
            self.0.push(Vec::new());
            line = self.0.last_mut();
        }

        let line = line.expect("line should be Some");
        line.push(c);
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn sprite_is_visible(&self, center: i32, width: i32) -> bool {
        let left = center - width / 2;
        let right = center + width / 2;
        let current_pos = self.horizontal_position() as i32;
        current_pos >= left && current_pos <= right
    }

    fn draw_sprite(&mut self, center: i32, width: i32) {
        if self.sprite_is_visible(center, width) {
            self.draw('#');
        } else {
            self.draw('.');
        }
    }
}

impl Display for Screen {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for line in &self.0 {
            for c in line {
                write!(f, "{c}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MatchineState {
    x: i32,
    pc: usize,
    cycle: usize,
    current_instruction: Option<CurrentInstruction>,
    instructions: Vec<Instruction>,
}

impl MatchineState {
    fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            x: 1,
            pc: 0,
            cycle: 0,
            current_instruction: None,
            instructions,
        }
    }

    fn apply_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Noop => {}
            Instruction::AddX(x) => self.x += x,
        }
    }

    fn step(&mut self) {
        // Start executing an instruction if we are not already executing one
        if self.current_instruction.is_none() && self.pc < self.instructions.len() {
            let instruction = self.instructions[self.pc];
            self.current_instruction = Some(CurrentInstruction::new(instruction, self.cycle));
        }

        // If the current instruction is finished executing, apply it's effects
        if let Some(current_instruction) = &self.current_instruction {
            if current_instruction.end_cycle == self.cycle {
                self.apply_instruction(current_instruction.instruction);
                self.pc += 1;
                self.current_instruction = None;
            }
        }

        self.cycle += 1;
    }

    fn is_running(&self) -> bool {
        self.pc < self.instructions.len()
    }

    fn run(&mut self) -> Signal {
        let mut x_values = Vec::new();
        x_values.reserve(self.instructions.len());

        while self.is_running() {
            let x_for_cycle = self.x;
            self.step();
            x_values.push(Sample::new(self.cycle, x_for_cycle));
        }

        Signal::new(x_values)
    }

    fn run_and_draw(&mut self) -> Screen {
        let mut screen = Screen::new();

        while self.is_running() {
            let x = self.x;
            self.step();
            screen.draw_sprite(x, 3);
        }

        screen
    }
}

pub fn day10(p: DayParams) -> eyre::Result<()> {
    let instructions = parse_instructions(&p.read_input()?)?;
    {
        let start = Instant::now();
        let mut state = MatchineState::new(instructions.clone());
        let signal = state.run();
        let result = signal.signal_strength();
        let elapsed = start.elapsed();
        println!("Day 10.1: {result} ({elapsed:?})");
    }
    {
        let start = Instant::now();
        let mut state = MatchineState::new(instructions);
        let screen = state.run_and_draw();
        let elapsed = start.elapsed();
        let nice_output = screen
            .to_string()
            .replace('.', " ")
            .replace('#', &Paint::yellow("█").to_string());
        println!("Day 10.2: ({elapsed:?})");
        println!("{nice_output}");
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    static TEST_VECTOR: &str = r#"addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop"#;

    #[test]
    fn simple() {
        let instructions = vec![
            Instruction::Noop,
            Instruction::AddX(3),
            Instruction::AddX(-5),
        ];
        let mut state = MatchineState::new(instructions);
        state.step();
        assert_eq!(state.x, 1);
        assert_eq!(state.pc, 1);
        assert_eq!(state.cycle, 1);
        state.step();
        assert_eq!(state.x, 1);
        assert_eq!(state.pc, 1);
        assert_eq!(state.cycle, 2);
        state.step();
        assert_eq!(state.x, 4);
        assert_eq!(state.pc, 2);
        assert_eq!(state.cycle, 3);
        state.step();
        assert_eq!(state.x, 4);
        assert_eq!(state.pc, 2);
        assert_eq!(state.cycle, 4);
        state.step();
        assert_eq!(state.x, -1);
        assert_eq!(state.pc, 3);
        assert_eq!(state.cycle, 5);
        assert_eq!(state.is_running(), false);
    }

    #[test]
    fn part_1() {
        let instructions = parse_instructions(TEST_VECTOR).unwrap();
        let mut state = MatchineState::new(instructions);
        let signal = state.run();
        let interesting = signal.interesting();
        assert_eq!(interesting.len(), 6);
        assert_eq!(interesting[0].signal_strength(), 420);
        assert_eq!(interesting[1].signal_strength(), 1140);
        assert_eq!(interesting[2].signal_strength(), 1800);
        assert_eq!(interesting[3].signal_strength(), 2940);
        assert_eq!(interesting[4].signal_strength(), 2880);
        assert_eq!(interesting[5].signal_strength(), 3960);
        assert_eq!(signal.signal_strength(), 13140);
    }

    #[test]
    fn part_2() {
        let instructions = parse_instructions(TEST_VECTOR).unwrap();
        let mut state = MatchineState::new(instructions);
        let screen = state.run_and_draw();
        let expected = r#"##..##..##..##..##..##..##..##..##..##..
###...###...###...###...###...###...###.
####....####....####....####....####....
#####.....#####.....#####.....#####.....
######......######......######......####
#######.......#######.......#######.....
"#;
        assert_eq!(screen.to_string(), expected);
    }
}
