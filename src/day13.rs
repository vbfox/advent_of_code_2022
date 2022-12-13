use std::time::Instant;

use itertools::{EitherOrBoth, Itertools};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, newline},
    combinator::map,
    multi::{many0, separated_list0},
    sequence::{delimited, pair, terminated, tuple},
    IResult,
};

use crate::utils::{nom_finish, parse_i32};

#[derive(Debug, PartialEq, Eq, Clone)]
enum Paket {
    Integer(i32),
    List(Vec<Paket>),
}

impl Paket {
    fn first_divider() -> Self {
        Paket::List(vec![Paket::List(vec![Paket::Integer(2)])])
    }

    fn second_divider() -> Self {
        Paket::List(vec![Paket::List(vec![Paket::Integer(6)])])
    }

    fn parse(input: &str) -> IResult<&str, Paket> {
        let element_parser = alt((Paket::parse, map(parse_i32, Paket::Integer)));
        let list_parser = map(separated_list0(tag(","), element_parser), Paket::List);
        let mut parser = delimited(char('['), list_parser, char(']'));
        parser(input)
    }
}

impl Ord for Paket {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Paket::Integer(a), Paket::Integer(b)) => {
                // If both values are integers, the lower integer should come first.
                a.cmp(b)
            }
            (Paket::List(a), Paket::List(b)) => {
                // If both values are lists,

                for either_or_both in a.iter().zip_longest(b.iter()) {
                    match either_or_both {
                        // compare the first value of each list, then the second value, and so on
                        EitherOrBoth::Both(a, b) => {
                            let cmp = a.cmp(b);
                            if cmp != std::cmp::Ordering::Equal {
                                return cmp;
                            }
                        }
                        // If the right list runs out of items first, the inputs are not in the right order
                        EitherOrBoth::Left(_) => {
                            return std::cmp::Ordering::Greater;
                        }
                        // If the left list runs out of items first, the inputs are in the right order.
                        EitherOrBoth::Right(_) => {
                            return std::cmp::Ordering::Less;
                        }
                    }
                }
                std::cmp::Ordering::Equal
            }
            (i @ Paket::Integer(_), l @ Paket::List(_)) => Paket::List(vec![i.clone()]).cmp(l),
            (l @ Paket::List(_), i @ Paket::Integer(_)) => l.cmp(&Paket::List(vec![i.clone()])),
        }
    }
}

impl PartialOrd for Paket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct PaketPair {
    first: Paket,
    second: Paket,
}

impl PaketPair {
    fn parse(input: &str) -> IResult<&str, PaketPair> {
        let mut parser = map(
            tuple((Paket::parse, newline, Paket::parse)),
            |(first, _, second)| PaketPair { first, second },
        );
        parser(input)
    }

