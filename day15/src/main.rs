use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Game {
    history: HashMap<u64, usize>,
    turn: usize,
    last_spoken: u64,
}

impl Iterator for Game {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.turn += 1;
        let entry = self.history.entry(self.last_spoken);
        let last_time = entry.or_insert(self.turn);
        let ret = self.turn - *last_time;

        *last_time = self.turn;

        self.last_spoken = ret as u64;

        Some(ret)
    }
}

fn main() -> Result<()>{
    let input: Vec<u64> = INPUT.split(",")
        .map(|l| l.parse().unwrap())
        .collect();

    {
        // init
        let history: HashMap<u64, usize> = input.iter().enumerate().map(|(i, v)| (*v, i)).collect();
        let turn = history.len() - 2;

        let g = Game {
            history,
            turn,
            last_spoken: *input.last().unwrap()
        };

        let one: Vec<_> = g.take(2020 - input.len()).collect();

        let part1 = one.last().unwrap();

        println!("Part 1: {}", part1);
    }

    {
        // init
        let history: HashMap<u64, usize> = input.iter().enumerate().map(|(i, v)| (*v, i)).collect();
        let turn = history.len() - 2;

        let mut g = Game {
            history,
            turn,
            last_spoken: *input.last().unwrap()
        };

        let part2= g.nth(30000000 - input.len() - 1).unwrap();

        println!("Part 2: {}", part2);
    }
    Ok(())
}

const INPUT: &str = r#"7,14,0,17,11,1,2"#;
