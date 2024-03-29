use std::collections::VecDeque;
use std::iter::FromIterator;
use anyhow::Result;
use itertools::Itertools;

fn main() -> Result<()> {
    let input: Vec<u32> = INPUT.chars().map(|l| l.to_string().parse().unwrap()).collect();

    let result = crab_cups(&input, 100);

    dbg!(result[1..].iter().join(""));

    let mut big_input = Vec::new();
    big_input.push(0);
    big_input.extend((1..10).into_iter().map(|n| {
        let (i, _) = input.iter().enumerate().filter(|(_, &m)| m==n).next().unwrap();
        if i == 8 {
            10 as usize
        } else {
            input[i+1] as usize
        }
    }));
    big_input.extend(11..1_000_001);
    big_input.push(input[0] as usize);

    dbg!(big_input.len());

    let big_result = indexed_crab_cups(input[0] as usize, &mut big_input, 10_000_000);

    dbg!(big_result);
    Ok(())
}

fn indexed_crab_cups(start: usize, index: &mut[usize], rounds: usize) -> usize {
    let max = index.len()-1;

    let mut current = start;
    let mut selected = Vec::new();

    for _ in 0..rounds {
        selected.push(index[current]);
        selected.push(index[index[current]]);
        selected.push(index[index[index[current]]]);

        let mut target = if current == 1 {
            max
        } else {
            current - 1
        };
        while selected.contains(&target) {
            target = if target == 1 {
                max
            } else {
                target - 1
            };
        }

        //Move the selected elements to after the target
        // Remove the selected elements from after the current element
        index[current] = index[selected[2]];
        // splice the selected element in before the target's next
        index[selected[2]] = index[target];
        // point the target at the beginning of the selection
        index[target] = selected[0];

        selected.clear();

        current = index[current];
    }

    index[1] * index[index[1]]
}

fn crab_cups(seq: &[u32], rounds: usize) -> Vec<u32>{
    let count = seq.len() as u32;
    let mut buf = VecDeque::from_iter(seq.iter().copied());

    let mut selected = Vec::new();

    for _ in 0..rounds {
        let current = buf.front().unwrap().to_owned();
        buf.rotate_left(1);

        selected.push(buf.pop_front().unwrap().to_owned());
        selected.push(buf.pop_front().unwrap().to_owned());
        selected.push(buf.pop_front().unwrap().to_owned());

        let mut target = if current == 1 {
            count
        } else {
            current - 1
        };
        while selected.contains(&target) {
            target = if target == 1 {
                count
            } else {
                target - 1
            };
        }

        while buf.front().unwrap() != &target {
            buf.rotate_right(1);
        }
        buf.rotate_left(1);
        buf.push_front(selected.pop().unwrap());
        buf.push_front(selected.pop().unwrap());
        buf.push_front(selected.pop().unwrap());

        while buf.back().unwrap() != &current {
            buf.rotate_right(1);
        }
    }

    while buf.front().unwrap().to_owned() != 1 {
        buf.rotate_left(1);
    }

    buf.into()
}

const INPUT: &str = r#"198753462"#;
const TEST: &str = r#"389125467"#;
