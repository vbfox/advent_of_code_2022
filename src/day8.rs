use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

use eyre::bail;
use eyre::eyre;

use crate::utils::CharSliceExt;

struct Tree {
    height: u32,
}

impl FromStr for Tree {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let height = s.parse()?;
        Ok(Tree { height })
    }
}

struct Forest {
    trees: Vec<Vec<Tree>>,
}

impl Forest {
    fn new() -> Self {
        Forest { trees: Vec::new() }
    }

    fn rows(&self) -> usize {
        self.trees.len()
    }

    fn cols(&self) -> usize {
        if self.rows() > 0 {
            self.trees[0].len()
        } else {
            0
        }
    }

    fn add_row(&mut self, row: Vec<Tree>) -> eyre::Result<()> {
        if self.rows() > 0 && row.len() != self.cols() {
            bail!("Row length mismatch");
        }

        self.trees.push(row);
        Ok(())
    }

    fn get(&self, row: usize, col: usize) -> Option<&Tree> {
        self.trees.get(row).and_then(|r| r.get(col))
    }
}

impl Display for Forest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.trees {
            for tree in row {
                write!(f, "{}", tree.height)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl FromStr for Forest {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut forest = Forest::new();
        for line in s.lines() {
            let row: Vec<Tree> = line.char_slices().map(|s| s.parse().unwrap()).collect();
            forest.add_row(row)?;
        }
        Ok(forest)
    }
}

struct TreeVisibility {
    is_visible: Vec<Vec<bool>>,
}

impl TreeVisibility {
    fn new(rows: usize, cols: usize, value: bool) -> Self {
        TreeVisibility {
            is_visible: vec![vec![value; cols]; rows],
        }
    }

    fn rows(&self) -> usize {
        self.is_visible.len()
    }

    fn cols(&self) -> usize {
        if self.rows() > 0 {
            self.is_visible[0].len()
        } else {
            0
        }
    }

    #[allow(dead_code)]
    fn get(&self, row: usize, col: usize) -> Option<bool> {
        self.is_visible.get(row).and_then(|r| r.get(col)).cloned()
    }

    fn op(&mut self, other: &Self, op: fn(bool, bool) -> bool) -> eyre::Result<()> {
        let rows = self.rows();
        let cols = self.cols();
        if rows != other.rows() || cols != other.cols() {
            bail!(
                "Dimension mismatch: {}x{} vs {}x{}",
                rows,
                cols,
                other.rows(),
                other.cols()
            );
        }

        for row in 0..rows {
            for col in 0..cols {
                self.is_visible[row][col] =
                    op(self.is_visible[row][col], other.is_visible[row][col]);
            }
        }

        Ok(())
    }

    fn compute_directional(
        forest: &Forest,
        direction_row: i64,
        direction_col: i64,
    ) -> eyre::Result<Self> {
        let rows = forest.rows();
        let cols = forest.cols();
        let mut visibility = Self::new(rows, cols, true);

        for row in 1..(rows - 1) {
            for col in 1..(cols - 1) {
                let size = forest.get(row, col).unwrap().height;
                let mut check_row = (row as i64) + direction_row;
                let mut check_col = (col as i64) + direction_col;
                while check_col >= 0
                    && check_row >= 0
                    && check_row < rows as i64
                    && check_col < cols as i64
                {
                    let size_at_check = forest
                        .get(check_row as usize, check_col as usize)
                        .ok_or_else(|| eyre!("Invalid index: {}x{}", check_row, check_col))?
                        .height;

                    if size_at_check >= size {
                        visibility.is_visible[row][col] = false;
                        break;
                    }

                    check_row += direction_row;
                    check_col += direction_col;
                }
            }
        }

        Ok(visibility)
    }

    pub fn compute(forest: &Forest) -> eyre::Result<Self> {
        let mut result = Self::compute_directional(forest, 0, 1)?;
        result.op(&Self::compute_directional(forest, 0, -1)?, |a, b| a || b)?;
        result.op(&Self::compute_directional(forest, 1, 0)?, |a, b| a || b)?;
        result.op(&Self::compute_directional(forest, -1, 0)?, |a, b| a || b)?;
        Ok(result)
    }

    pub fn count_visible(&self) -> usize {
        self.is_visible.iter().flatten().filter(|&&v| v).count()
    }
}

impl Display for TreeVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.is_visible {
            for is_visible in row {
                if *is_visible {
                    write!(f, "X")?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub fn day8() -> eyre::Result<()> {
    let forest: Forest = include_str!("../data/day8.txt").parse()?;
    {
        let visibility = TreeVisibility::compute(&forest)?;
        println!("Day 8.2: {}", visibility.count_visible());
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use super::*;

    static TEST_VECTOR: &str = r#"30373
25512
65332
33549
35390"#;

    #[test]
    fn parse() {
        let forest: Forest = TEST_VECTOR.parse().unwrap();
        assert_eq!(forest.rows(), 5);
        assert_eq!(forest.cols(), 5);
        assert_eq!(forest.get(0, 0).unwrap().height, 3);
        assert_eq!(forest.get(1, 3).unwrap().height, 1);
    }

    #[test]
    fn visibility() {
        let forest: Forest = TEST_VECTOR.parse().unwrap();
        let vis = TreeVisibility::compute(&forest).unwrap();

        assert_eq!(vis.get(0, 0).unwrap(), true);
        assert_eq!(vis.get(0, 1).unwrap(), true);
        assert_eq!(vis.get(0, 2).unwrap(), true);
        assert_eq!(vis.get(0, 3).unwrap(), true);
        assert_eq!(vis.get(0, 4).unwrap(), true);

        assert_eq!(vis.get(1, 0).unwrap(), true);
        assert_eq!(vis.get(1, 1).unwrap(), true);
        assert_eq!(vis.get(1, 2).unwrap(), true);
        assert_eq!(vis.get(1, 3).unwrap(), false);
        assert_eq!(vis.get(1, 4).unwrap(), true);

        assert_eq!(vis.get(2, 0).unwrap(), true);
        assert_eq!(vis.get(2, 1).unwrap(), true);
        assert_eq!(vis.get(2, 2).unwrap(), false);
        assert_eq!(vis.get(2, 3).unwrap(), true);
        assert_eq!(vis.get(2, 4).unwrap(), true);

        assert_eq!(vis.get(3, 0).unwrap(), true);
        assert_eq!(vis.get(3, 1).unwrap(), false);
        assert_eq!(vis.get(3, 2).unwrap(), true);
        assert_eq!(vis.get(3, 3).unwrap(), false);
        assert_eq!(vis.get(3, 4).unwrap(), true);

        assert_eq!(vis.get(4, 0).unwrap(), true);
        assert_eq!(vis.get(4, 1).unwrap(), true);
        assert_eq!(vis.get(4, 2).unwrap(), true);
        assert_eq!(vis.get(4, 3).unwrap(), true);
        assert_eq!(vis.get(4, 4).unwrap(), true);
    }

    #[test]
    fn count_visibility() {
        let forest: Forest = TEST_VECTOR.parse().unwrap();
        let vis = TreeVisibility::compute(&forest).unwrap();
        let count = vis.count_visible();
        assert_eq!(count, 21);
    }
}
