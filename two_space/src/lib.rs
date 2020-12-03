use std::convert::TryInto;
use std::ops::{Add, Mul, Neg, Sub};

/// A point in 2-space. Supports addition, scalar multiplication, manhattan_dist.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T>
where
    T: Add<Output = T>,
    T: Sub<Output = T>,
    T: Copy,
    T: Neg<Output = T>,
    T: Default,
    T: std::cmp::PartialOrd,
{
    pub fn manhattan_dist(&self, rhs: &Self) -> T {
        let mut d0 = self.x - rhs.x;
        if d0 < Default::default() {
            d0 = -d0;
        }

        let mut d1 = self.y - rhs.y;
        if d1 < Default::default() {
            d1 = -d1;
        }

        d0 + d1
    }
}

impl<T: Add<Output = T>> Add for Point<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Mul<T> for Point<T>
where
    T: Mul<Output = T>,
    T: Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> Neg for Point<T>
where
    T: Neg<Output = T>,
    T: Copy,
{
    type Output = Point<T>;

    fn neg(self) -> Self::Output {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> From<(T, T)> for Point<T> {
    fn from(p: (T, T)) -> Self {
        Point { x: p.0, y: p.1 }
    }
}

pub trait Grid<Glyph, T>
where
    T: TryInto<usize>,
    Glyph: Copy,
{
    fn coord_transform(&self, p: Point<T>) -> Point<T>;

    fn data(&self) -> &Vec<Vec<Glyph>>;

    fn at(&self, p: Point<T>) -> Option<Glyph> {
        let Point { x, y } = self.coord_transform(p);
        if let Ok(row) = y.try_into() {
            if let Ok(col) = x.try_into() {
                self.data().get(row).and_then(|row| row.get(col).map(|g| *g))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Dense Grid
pub struct DenseGrid<Glyph, T>
where
    T: TryInto<usize>,
{
    data: Vec<Vec<Glyph>>,
    offset: Point<T>,
}

impl<Glyph, T> DenseGrid<Glyph, T>
where
    T: TryInto<usize>,
    T: Default,
    Point<T>: Neg<Output = Point<T>>,
    Glyph: From<char>,
{
    pub fn new(glyph_str: &str) -> Self {
        let data = glyph_str
            .lines()
            .map(|l| l.chars().map(|c| c.into()).collect())
            .collect();

        let offset: Point<T> = Default::default();

        DenseGrid {
            data,
            offset,
        }
    }

    pub fn with_offset(mut self, origin: Point<T>) -> Self {
        self.offset = -origin;
        self
    }
}

impl<Glyph, T> Grid<Glyph, T> for DenseGrid<Glyph, T>
where
    T: TryInto<usize>,
    Point<T>: Add<Output = Point<T>>,
    Point<T>: Copy,
    Glyph: Copy,
{
    fn coord_transform(&self, p: Point<T>) -> Point<T> {
        p + self.offset
    }

    fn data(&self) -> &Vec<Vec<Glyph>> {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use crate::Point;

    #[test]
    fn add_test() {
        assert_eq!(
            Point { x: 2, y: 2 } + Point { x: 3, y: 4 },
            Point { x: 5, y: 6 }
        );
    }

    #[test]
    fn mul_test() {
        assert_eq!(Point { x: 3, y: 4 } * 5, Point { x: 15, y: 20 });
    }

    #[test]
    fn into_test() {
        let p1: Point<i32> = (2, 2).into();
        let p2: Point<i32> = (3, 4).into();
        assert_eq!(p1 + p2, (5, 6).into());
    }

    #[test]
    fn manhattan_dist_test() {
        let p0: Point<i32> = (0, 0).into();
        let p1: Point<i32> = (1, 4).into();

        assert_eq!(p0.manhattan_dist(&p1), 5);
        assert_eq!(p1.manhattan_dist(&p0), 5);
    }
}
