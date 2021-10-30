use std::iter::FromIterator;
use std::ops::Add;
use anyhow::Result;
use std::str::FromStr;
use itertools::Itertools;

#[derive(Debug)]
struct HexVector {
    x: i32,
    y: i32,
    z: i32,
}

impl Add for HexVector {
    type Output = HexVector;

    fn add(self, rhs: Self) -> Self::Output {
        HexVector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl FromStr for HexVector {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pattern = regex::Regex::new(r"e|w|se|sw|ne|nw").unwrap();
        let mut ret = HexVector {
            x: 0,
            y: 0,
            z: 0
        };
        for cap in pattern.captures_iter(s) {
            let dir = match &cap[0] {
                "e" => HexVector {
                    x: 1,
                    y: -1,
                    z: 0
                },
                "w" => HexVector {
                    x: -1,
                    y: 1,
                    z: 0
                },
                "se" => HexVector {
                    x: 0,
                    y: -1,
                    z: 1
                },
                "sw" => HexVector {
                    x: -1,
                    y: 0,
                    z: 1
                },
                "ne" => HexVector {
                    x: 1,
                    y: 0,
                    z: -1
                },
                "nw" => HexVector {
                    x: 0,
                    y: 1,
                    z: -1
                },
                _ => panic!("bad direction")
            };

            ret = ret + dir;
        }
        Ok(ret)
    }
}

fn main() -> Result<()> {
    let input: Vec<HexVector> = INPUT.lines().map(|l| l.parse().unwrap()).collect();

    dbg!(input);
    Ok(())
}

const INPUT: &str = r#""#;
