use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    time::Instant,
};

use eyre::eyre;
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, newline, not_line_ending},
    combinator::{map, map_res},
    multi::separated_list0,
    sequence::{preceded, separated_pair},
    IResult,
};

use crate::utils::DayParams;

#[derive(Debug, Clone, PartialEq, Eq)]
enum InputLine {
    Cd(String),
    Dir(String),
    File(String, usize),
    Ls,
}

fn parse_usize(input: &str) -> IResult<&str, usize> {
    map_res(digit1, str::parse)(input)
}

fn parse_input_line(input: &str) -> IResult<&str, InputLine> {
    let cd = map(preceded(tag("$ cd "), not_line_ending), |s: &str| {
        InputLine::Cd(s.to_string())
    });

    let file = map(
        separated_pair(parse_usize, tag(" "), not_line_ending),
        |(size, name)| InputLine::File(name.to_string(), size),
    );

    let dir = map(preceded(tag("dir "), not_line_ending), |s: &str| {
        InputLine::Dir(s.to_string())
    });

    let ls = map(tag("$ ls"), |_| InputLine::Ls);

    alt((cd, file, dir, ls))(input)
}

fn parse_input(input: &str) -> IResult<&str, Vec<InputLine>> {
    separated_list0(newline, parse_input_line)(input)
}

fn load_from_reader(reader: impl BufRead) -> eyre::Result<Vec<InputLine>> {
    let s = io::read_to_string(reader)?;
    let (_, input) = parse_input(&s).map_err(|e| eyre!(e.to_owned()))?;
    Ok(input)
}

