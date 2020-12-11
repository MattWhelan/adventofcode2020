use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Mul, Neg};

use itertools::Itertools;

/// A point in 2-space. Supports addition, scalar multiplication, manhattan_dist.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

impl Point {
    pub fn manhattan_dist(&self, rhs: &Self) -> isize {
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

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul<isize> for Point {
    type Output = Self;

    fn mul(self, rhs: isize) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Neg for Point {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> From<&(T, T)> for Point
where
    isize: TryFrom<T>,
    (T, T): Clone,
{
    fn from(p: &(T, T)) -> Self {
        Point::from(p.clone())
    }
}

impl<T> From<(T, T)> for Point
where
    isize: TryFrom<T>,
{
    fn from(p: (T, T)) -> Self {
        if let Ok(x) = isize::try_from(p.0) {
            if let Ok(y) = isize::try_from(p.1) {
                return Point { x, y };
            }
        }
        panic!("Could not convert to Point")
    }
}

pub trait Grid<Glyph>
where
    Glyph: Copy,
{
    fn coord_transform(&self, p: Point) -> Point;

    fn data(&self) -> &Vec<Vec<Glyph>>;

    fn data_mut(&mut self) -> &mut Vec<Vec<Glyph>>;

    fn at(&self, p: Point) -> Option<Glyph> {
        let Point { x, y } = self.coord_transform(p);
        if let Ok(row) = usize::try_from(y) {
            if let Ok(col) = usize::try_from(x) {
                self.data()
                    .get(row)
                    .and_then(|row: &Vec<Glyph>| row.get(col).map(|g| *g))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn set_at(&mut self, p: Point, g: Glyph) {
        let Point { x, y } = self.coord_transform(p);
        if let Ok(row) = usize::try_from(y) {
            if let Ok(col) = usize::try_from(x) {
                self.data_mut()[row][col] = g;
            }
        }
    }
}

pub fn parse_grid<Glyph: From<char>>(glyph_str: &str) -> Vec<Vec<Glyph>> {
    glyph_str
        .lines()
        .map(|l| l.chars().map(|c| c.into()).collect())
        .collect()
}

/// Dense Grid
#[derive(Clone, Eq, PartialEq)]
pub struct DenseGrid<Glyph> {
    data: Vec<Vec<Glyph>>,
    offset: Point,
}

impl<Glyph> DenseGrid<Glyph>
where
    Glyph: From<char>,
    Glyph: Clone,
{
    pub fn new(src: &str) -> Self {
        DenseGrid {
            data: parse_grid(src),
            offset: Default::default(),
        }
    }

    pub fn with_offset(mut self, origin: Point) -> Self {
        self.offset = -origin;
        self
    }

    pub fn enumerate_tiles(&self) -> impl Iterator<Item = (Glyph, Point)> + '_ {
        self.data.iter().enumerate().flat_map(|(y, src_row)| {
            src_row.iter().enumerate().map(move |(x, g)| {
                let p = (x, y).into();
                (g.clone(), p)
            })
        })
    }

    pub fn transform<F: FnMut(&Glyph, Point) -> Glyph>(&self, mut tile_mapper: F) -> Self {
        let data = self
            .data
            .iter()
            .enumerate()
            .map(|(y, src_row)| {
                let mut row = Vec::with_capacity(self.data[y as usize].len());
                row.extend(src_row.iter().enumerate().map(|(x, g)| {
                    let p = (x, y).into();
                    tile_mapper(g, p)
                }));
                row
            })
            .collect();
        Self {
            data,
            offset: self.offset,
        }
    }
}

impl<Glyph> Grid<Glyph> for DenseGrid<Glyph>
where
    Glyph: Copy,
{
    fn coord_transform(&self, p: Point) -> Point {
        p + self.offset
    }

    fn data(&self) -> &Vec<Vec<Glyph>> {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Vec<Vec<Glyph>> {
        &mut self.data
    }
}

impl<Glyph> Display for DenseGrid<Glyph>
where
    Glyph: Copy,
    char: From<Glyph>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let s = self
            .data
            .iter()
            .map(|row| row.iter().map(|&g| char::from(g)).collect::<String>())
            .join("\n");
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_grid, Point};

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
        let p1: Point = (2, 2).into();
        let p2: Point = (3, 4).into();
        assert_eq!(p1 + p2, (5, 6).into());
    }

    #[test]
    fn manhattan_dist_test() {
        let p0: Point = (0, 0).into();
        let p1: Point = (1, 4).into();

        assert_eq!(p0.manhattan_dist(&p1), 5);
        assert_eq!(p1.manhattan_dist(&p0), 5);
    }

    #[test]
    fn parse_grid_test() {
        #[derive(Debug, Eq, PartialEq)]
        enum Tiles {
            Space,
            Tree,
        }

        impl From<char> for Tiles {
            fn from(ch: char) -> Self {
                match ch {
                    '#' => Tiles::Tree,
                    '.' => Tiles::Space,
                    _ => panic!("Invalid character"),
                }
            }
        }

        let in_str = "..#\n.#.";

        let data = parse_grid::<Tiles>(in_str);
        assert_eq!(data[0][2], Tiles::Tree);
        assert_eq!(data[1][1], Tiles::Tree);
        assert_eq!(data[0][0], Tiles::Space);
    }
}
