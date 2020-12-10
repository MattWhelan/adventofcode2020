use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;


fn main() -> Result<()>{
    let orig_input: Vec<i32> = INPUT.lines()
        .map(|l| l.parse().unwrap())
        .sorted()
        .collect();

    let mut input = orig_input.clone();
    input.insert(0, 0);
    let device = orig_input.iter().max().unwrap() + 3;
    input.push(device);
    //dbg!(&input);

    let counts : HashMap<i32, i32> = input.windows(2)
        .map(|w| w[1] - w[0])
        .fold(HashMap::new(), |mut m, d| {
            *(m.entry(d).or_default()) += 1;
            m
        });

    //dbg!(&counts);

    println!("part1 {}", counts[&1] * counts[&3]);

    let arrangements = count_arrangements(0, &orig_input, device);
    println!("Arrangements {}", arrangements);
    // too high 4398046511104
    Ok(())
}

fn count_arrangements(prefix: i32, nums: &[i32], suffix: i32) -> i128 {
    match nums.len() {
        0 => 1,
        1 => {
            if suffix - prefix <= 3 {
                2
            } else {
                1
            }
        },
        _ => {
            // len >= 2, so pivot is at least 1, and less than len.
            let pivot = nums.len() / 2;
            let left_with = count_arrangements(prefix, &nums[..pivot], nums[pivot]);
            let right_with = count_arrangements(nums[pivot], &nums[pivot+1..], suffix);
            let with = left_with * right_with;

            let prev = if pivot > 0 {
                nums[pivot-1]
            } else {
                prefix
            };
            let next = if pivot + 1 < nums.len() {
                nums[pivot+1]
            } else {
                suffix
            };

            if next - prev <= 3 {
                // pivot can be removed; count that
                let without = [&nums[..pivot], &nums[pivot+1..]].concat();


                with + count_arrangements(prefix, &without, suffix)
            } else {
                with
            }
        }
    }
}

const INPUT: &str = r#"86
149
4
75
87
132
12
115
62
61
153
78
138
43
88
108
59
152
109
63
42
60
7
104
49
156
35
2
52
72
125
94
46
136
26
16
76
117
116
150
20
13
141
131
127
67
3
40
54
82
36
100
41
56
146
157
89
23
8
55
111
135
144
77
124
18
53
92
126
101
69
27
145
11
151
31
19
34
17
130
118
28
107
137
68
93
85
66
97
110
37
114
79
121
1
"#;
