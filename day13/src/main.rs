use anyhow::Result;
use num::Integer;

fn main() -> Result<()> {
    let mut lines = INPUT.lines();
    let earliest = lines.next().unwrap().parse::<i32>()?;
    let departures: Vec<_> = lines
        .next()
        .unwrap()
        .split(",")
        .map(|s| s.parse::<i32>().ok())
        .collect();
    let busses: Vec<_> = departures
        .iter()
        .filter(|r| r.is_some())
        .map(|opt| opt.iter().next().unwrap().clone())
        .collect();

    let (minutes, bus) = busses
        .iter()
        .map(|b| (b - earliest % b, b))
        .min_by_key(|(m, _)| *m)
        .unwrap();

    println!("Bus {} in {} min: {}", bus, minutes, bus * minutes);

    let vals: Vec<_> = departures
        .iter()
        .enumerate()
        .filter(|(_, opt)| opt.is_some())
        .map(|(i, opt)| {
            let id = opt.iter().next().unwrap().clone() as usize;
            (i, id)
        })
        .collect();

    let mut time: usize = 0;
    let mut increment: usize = 1;
    while !vals.iter().all(|(i, id)| (time + i) % id == 0) {
        for (i, id) in &vals {
            if (time + i) % id == 0 {
                increment = increment.lcm(id);
            }
        }
        time += increment;
    }

    println!("Sync {}", time);

    Ok(())
}

const INPUT: &str = r#"1000510
19,x,x,x,x,x,x,x,x,41,x,x,x,x,x,x,x,x,x,523,x,x,x,x,x,x,x,x,x,x,x,x,x,x,x,x,17,13,x,x,x,x,x,x,x,x,x,x,29,x,853,x,x,x,x,x,37,x,x,x,x,x,x,x,x,x,x,x,x,x,x,x,x,23
"#;