    fn right_order(&self) -> bool {
        self.first < self.second
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct PaketFile {
    pairs: Vec<PaketPair>,
}

impl PaketFile {
    fn parse(input: &str) -> IResult<&str, PaketFile> {
        let mut parser = terminated(
            map(
                separated_list0(pair(newline, newline), PaketPair::parse),
                |pairs| PaketFile { pairs },
            ),
            many0(newline),
        );
        parser(input)
    }

    fn indices_in_right_order(&self) -> Vec<i32> {
        self.pairs
            .iter()
            .enumerate()
            .filter_map(|(i, pair)| {
                if pair.right_order() {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                    Some(i as i32 + 1)
                } else {
                    None
                }
            })
            .collect()
    }

    fn part1(&self) -> i32 {
        self.indices_in_right_order().iter().sum()
    }

    fn all_pakets(&self) -> Vec<Paket> {
        self.pairs
            .iter()
            .flat_map(|pair| vec![pair.first.clone(), pair.second.clone()])
            .collect()
    }

    fn all_pakets_and_divider_sorted(&self) -> Vec<Paket> {
        let mut pakets = self.all_pakets();
        pakets.push(Paket::first_divider());
        pakets.push(Paket::second_divider());
        pakets.sort();
        pakets
    }

    fn part2(&self) -> usize {
        let pakets = self.all_pakets_and_divider_sorted();

        let first_divider_index = {
            let first_divider = Paket::first_divider();
            pakets.iter().position(|p| p == &first_divider).unwrap()
        };

        let second_divider_index = {
            let second_divider = Paket::second_divider();
            pakets.iter().position(|p| p == &second_divider).unwrap()
        };

        (first_divider_index + 1) * (second_divider_index + 1)
    }
}

pub fn day13() -> eyre::Result<()> {
    let input = include_str!("../data/day13.txt");
    let input = nom_finish(PaketFile::parse, input)?;
    {
        let start = Instant::now();
        let result = input.part1();

        let elapsed = start.elapsed();
        println!("Day 13.1: {result} ({elapsed:?})");
    }
    {
        let start = Instant::now();
        let result = input.part2();

        let elapsed = start.elapsed();
        println!("Day 13.2: {result} ({elapsed:?})");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::cmp::Ordering;

    static TEST_VECTOR: &str = r#"[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]"#;

    #[test]
    fn parse_paket() {
        let input = "[[1],2,3,[]]";
        let expected = Paket::List(vec![
            Paket::List(vec![Paket::Integer(1)]),
            Paket::Integer(2),
            Paket::Integer(3),
            Paket::List(vec![]),
        ]);

        assert_eq!(Paket::parse(input), Ok(("", expected)));
    }

    #[test]
    fn parse_paket_pair() {
        let input = r#"[[1],2,3,[]]
[[1],2,3,[]]"#;
        let expected_paket = Paket::List(vec![
            Paket::List(vec![Paket::Integer(1)]),
            Paket::Integer(2),
            Paket::Integer(3),
            Paket::List(vec![]),
        ]);

        let expected = PaketPair {
            first: expected_paket.clone(),
            second: expected_paket,
        };

        assert_eq!(PaketPair::parse(input), Ok(("", expected)));
    }

    #[test]
    fn parse_test_vector() {
        let (remaining, file) = PaketFile::parse(TEST_VECTOR).unwrap();

        assert_eq!(remaining, "");
        assert_eq!(file.pairs.len(), 8);
        assert_eq!(
            file.pairs[5],
            PaketPair {
                first: Paket::List(vec![]),
                second: Paket::List(vec!(Paket::Integer(3))),
            }
        );
    }

    #[test]
    fn ord() {
        // Both integer
        assert_eq!(Paket::Integer(42).cmp(&Paket::Integer(42)), Ordering::Equal);
        assert_eq!(Paket::Integer(41).cmp(&Paket::Integer(42)), Ordering::Less);
        assert_eq!(
            Paket::Integer(42).cmp(&Paket::Integer(41)),
            Ordering::Greater
        );

        // Both lists same size
        assert_eq!(
            Paket::List(vec![Paket::Integer(42)]).cmp(&Paket::List(vec![Paket::Integer(42)])),
            Ordering::Equal
        );
        assert_eq!(
            Paket::List(vec![Paket::Integer(41)]).cmp(&Paket::List(vec![Paket::Integer(42)])),
            Ordering::Less
        );
        assert_eq!(
            Paket::List(vec![Paket::Integer(42)]).cmp(&Paket::List(vec![Paket::Integer(41)])),
            Ordering::Greater
        );

        // Both lists different size (with diff)
        assert_eq!(
            Paket::List(vec![Paket::Integer(41)])
                .cmp(&Paket::List(vec![Paket::Integer(42), Paket::Integer(1)])),
            Ordering::Less
        );
        assert_eq!(
            Paket::List(vec![Paket::Integer(42)]).cmp(&Paket::List(vec![
                Paket::Integer(41),
                Paket::Integer(5_000)
            ])),
            Ordering::Greater
        );

        // Both lists different size (no diff)
        assert_eq!(
            Paket::List(vec![Paket::Integer(42)])
                .cmp(&Paket::List(vec![Paket::Integer(42), Paket::Integer(1)])),
            Ordering::Less
        );
        assert_eq!(
            Paket::List(vec![Paket::Integer(42), Paket::Integer(5_000)])
                .cmp(&Paket::List(vec![Paket::Integer(42),])),
            Ordering::Greater
        );

        // Integer and list
        // Both integer
        assert_eq!(
            Paket::List(vec![Paket::Integer(42)]).cmp(&Paket::Integer(42)),
            Ordering::Equal
        );
        assert_eq!(
            Paket::Integer(41).cmp(&Paket::List(vec![Paket::Integer(42)])),
            Ordering::Less
        );
        assert_eq!(
            Paket::List(vec![Paket::Integer(42)]).cmp(&Paket::Integer(41)),
            Ordering::Greater
        );
    }

    #[test]
    fn ord_manual_test_vector_pair1() {
        let left = Paket::List(vec![
            Paket::Integer(1),
            Paket::Integer(1),
            Paket::Integer(3),
            Paket::Integer(1),
            Paket::Integer(1),
        ]);
        let right = Paket::List(vec![
            Paket::Integer(1),
            Paket::Integer(1),
            Paket::Integer(5),
            Paket::Integer(1),
            Paket::Integer(1),
        ]);
        assert_eq!(left.cmp(&right), Ordering::Less);
    }

    #[test]
    fn ord_manual_test_vector_pair2() {
        let left = Paket::List(vec![
            Paket::List(vec![Paket::Integer(1)]),
            Paket::List(vec![
                Paket::Integer(2),
                Paket::Integer(3),
                Paket::Integer(4),
            ]),
        ]);
        let right = Paket::List(vec![
            Paket::List(vec![Paket::Integer(1)]),
            Paket::Integer(4),
        ]);
        assert_eq!(left.cmp(&right), Ordering::Less);
    }

    #[test]
    fn ord_test_vector() {
        let (_, file) = PaketFile::parse(TEST_VECTOR).unwrap();

        assert_eq!(file.pairs[0].right_order(), true);
        assert_eq!(file.pairs[1].right_order(), true);
        assert_eq!(file.pairs[2].right_order(), false);
        assert_eq!(file.pairs[3].right_order(), true);
        assert_eq!(file.pairs[4].right_order(), false);
        assert_eq!(file.pairs[5].right_order(), true);
        assert_eq!(file.pairs[6].right_order(), false);
        assert_eq!(file.pairs[7].right_order(), false);
    }

    #[test]
    fn part1() {
        let (_, file) = PaketFile::parse(TEST_VECTOR).unwrap();
        assert_eq!(file.part1(), 13);
    }

    #[test]
    fn part2() {
        let (_, file) = PaketFile::parse(TEST_VECTOR).unwrap();
        assert_eq!(file.part2(), 140);
    }
}
