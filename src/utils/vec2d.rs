use std::iter::Flatten;

use eyre::bail;

pub struct Vec2D<T> {
    pub values: Vec<Vec<T>>,
    pub rows: usize,
    pub cols: usize,
}

impl<T: std::clone::Clone> Vec2D<T> {
    pub fn new(rows: usize, cols: usize, value: T) -> Self {
        Vec2D {
            values: vec![vec![value; cols]; rows],
            rows,
            cols,
        }
    }
}

impl<T> Vec2D<T> {
    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        self.values.get(row).and_then(|r| r.get(col))
    }

    pub fn set(&mut self, row: usize, col: usize, value: T) -> Option<()> {
        let row_vec = self.values.get_mut(row)?;
        let cell = row_vec.get_mut(col)?;
        *cell = value;
        Some(())
    }

    pub fn op(&mut self, other: &Self, op: fn(&T, &T) -> T) -> eyre::Result<()> {
        if self.rows != other.rows || self.cols != other.cols {
            bail!(
                "Dimension mismatch: {}x{} vs {}x{}",
                self.rows,
                self.cols,
                other.rows,
                other.cols
            );
        }

        for row in 0..self.rows {
            for col in 0..self.cols {
                self.values[row][col] = op(&self.values[row][col], &other.values[row][col]);
            }
        }

        Ok(())
    }

    pub fn iter(&self) -> Flatten<std::slice::Iter<'_, Vec<T>>> {
        self.values.iter().flatten()
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn get_set() {
        let mut vec2d = Vec2D::new(3, 3, 0);
        assert_eq!(vec2d.get(1, 1), Some(&0));
        vec2d.set(1, 1, 1).unwrap();
        assert_eq!(vec2d.get(1, 1), Some(&1));
        assert_eq!(vec2d.get(3, 3), None);
    }

    #[test]
    fn iter() {
        let mut vec2d = Vec2D::new(3, 3, 0);
        vec2d.set(1, 1, 1).unwrap();
        let values = vec2d.iter().collect_vec();
        assert_eq!(values, vec![&0, &0, &0, &0, &1, &0, &0, &0, &0]);
    }
}
