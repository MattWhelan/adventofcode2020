use anyhow::Result;
use std::str::FromStr;
use std::ops::{Add};
use std::collections::{HashMap, HashSet};
use crate::State::Inactive;
use State::Active;
use std::fmt::Debug;


#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Default)]
struct Point {
    x: i32,
    y: i32,
    z: i32,
}

impl<T> From<(T, T, T)> for Point
where
    T: Into<i32>
{
    fn from(p: (T, T, T)) -> Self {
        Self {
            x: p.0.into(),
            y: p.1.into(),
            z: p.2.into(),
        }
    }
}


impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Point {
    fn neighbors(&self) -> impl Iterator<Item=Self> {
        let mut ret: Vec<Point> = Vec::with_capacity(26);
        for i in -1..2 {
            for j in -1..2 {
                for k in -1..2 {
                    if i != 0 || j != 0 || k != 0 {
                        ret.push(*self + (i, j, k).into())
                    }
                }
            }
        }

        ret.into_iter()
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum State {
    Active, Inactive
}

impl From<char> for State {
    fn from(ch: char) -> Self {
        match ch {
            '#' => Active,
            '.' => Inactive,
            _ => panic!("Invalid state glyph")
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Space {
    state: HashMap<Point, State>
}

impl Space {
    fn cycle(&self) -> Self {
        let mut points: HashSet<Point> = self.state.keys().copied().collect();
        points.extend(self.state.keys()
            .flat_map(|k| k.neighbors()));

        let state = points.iter()
            .map(|p| {
                let active_neighbors = p.neighbors()
                    .filter(|p| self.state.get(p).cloned().unwrap_or(Inactive) == Active)
                    .count();
                let s = match self.state.get(p).unwrap_or(&Inactive) {
                    Active => if active_neighbors == 2 || active_neighbors == 3 {
                        Active
                    } else {
                        Inactive
                    }
                    Inactive => if active_neighbors == 3 {
                        Active
                    } else {
                        Inactive
                    }
                };

                (*p, s)
            })
            .collect();

        Self {
            state
        }
    }
}

impl FromStr for Space {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let state = s.lines()
            .enumerate()
            .flat_map(|(row_index, line)| {
                line.chars()
                    .map(|ch| ch.into())
                    .enumerate()
                    .map(move |(col_index, state)| ((col_index as i32, -(row_index as i32), 0).into(), state))
            })
            .collect();

        Ok(Space{state})
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////


#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Default)]
struct Point4 {
    x: i32,
    y: i32,
    z: i32,
    w: i32,
}

impl<T> From<(T, T, T, T)> for Point4
where
    T: Into<i32>
{
    fn from(p: (T, T, T, T)) -> Self {
        Self {
            x: p.0.into(),
            y: p.1.into(),
            z: p.2.into(),
            w: p.3.into(),
        }
    }
}


impl Add for Point4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl Point4 {
    fn neighbors(&self) -> impl Iterator<Item=Self> {
        let mut ret: Vec<Self> = Vec::with_capacity(26);
        for i in -1..2 {
            for j in -1..2 {
                for k in -1..2 {
                    for h in -1..2 {
                        if i != 0 || j != 0 || k != 0 || h != 0 {
                            ret.push(*self + (i, j, k, h).into())
                        }
                    }
                }
            }
        }

        ret.into_iter()
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct HyperSpace {
    state: HashMap<Point4, State>
}

impl HyperSpace {
    fn cycle(&self) -> Self {
        let mut points: HashSet<Point4> = self.state.keys().copied().collect();
        points.extend(self.state.keys()
            .flat_map(|k| k.neighbors()));

        let state = points.iter()
            .map(|p| {
                let active_neighbors = p.neighbors()
                    .filter(|p| self.state.get(p).cloned().unwrap_or(Inactive) == Active)
                    .count();
                let s = match self.state.get(p).unwrap_or(&Inactive) {
                    Active => if active_neighbors == 2 || active_neighbors == 3 {
                        Active
                    } else {
                        Inactive
                    }
                    Inactive => if active_neighbors == 3 {
                        Active
                    } else {
                        Inactive
                    }
                };

                (*p, s)
            })
            .collect();

        Self {
            state
        }
    }
}

impl FromStr for HyperSpace {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let state = s.lines()
            .enumerate()
            .flat_map(|(row_index, line)| {
                line.chars()
                    .map(|ch| ch.into())
                    .enumerate()
                    .map(move |(col_index, state)| ((col_index as i32, -(row_index as i32), 0, 0).into(), state))
            })
            .collect();

        Ok(HyperSpace{state})
    }
}

fn main() -> Result<()> {
    {
        let input: Space = INPUT.parse().expect("Parse failed");
        let mut last = input.clone();
        for _ in 0..6 {
            last = last.cycle()
        }
        let active_count = last.state.values().filter(|s| **s == Active).count();

        println!("Part 1 {}", active_count);
    }

    {
        let input: HyperSpace = INPUT.parse().expect("Parse failed");
        let mut last = input.clone();
        for _ in 0..6 {
            last = last.cycle()
        }
        let active_count = last.state.values().filter(|s| **s == Active).count();

        println!("Part 2 {}", active_count);
    }
    Ok(())
}

const INPUT: &str = r#"#.#.#.##
.####..#
#####.#.
#####..#
#....###
###...##
...#.#.#
#.##..##
"#;
