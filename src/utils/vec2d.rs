use std::{
    iter::Flatten,
    ops::{Add, Div, Mul, Sub},
};

use eyre::bail;
use itertools::Itertools;
use scarlet::{
    colormap::{ColorMap, ListedColorMap},
    prelude::RGBColor,
};

use super::scale;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    pub fn map<U, F>(&self, op: F) -> Vec2D<U>
    where
        F: Fn(&T, usize, usize) -> U,
    {
        Vec2D {
            values: self
                .values
                .iter()
                .enumerate()
                .map(|(row, r)| {
                    r.iter()
                        .enumerate()
                        .map(|(col, val)| op(val, row, col))
                        .collect()
                })
                .collect(),
            rows: self.rows,
            cols: self.cols,
        }
    }

    pub fn iter(&self) -> Flatten<std::slice::Iter<'_, Vec<T>>> {
        self.values.iter().flatten()
    }

    pub fn paint<F>(&self, paint_one: F)
    where
        F: Fn(&T) -> String,
    {
        for row in 0..self.rows {
            for col in 0..self.cols {
                print!("{}", paint_one(&self.values[row][col]));
            }
            println!();
        }
    }

    pub fn paint_color(&self)
    where
        T: Sub<Output = T>
            + Mul<Output = T>
            + Div<Output = T>
            + Add<Output = T>
            + PartialOrd
            + Ord
            + Into<f64>
            + Copy,
    {
        self.paint_color_map(|x| *x, |_| '█'.to_string());
    }

    pub fn paint_color_map<U, FIntensity, FCharacter>(
        &self,
        intensity: FIntensity,
        character: FCharacter,
    ) where
        FIntensity: Fn(&T) -> U + Copy,
        FCharacter: Fn(&T) -> String,
        U: Sub<Output = U>
            + Mul<Output = U>
            + Div<Output = U>
            + Add<Output = U>
            + PartialOrd
            + Ord
            + Into<f64>
            + Copy,
    {
        let (min, max) = self.iter().map(intensity).minmax().into_option().unwrap();
        let viridis = ListedColorMap::viridis();

        self.paint(|h| {
            let scaled = scale(
                Into::<f64>::into(intensity(h)),
                Into::<f64>::into(min),
                Into::<f64>::into(max),
                0.0,
                1.0,
            );
            let colorpoint: RGBColor = viridis.transform_single(scaled);

            let color =
                yansi::Color::RGB(colorpoint.int_r(), colorpoint.int_g(), colorpoint.int_b());

            color.paint(character(h)).to_string()
        });
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
