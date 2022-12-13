use eyre::eyre;
use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufReader},
    path::Path,
};

fn load_from_file(path: impl AsRef<Path>) -> io::Result<String> {
    let file = File::open(path)?;
    io::read_to_string(BufReader::new(file))
}

fn has_repetitions(s: &str) -> bool {
    s.chars().collect::<HashSet<_>>().len() != s.len()
}

fn find_marker(s: &str, len: usize) -> Option<usize> {
    if s.len() < len {
        return None;
    }

    for i in (len - 1)..s.len() {
        let start = i + 1 - len;
        let marker = &s[start..=i];

        if !has_repetitions(marker) {
            return Some(i + 1);
        }
    }

    None
}

pub fn day6() -> eyre::Result<()> {
    let text = load_from_file("data/day6.txt")?;

    {
        let marker = find_marker(&text, 4).ok_or_else(|| eyre!("No marker found"))?;
        println!("Day 6.1: {marker}");
    }
    {
        let marker = find_marker(&text, 14).ok_or_else(|| eyre!("No marker found"))?;
        println!("Day 6.2: {marker}");
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn part1_index(s: &str) -> usize {
        find_marker(s, 4).unwrap()
    }

    #[test]
    fn part_1() {
        assert_eq!(part1_index("mjqjpqmgbljsphdztnvjfqwrcgsmlb"), 7);
        assert_eq!(part1_index("bvwbjplbgvbhsrlpgdmjqwftvncz"), 5);
        assert_eq!(part1_index("nppdvjthqldpwncqszvftbrmjlhg"), 6);
        assert_eq!(part1_index("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), 10);
        assert_eq!(part1_index("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), 11);
    }

    fn part2_index(s: &str) -> usize {
        find_marker(s, 14).unwrap()
    }

    #[test]
    fn part_2() {
        assert_eq!(part2_index("mjqjpqmgbljsphdztnvjfqwrcgsmlb"), 19);
        assert_eq!(part2_index("bvwbjplbgvbhsrlpgdmjqwftvncz"), 23);
        assert_eq!(part2_index("nppdvjthqldpwncqszvftbrmjlhg"), 23);
        assert_eq!(part2_index("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), 29);
        assert_eq!(part2_index("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), 26);
    }
}
