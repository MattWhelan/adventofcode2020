use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Mul, Neg, Index, IndexMut};

use itertools::Itertools;
use std::str::FromStr;

/// A point in 2-space. Supports addition, scalar multiplication, manhattan_dist.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    T: TryInto<isize>,
    (T, T): Clone,
{
    fn from(p: &(T, T)) -> Self {
        Point::from(p.clone())
    }
}

impl<T> From<(T, T)> for Point
where
    T: TryInto<isize>,
{
    fn from(p: (T, T)) -> Self {
        if let Ok(x) = p.0.try_into() {
            if let Ok(y) = p.1.try_into() {
                return Point { x, y };
            }
        }
        panic!("Could not convert to Point")
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct DenseStore<Glyph> {
    data: Vec<Glyph>,
    width: usize,
}

impl<Glyph: Clone> DenseStore<Glyph> {
    pub fn new(grid: &Vec<Vec<Glyph>>) -> Self {
        if grid.is_empty() {
            DenseStore {
                data: Vec::new(),
                width: 0
            }
        } else {
            DenseStore {
                data: grid.iter().flatten().cloned().collect(),
                width: grid[0].len()
            }
        }
    }

    pub fn get(&self, p: Point) -> Option<&Glyph> {
        let Point { x, y} = p;
        let width = self.width as isize;
        if let Ok(i) = usize::try_from(x + y * width) {
            return self.data.get(i);
        }
        None
    }

    pub fn get_mut(&mut self, p: Point) -> Option<&mut Glyph> {
        let Point { x, y} = p;
        let width = self.width as isize;
        if let Ok(i) = usize::try_from(x + y * width) {
            return self.data.get_mut(i);
        }
        None
    }

    pub fn tiles<'a>(&'a self) -> impl Iterator<Item = (&'a Glyph, Point)> + 'a {
        let width = self.width;
        self.data.iter()
            .enumerate()
            .map(move |(i, g)| {
                let y = (i / width) as isize;
                let x = (i % width) as isize;
                let p = (x, y).into();
                (g, p)
            })
    }

    pub fn tiles_mut(&mut self) -> impl Iterator<Item = (&mut Glyph, Point)> + '_ {
        let width = self.width;
        self.data.iter_mut()
            .enumerate()
            .map(move |(i, g)| {
                let y = (i / width) as isize;
                let x = (i % width) as isize;
                let p = (x, y).into();
                (g, p)
            })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.data.len() / self.width
    }
}

impl<Glyph: Clone> Index<Point> for DenseStore<Glyph> {
    type Output = Glyph;

    fn index(&self, index: Point) -> &Self::Output {
        self.get(index).expect("No such point")
    }
}

impl<Glyph: Clone> IndexMut<Point> for DenseStore<Glyph> {
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        self.get_mut(index).expect("No such point")
    }
}

impl<Glyph> Display for DenseStore<Glyph>
    where
        Glyph: Clone,
        Glyph: Into<char>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let s = self
            .data.windows(self.width)
            .map(|row| {
                row.iter()
                    .map(|g| <Glyph as Into<char>>::into(g.clone()))
                    .collect::<String>()
            })
            .join("\n");
        write!(f, "{}", s)
    }
}

impl<Glyph> FromStr for DenseStore<Glyph>
where Glyph: Clone,
    char: Into<Glyph>,
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = parse_grid(s);
        Ok(DenseStore::new(&data))
    }
}

pub trait Grid {
    type Glyph;

    fn coord_transform(&self, p: Point) -> Point;

    fn data(&self) -> &Vec<Vec<Self::Glyph>>;

    fn data_mut(&mut self) -> &mut Vec<Vec<Self::Glyph>>;

    fn at(&self, p: Point) -> Option<&Self::Glyph> {
        let Point { x, y } = self.coord_transform(p);
        if let Ok(row) = usize::try_from(y) {
            if let Ok(col) = usize::try_from(x) {
                self.data()
                    .get(row)
                    .and_then(|row: &Vec<Self::Glyph>| row.get(col))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn at_mut(&mut self, p: Point) -> Option<&mut Self::Glyph> {
        let Point { x, y } = self.coord_transform(p);
        if let Ok(row) = usize::try_from(y) {
            if let Ok(col) = usize::try_from(x) {
                return Some(&mut self.data_mut()[row][col]);
            }
        }
        None
    }
}

pub fn parse_grid<Glyph>(glyph_str: &str) -> Vec<Vec<Glyph>>
where
    char: Into<Glyph>
{
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
    char: Into<Glyph>,
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

impl<Glyph> Grid for DenseGrid<Glyph> {
    type Glyph = Glyph;

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
    Glyph: Clone,
    Glyph: Into<char>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let s = self
            .data
            .iter()
            .map(|row| {
                row.iter()
                    .map(|g| <Glyph as Into<char>>::into(g.clone()))
                    .collect::<String>()
            })
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
