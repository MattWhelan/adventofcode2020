use std::str::FromStr;
use thiserror::Error;
use anyhow::Result;

const INPUT: &str = r#""#;

#[derive(Error, Debug)]
enum ParseError {
}

#[derive(Debug)]
struct Record;

impl FromStr for Record {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Record)
    }
}

fn main() {
    let input: Vec<Record> = INPUT.lines()
        .map(|l| l.parse().unwrap())
        .collect::<Vec<_>>();

    dbg!(input);
}