fn load_from_file(path: impl AsRef<Path>) -> eyre::Result<Vec<InputLine>> {
    let file = File::open(path)?;
    load_from_reader(BufReader::new(file))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FsNode {
    Dir,
    File(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Fs(HashMap<PathBuf, FsNode>);

impl Fs {
    fn from_input(input: &[InputLine]) -> Self {
        let mut fs = HashMap::<PathBuf, FsNode>::new();
        let mut current_dir = PathBuf::new();

        // Root dir, always present
        fs.insert(current_dir.clone(), FsNode::Dir);

        for line in input {
            match line {
                InputLine::Cd(dir) => {
                    if dir.starts_with('/') {
                        current_dir = dir.trim_start_matches('/').split('/').collect();
                    } else if dir == ".." {
                        current_dir.pop();
                    } else {
                        current_dir.push(dir);
                    }
                    fs.insert(current_dir.clone(), FsNode::Dir);
                }
                InputLine::Dir(dir) => {
                    let dir = current_dir.join(dir);
                    fs.insert(dir, FsNode::Dir);
                }
                InputLine::File(name, size) => {
                    let file = current_dir.join(name);
                    fs.insert(file, FsNode::File(*size));
                }
                InputLine::Ls => {}
            }
        }

        Fs(fs)
    }

    #[allow(dead_code)]
    fn get(&self, path: &Path) -> Option<&FsNode> {
        self.0.get(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirSizes(HashMap<PathBuf, usize>);

impl DirSizes {
    pub fn from_fs(fs: &Fs) -> Self {
        let mut sizes = HashMap::new();

        for (path, node) in &fs.0 {
            match node {
                FsNode::Dir => {
                    sizes.insert(path.clone(), 0);
                }
                FsNode::File(_) => {}
            }
        }

        for (path, node) in &fs.0 {
            match node {
                FsNode::Dir => {}
                FsNode::File(file_size) => {
                    for (dir, dir_size) in &mut sizes {
                        if path.starts_with(dir) {
                            *dir_size += file_size;
                        }
                    }
                }
            }
        }

        DirSizes(sizes)
    }

    pub fn get(&self, path: &Path) -> Option<usize> {
        self.0.get(path).copied()
    }

    pub fn sum_smaller_than(&self, size: usize) -> usize {
        self.0
            .iter()
            .filter(|(_, &dir_size)| dir_size <= size)
            .map(|(_, &dir_size)| dir_size)
            .sum()
    }

    pub fn find_dir_to_delete(
        &self,
        disk_space: usize,
        required_free: usize,
    ) -> Option<(PathBuf, usize)> {
        let used = self.get(Path::new("")).unwrap_or(0);
        let free = disk_space - used;
        if free >= required_free {
            return None;
        }

        let to_delete = required_free - free;
        self.0
            .iter()
            .filter(|(_, &dir_size)| dir_size >= to_delete)
            .sorted_by_key(|(_, &dir_size)| dir_size)
            .next()
            .map(|(dir, size)| (dir.clone(), *size))
    }
}

pub fn day07(p: &DayParams) -> eyre::Result<()> {
    let text = load_from_file(p.input_path())?;
    let fs = Fs::from_input(&text);

    {
        let start = Instant::now();
        let sizes = DirSizes::from_fs(&fs);
        let result = sizes.sum_smaller_than(100_000);
        let elapsed = start.elapsed();
        println!("Day 7.1: {result} ({elapsed:?})",);
    }
    {
        let start = Instant::now();
        let sizes = DirSizes::from_fs(&fs);
        let (_, to_delete_size) = sizes
            .find_dir_to_delete(70_000_000, 30_000_000)
            .ok_or_else(|| eyre!("No dir to delete"))?;
        let elapsed = start.elapsed();
        println!("Day 7.2: {to_delete_size:?} ({elapsed:?})");
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    static TEST_VECTOR: &str = r#"$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k"#;

    #[test]
    fn parse_lines() {
        let lines = load_from_reader(TEST_VECTOR.as_bytes()).unwrap();
        assert_eq!(lines.len(), 23);
        assert_eq!(lines[0], InputLine::Cd("/".to_string()));
        assert_eq!(lines[1], InputLine::Ls);
        assert_eq!(lines[2], InputLine::Dir("a".to_string()));
        assert_eq!(
            lines[3],
            InputLine::File("b.txt".to_string(), 14_848_514_usize)
        );
    }

    #[test]
    fn fs() {
        let lines = load_from_reader(TEST_VECTOR.as_bytes()).unwrap();
        let fs = Fs::from_input(&lines);

        assert_eq!(fs.get(&PathBuf::from("")), Some(&FsNode::Dir));
        assert_eq!(fs.get(&PathBuf::from("a/e/")), Some(&FsNode::Dir));
        assert_eq!(fs.get(&PathBuf::from("a/e/i")), Some(&FsNode::File(584)));
    }

    #[test]
    fn sizes() {
        let lines = load_from_reader(TEST_VECTOR.as_bytes()).unwrap();
        let fs = Fs::from_input(&lines);
        let sizes = DirSizes::from_fs(&fs);

        assert_eq!(sizes.get(&PathBuf::from("a/e")), Some(584));
        assert_eq!(sizes.get(&PathBuf::from("a")), Some(94_853));
        assert_eq!(sizes.get(&PathBuf::from("d")), Some(24_933_642));
        assert_eq!(sizes.get(&PathBuf::from("")), Some(48_381_165));
    }

    #[test]
    fn sum() {
        let lines = load_from_reader(TEST_VECTOR.as_bytes()).unwrap();
        let fs = Fs::from_input(&lines);
        let sizes = DirSizes::from_fs(&fs);
        let sum = sizes.sum_smaller_than(100_000);
        assert_eq!(sum, 95_437);
    }

    #[test]
    fn to_delete() {
        let lines = load_from_reader(TEST_VECTOR.as_bytes()).unwrap();
        let fs = Fs::from_input(&lines);
        let sizes = DirSizes::from_fs(&fs);
        let to_delete = sizes.find_dir_to_delete(70_000_000, 30_000_000);
        assert_eq!(to_delete, Some((PathBuf::from("d"), 24_933_642)));
    }
}
