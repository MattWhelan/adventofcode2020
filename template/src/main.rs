use anyhow::Result;
use std::str::FromStr;

#[derive(Debug)]
struct Record;

impl FromStr for Record {
    type Err = anyhow::Error;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Record)
    }
}

fn main() -> Result<()> {
    let input: Vec<Record> = INPUT.lines().map(|l| l.parse().unwrap()).collect();

    dbg!(input);
    Ok(())
}

const INPUT: &str = r#""#;
