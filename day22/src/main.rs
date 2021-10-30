use std::collections::{HashSet, VecDeque};
use std::iter::FromIterator;
use anyhow::Result;
use std::str::FromStr;
use itertools::Itertools;

#[derive(Debug)]
struct Record;

impl FromStr for Record {
    type Err = anyhow::Error;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Record)
    }
}

fn main() -> Result<()> {
    let player1: Vec<usize> = PLAYER_1.lines().map(|l| l.parse().unwrap()).collect();
    let player2: Vec<usize> = PLAYER_2.lines().map(|l| l.parse().unwrap()).collect();

    dbg!(&player1, &player2);

    let (p1_win, deck) = combat(&player1, &player2);
    dbg!(p1_win);

    let winner_score = score(&deck);
    dbg!(winner_score);

    let (p1_win, recursive_deck) = recursive_combat(&player1, &player2);

    dbg!(p1_win, score(&recursive_deck));

    Ok(())
}

fn recursive_combat(p1: &[usize], p2: &[usize]) -> (bool, Vec<usize>) {
    let mut d1 = VecDeque::from_iter(p1.iter().copied());
    let mut d2 = VecDeque::from_iter(p2.iter().copied());

    let mut memory: HashSet<String> = HashSet::new();

    while d1.len() != 0 && d2.len() != 0 {
        let config = format!("{}p{}",
            d1.iter().map(|n|n.to_string()).join(","),
            d2.iter().map(|n|n.to_string()).join(","),
        );
        if memory.contains(&config) {
            return (true, d1.into())
        }
        memory.insert(config);

        let c1 = d1.pop_front().unwrap();
        let c2 = d2.pop_front().unwrap();

        let p1_win = if c1 <= d1.len() && c2 <= d2.len() {
            let (p1_win, _deck) = recursive_combat(
                &d1.make_contiguous()[0..c1],
                &d2.make_contiguous()[0..c2]
            );
            p1_win
        } else {
            c1 > c2
        };
        if p1_win {
            d1.push_back(c1);
            d1.push_back(c2);
        } else {
            d2.push_back(c2);
            d2.push_back(c1);
        }
    }

    if d1.len() == 0 {
        (false, d2.into())
    } else {
        (true, d1.into())
    }
}

fn score(d: &[usize]) -> usize {
    let len = d.len();
    d.iter().enumerate()
        .map(|(i, v)| (len-i) * v)
        .sum()

}

fn combat(p1: &[usize], p2: &[usize]) -> (bool, Vec<usize>) {
    let mut d1 = VecDeque::from_iter(p1.iter().copied());
    let mut d2 = VecDeque::from_iter(p2.iter().copied());

    while d1.len() != 0 && d2.len() != 0 {
        let c1 = d1.pop_front().unwrap();
        let c2 = d2.pop_front().unwrap();
        if c1 > c2 {
            d1.push_back(c1);
            d1.push_back(c2);
        } else {
            d2.push_back(c2);
            d2.push_back(c1);
        }
    }

    if d1.len() == 0 {
        (false, d2.into())
    } else {
        (true, d1.into())
    }
}

const PLAYER_1: &str = r#"17
19
30
45
25
48
8
6
39
36
28
5
47
26
46
20
18
13
7
49
34
23
43
22
4
"#;

const PLAYER_2: &str = r#"44
10
27
9
14
15
24
16
3
33
21
29
11
38
1
31
50
41
40
32
42
35
37
2
12
"#;
