use anyhow::Result;

fn main() -> Result<()> {
    // let door_key = 5764801;
    // let card_key = 17807724;
    //
    let door_key = 11562782;
    let card_key = 18108497;

    let mut door_loop = 0;
    let mut card_loop = 0;

    let mut found = 0;
    let mut v = 1;
    for loop_size in 1.. {
        v = round(v, 7);
        if v == door_key {
            println!("door loop {}", loop_size);
            door_loop = loop_size;
            found += 1
        } else if v == card_key {
            println!("card loop {}", loop_size);
            card_loop = loop_size;
            found += 1
        }
        if found == 2 {
            break
        }
    }

    dbg!(door_loop, card_loop);

    v = 1;
    for _ in 0..card_loop {
        v = round(v, door_key);
    }
    dbg!(v);

    Ok(())
}

fn round(n: isize, sn: isize) -> isize {
    (n * sn) % 20201227
}
