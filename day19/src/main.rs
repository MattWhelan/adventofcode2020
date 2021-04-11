use anyhow::Result;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
enum RuleTree {
    Leaf(String),
    Alt(Box<RuleTree>, Box<RuleTree>),
    RefList(Vec<u32>),
}

enum RuleTokens {
    Literal(String),
    Ref(u32),
    Alt,
}

impl RuleTree {
    fn parse_tree(tokens: &[RuleTokens]) -> (RuleTree, usize) {
        if let RuleTokens::Literal(l) = &tokens[0] {
            (RuleTree::Leaf(l.clone()), 1)
        } else {
            let rt = tokens
                .split(|t| matches!(t, RuleTokens::Alt))
                .map(|ts| {
                    let ids: Vec<u32> = ts
                        .iter()
                        .map(|t| match t {
                            RuleTokens::Literal(_) => unreachable!(),
                            RuleTokens::Ref(id) => *id,
                            RuleTokens::Alt => unreachable!(),
                        })
                        .collect();
                    RuleTree::RefList(ids)
                })
                .fold1(|left, right| RuleTree::Alt(Box::new(left.clone()), Box::new(right.clone())))
                .unwrap();
            (rt, tokens.len())
        }
    }
}

impl FromStr for RuleTree {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<_> = s
            .split_whitespace()
            .map(|s| {
                if s.starts_with("\"") {
                    RuleTokens::Literal(s[1..s.len() - 1].to_string())
                } else if s == "|" {
                    RuleTokens::Alt
                } else {
                    let id: u32 = s.parse().unwrap();
                    RuleTokens::Ref(id)
                }
            })
            .collect();

        let (tree, _) = Self::parse_tree(&tokens);
        Ok(tree)
    }
}

#[derive(Debug)]
struct Rule(u32, RuleTree);

impl From<Rule> for (u32, RuleTree) {
    fn from(r: Rule) -> Self {
        (r.0, r.1)
    }
}

impl FromStr for Rule {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+): (.*)$").unwrap();
        }

        let caps = RE.captures(s).unwrap();

        let rule_num = caps[1].parse::<u32>()?;
        let rule_tree: RuleTree = caps[2].parse()?;

        Ok(Rule(rule_num, rule_tree))
    }
}

fn rules_to_string(rt: &RuleTree, rules: &HashMap<u32, RuleTree>) -> String {
    match rt {
        RuleTree::Leaf(s) => s.clone(),
        RuleTree::Alt(l, r) => {
            let left_str = rules_to_string(&l, &rules);
            let right_str = rules_to_string(&r, &rules);
            format!("(?:{}|{})", left_str, right_str)
        }
        RuleTree::RefList(refs) => refs
            .iter()
            .map(|r| rules_to_string(rules.get(r).unwrap(), rules))
            .join(""),
    }
}

fn rules_to_regex(rules: HashMap<u32, RuleTree>) -> Regex {
    let start = rules.get(&0).unwrap();
    let pattern = rules_to_string(start, &rules);

    let anchored = format!("^{}$", &pattern);
    Regex::new(&anchored).unwrap()
}

fn main() -> Result<()> {
    part1();
    part2();

    Ok(())
}

fn part1() {
    let mut parts = INPUT.split("\n\n");

    let rule_lines = parts.next().unwrap();
    let messages = parts.next().unwrap();

    let rules: HashMap<u32, RuleTree> = rule_lines
        .lines()
        .map(|l| l.parse::<Rule>().unwrap().into())
        .collect();

    let re = rules_to_regex(rules);
    let valid_count = messages.lines().filter(|m| re.is_match(m)).count();

    println!("Valid: {}", valid_count);
}

fn part2() {
    let mut parts = INPUT.split("\n\n");

    let rule_lines = parts.next().unwrap();
    let messages = parts.next().unwrap();

    let rules: HashMap<u32, RuleTree> = rule_lines
        .lines()
        .chain(
            r#"8: 42 | 42 8
11: 42 31 | 42 11 31"#
                .lines(),
        )
        .map(|l| l.parse::<Rule>().unwrap().into())
        .collect();

    let valid_count = messages.lines().filter(|m| matches(&rules, m)).count();

    println!("Valid 2: {}", valid_count);
}

fn apply_list(rule_ids: &[u32], rules: &HashMap<u32, RuleTree>, s: &str) -> Result<Vec<usize>, ()> {
    let rule = rules.get(&rule_ids[0]).unwrap();
    if let Ok(offsets) = apply(rule, rules, s) {
        if rule_ids.len() > 1 {
            let mut ret = HashSet::new();
            for offset in offsets {
                if let Ok(inner_offsets) = apply_list(&rule_ids[1..], rules, &s[offset..]) {
                    ret.extend(inner_offsets.iter().map(|n| n + offset))
                }
            }

            if !ret.is_empty() {
                Ok(Vec::from_iter(ret.iter().copied()))
            } else {
                Err(())
            }
        } else {
            Ok(offsets)
        }
    } else {
        Err(())
    }
}

fn apply(rule: &RuleTree, rules: &HashMap<u32, RuleTree>, s: &str) -> Result<Vec<usize>, ()> {
    match rule {
        RuleTree::Leaf(l) => {
            if s.starts_with(l) {
                Ok(vec![l.len()])
            } else {
                Err(())
            }
        }
        RuleTree::Alt(left, right) => {
            let left_result = apply(left, rules, s);
            let right_result = apply(right, rules, s);

            if left_result.is_ok() && right_result.is_ok() {
                let mut lefts = left_result.unwrap();
                lefts.extend_from_slice(&right_result.unwrap());
                lefts.sort();
                lefts.dedup();
                Ok(lefts)
            } else if left_result.is_ok() {
                Ok(left_result.unwrap())
            } else if right_result.is_ok() {
                Ok(right_result.unwrap())
            } else {
                Err(())
            }
        }
        RuleTree::RefList(ids) => apply_list(&ids, rules, s),
    }
}

fn matches(rules: &HashMap<u32, RuleTree>, s: &str) -> bool {
    let start = rules.get(&0).unwrap();

    if let Ok(offsets) = apply(start, rules, s) {
        offsets.contains(&s.len())
    } else {
        false
    }
}

const INPUT: &str = r#"34: 50 12 | 92 57
42: 9 12 | 124 57
115: 12 106 | 57 91
106: 57 66 | 12 12
71: 39 57 | 27 12
56: 69 57 | 6 12
10: 12 53 | 57 77
46: 3 57 | 122 12
20: 66 75
67: 12 126 | 57 106
78: 57 28 | 12 33
25: 3 12 | 116 57
74: 46 12 | 24 57
103: 12 40 | 57 116
94: 126 57 | 91 12
116: 57 12 | 12 12
60: 57 61 | 12 48
0: 8 11
110: 95 57 | 81 12
55: 75 57 | 106 12
100: 57 41 | 12 27
58: 116 12 | 85 57
61: 92 57 | 75 12
84: 47 12 | 87 57
65: 72 12 | 48 57
5: 55 12 | 77 57
112: 102 57 | 97 12
99: 18 12 | 108 57
39: 57 50 | 12 15
52: 121 12 | 39 57
47: 125 57 | 74 12
31: 57 117 | 12 73
4: 40 57 | 98 12
53: 91 57 | 106 12
124: 84 12 | 23 57
48: 57 126 | 12 116
123: 85 12 | 116 57
26: 57 121 | 12 79
51: 61 57 | 107 12
6: 12 86 | 57 109
122: 66 66
93: 57 68 | 12 110
37: 40 57 | 91 12
44: 57 50 | 12 85
12: "a"
45: 25 57 | 58 12
19: 57 116 | 12 50
33: 57 36 | 12 70
108: 16 12 | 46 57
95: 12 98 | 57 75
82: 85 57
69: 57 41 | 12 118
104: 12 3 | 57 75
35: 122 57 | 91 12
32: 12 53 | 57 16
27: 85 12 | 40 57
40: 66 12 | 12 57
66: 57 | 12
101: 57 88 | 12 103
109: 57 116 | 12 106
80: 57 82 | 12 49
83: 85 57 | 126 12
64: 12 126 | 57 3
30: 12 62 | 57 64
17: 115 57 | 67 12
86: 12 116 | 57 15
15: 12 57
24: 57 75 | 12 126
11: 42 31
118: 12 126 | 57 40
98: 12 57 | 57 12
16: 126 57
89: 57 34 | 12 46
113: 92 12 | 15 57
126: 57 12 | 57 57
68: 12 118 | 57 123
79: 12 15 | 57 75
91: 12 12
96: 83 12 | 107 57
105: 30 12 | 51 57
49: 57 122 | 12 50
29: 57 63 | 12 76
117: 57 119 | 12 78
76: 96 57 | 90 12
114: 12 1 | 57 93
73: 114 12 | 112 57
54: 91 12 | 50 57
8: 42
63: 12 7 | 57 111
102: 57 101 | 12 5
41: 116 12 | 40 57
28: 57 10 | 12 32
121: 91 12 | 116 57
7: 13 57 | 104 12
23: 56 57 | 105 12
59: 12 71 | 57 60
92: 57 57
70: 4 12 | 21 57
57: "b"
119: 57 59 | 12 43
77: 57 85 | 12 50
87: 57 89 | 12 17
85: 57 12
43: 57 26 | 12 45
90: 22 57 | 54 12
50: 66 57 | 57 12
3: 12 57 | 57 57
88: 106 66
107: 12 3 | 57 15
36: 37 12 | 107 57
97: 52 57 | 80 12
13: 122 57 | 106 12
2: 57 65 | 12 120
120: 57 35 | 12 94
14: 2 12 | 99 57
18: 72 12 | 121 57
75: 12 12 | 57 57
1: 57 100 | 12 38
81: 57 106 | 12 75
9: 12 14 | 57 29
111: 35 12 | 113 57
38: 57 20 | 12 19
21: 116 57 | 126 12
22: 57 3 | 12 50
72: 126 12 | 91 57
62: 91 57
125: 24 57 | 44 12

bbbabbbaaaaaabbbbaaabaab
abababbabaabbabbaaababbaabbbabbbbbbbaabbbbaababaabaaabaa
abaaababbbabababaaabbbba
aabbababbababaaabbbbbbba
baabababaabaaaaabbabababbaaabbab
bbaabaababababbbabababbaaabaabaababaaaabbbaababb
bbbbbaababbbabbaaaabbbab
baababaaabababbaabbbabbb
abbbababbbabbbaababbaabbbbabaabbabbabbbbabaababbbababbbbbabaabaabbbabbbb
babababbbbbaababbbababaa
baabaaabbbbbabaabaabaabb
bbaaaaabbaaaaaabbbbbabba
aababbbabaabbbababbaabbb
abbbbaabbabaaabbbbaaaaabaaabbbab
aabaabaabbbaabbbaaababbb
abbabbbbbaabbbaabababbab
bbbabbaaabbbababaabbbaab
bbbbaaababbbaaaaabababab
abbbbaabaababbaaaabbbabb
aaaabbbaaabababbaaaaabba
babaaababbabbbbbabaaaaababbaabba
abbbbaabbabbababbbbaaabb
abaabaaabbbbaaabbbbababa
aaaaaaabaaabbabaaaaaaabb
ababbabbabbaabbbbbbaababaabbaaba
aababbabbbaaabaabaaaabbbbbbbbaaabbbabbab
baabbaabaaababaabaabaaba
bbabbabbbbaaabbbbababaabbbababaa
baabbaaabbbbbabbaaaaaabb
bbbbbaabbbabbaaababaabbabbbabbbb
bbaaabbabbbbabaaabaabbaa
bbbbaabaaabbaaaabaabbaba
bbbbababbabbaabaababbbba
bbabbbbbabaababbaaaaabaabbababbbbaabaabb
bbabbbbaabbbbbbabbaababa
ababbabaaaabbabaaaabbbab
bbaaaaabbabaaabbaababaaa
bbaabaababbabbbaabbbaaba
bbbbabaabbabaabaabbbbaaa
aabababbababbbbbbbababbb
babbaabaabaabababaabaabbabbaabaababaabaa
abbabbbabaaabbaabbbbabbb
babbbbaabababbaaabbbabbb
babbbbabbbaabbbaaabaaaba
bbaaabbabbabaaaabbabbbab
babbbaaababbbaaababaabba
abaabbbaabbbabaaaabbbaaa
aaabbbaaabaabaaabaababba
abbbabbaabbabbbbbbbabaaa
bbbbaaabbbabbbaaaaaaaaba
babbbbbbbabbbbabbbbaabba
bababbaabaaabbbbbbaabaaa
ababbbbbabbaabababbbbaba
aabababbbabbabbbaabbaaab
aabbbbabbbbbabaaabaaabaaabababaabaabaaaa
baaaabbbbabbbbbaaabbabbb
aabaabbbbbbbababbbbbabaaababababbbbabaaa
abaabbbabababaabaaabbaab
baabbbbaabababbbbaabababababbabaaabbbbabbbbbbbbbbbabbaaaaaaabaabbaabaaba
aababbaaaaaabababaabbbbbaaabaaaaaaaababb
abababaaabababaaaaaabababbbbaaba
bbbbbaabbabbaabaaabaaaab
bbbbbabbbbbaabbabbbbaaabbaabbabaaaabbbbabbbaaabbbababbaaabbbbababababbbaabaaababbbabbbaa
bbbabbaabbbbaaabbababbaabbbaaaabaaabaabaababbbab
babaaabaabbbbaabbbbbabbb
baabbabaababbbababbabbabaababaaaaaaabbbb
abbbaaaaaababbbaaaabbaaa
bbaabbbabaabababbbbabaaa
aaabbbbbbaababbbababaaaaaabbbabbaabbaaba
bbabaabababaabbbabaabbabaabbabababbbaaaaaabbaaabaabaabbaaabbbaaa
bbaaaababbbbbabaabbbaaba
aaaaabbbbbbabaabaabaababbbbaaaabbaabababbabbbbbaaabbaaaa
babbbbabbabbaabbaabbabba
aaabbababbbbabaababbaaaa
bbbbabaabbabaaaaababbabaaababbbbaaabababbaababbaabaaabbb
bababaaabbabbbaaabbabbaa
abaabaaabaaaabababbaabaabaababbbababaabbbaababbabbabbaab
bababbbaaababbbbaaabbabaaaaababbbaabbaba
bbbabbbabbbbbabbbabbbbba
abbbbaabbabbaabbbbabaabbaaaaabaaabaababa
aaabbababbbbaabbaaaabaab
aaabbaabbaaaaababbbabbaababbbbaabbaababa
aaaaaaaabbaaaaabaaabbaab
bbbabbabaabbbaaaabbabaaa
bbabaaaaabaabaaababbbaabbaaaaaabbbbbabbaabbbaabbabaaaaaaaabaabbbbbabaaba
aababbababaaabaaaaaaabba
abbbabaaaabbbbbbababbbba
babaaabbbaabbbbbabbbabbb
abaabbabbbbaabaabbabbabaaabaabababbaabba
bbbaababbbaabaabaaabaaab
abbabbbaaababbbaaaabbabababbbbaabbaabaabbaabbabbaaaaabba
aabbabaabaababaabbaabbbbaabbaaba
aabbaabababbaabbaaababaaaaabbbbabaaaabaaaaaaabaa
aabbababbbaabbbababbbaaaabaababb
bababbaaabaabaaaaabaaabb
abbabbbababbbbbbaaababba
babaaabbbbabaaaabaaabaab
abbababbbbaaabaaaaaabbaa
aaaaaaaaabbbabaabaabaaba
aabbbbbbabababbaababbbab
aaaabbbaaabababbbabbbaaaabbbaaabbbbbbaabaaaababbaabaaaabababbbbabbbbbaabbbbbabab
abaabaaabbbbaabbbbbbaaabbaabaabaaabbaabb
abaaababbbaabbbaaabbaaab
bbbaaaabbbbaabaaababbabb
bbbbbbaaaabbabbabbbbaaaaaabbbbaaaaaababbbaaabaaaabbaaaabaaababaa
bbaabaabaabababbabaaabbb
baababaabbabbbbbbbaabbab
baaaabbbaabaabbbbaaabaab
abaabbbabbaaabaabbbbabbb
bbbbaaababaaabababbabbbbbbbabaabbbbabaabaabaabbb
aaaaaaababbabbbbaabbaaba
baabbaaaaabaababababaabb
aaaababaabbaababbabbaaab
bbbabbaaaaabaababaabaaaa
bbbabaababbabbbbabbbbbab
bbaaabbbabababbaabaaaabb
bbbbaababbabbabbabaaabaabbbbaaaaabbabbaabaabbabbbbababbbaaaaaaabbaaaaaaababbabbbaabbabbbbaabbbaa
abababaabbabbaababbbbbbababbaaaaabaabbaaaabbabbaabbbbbabbaabbbaaaaaaaaaaaaabbbabaabaaabbabababaa
bbbabbaabbbaaaabbaabaaba
bbabbabaabbbbbaabbaabaaaaababaaa
aaaaaaaabaaaababaaaabbbb
bbaaabbabbabbabbbababbbb
bbbaaaaaabaaaabaabbbbbab
babbbbbbbbbbabaabbbbaaaa
abababaaaabbababaabaaabb
bbabbbbaaababbaabaaaabba
abaaaaabbbababbaaabababa
abbaaaaabbababbaaaababab
bababbbababaaabbabbbaabb
bbabbbbbbabaabababbaabaa
aaababaabbaabbbababaababbbabbbaaaabbabba
bababbaabbbaaaaaabbbaabb
aabbababbbabbbbbbabbbabb
ababbaababbbbbbabbbbababaaaabaab
babbabbbaabbbbbabaaabbab
bababaaababaaabaabaabbbabaaaababaababaabbbbababa
babbbbaabaaaabbbbbaabbaaababbaaa
bbabbababaabbaaaaaaaabba
baababaaaaaaaaabaabbabba
abbaaabaabbaaaabbabbaabbbbbbabba
abbbaaabaaaaaaabbbabaaaaabaabaabaabbabaababbbbaabbaababa
baabaaabaaabaabaabbaaabb
abbbabbababababaaaaaabbbaabbaabbbaabaaba
baaaababbbbbaaabbbbabbab
bbbbaabbbabbbaaabbbbaaaa
baabbbaabbabababababaabb
baaaabbbbbabbbaaaaabaaaa
abaabaaaabbbbababbaaaabbbabbabaaaabbbabbaaaaabab
baabbaabaaababaaaaaaabba
baaabbbbbbbabaababbbababaababaaa
aaabbbaaaaaaaaabaababbaabbaabbbaabbaaababbbabaabababbabbbababbbb
abbbaaabbbbbbabbababaaaa
abbbabbaaabaabbbbabbbabb
bababbbaabbabbabbbbaabaabbbbaaaabbabbaaa
aabbaabbaababbbbbabbbbaaaaabbbaaaabaaaaaaaabbbbabaabbabababbaaabbbbabbbabbbaabaa
abbbbbabbaaababaaabaaaababababab
bababbaaabbaaababbbaaaabbbaaaabbbabaaaaa
aaabaababbbabbaababaabaa
aababbaaabaabaaaabbbbbbb
aaaaabbbbabbaababbabbbab
aaaabaabaaaabbbbabbbbababaababbb
abaabbbaabbbbbaabaabaaaa
bbbabbbabbaabbbbbababbbaaababaab
baababaabaabbbbaaababbbaaabbaaaa
ababbaabaaabbbaaabbbbabb
bbabaaaaabbabbbbaabaaabb
babbaabbbaabbbababababbbbbaabbaaaabaaabbabbbaaba
aabbabaabbaabbbaabbbaabb
abaaabaabbabbababbabbaba
babaabbaaabbaaaaabbaabbbbabbbaab
abababbabbabbbaababaabababbababbbabbabbbabbaabba
bbbabbaabaabbbbaaaabbaab
baabbbabbbaabbbbbaabbbbabbaabbabbabbaaabbaaaabba
babbabbbbaabbbaaaabbbbbb
bbaabaabbabbaabababaaaaa
bbaaabaababbaababaaaaaba
aabbbbaabbbbabaabaaabaaa
babbabbbbaababaabbaaabbbaabbbbbaababbbaaaaaaaababbbbabbb
abababbaabbbaaaabaabaabb
bbbaababaabbbbabaabbabaaabbbabbbabbabaaa
bbaabbbabbbaaaaaaaabaababbaaabaaababababbaababbabbbbbaaa
babbbbaabbbabaabaaaabaab
aaabbbaabababbbabababababbaaaabb
bbbaabbbaabbbbbabbabbbaaaaaaabbaaabbaabb
bbbbabaaababbbbbbaabaaba
bbabababbababababaaaaaaabbabaababababbbbabababababbabbab
baabbbbbbabbbbaabbabaabbbbababababbbaaaababaabaabbaaabababbabababbbbaaba
aabaaaaabbbbbabaaabbabbb
aababbabaaaababababababaabbabbba
ababaaabbbabbabbaababbabbbaaaabaaaabaaabbbbbbaaa
baabbaababbbabbaaaababba
aabbaabaabbabbabbabbbaabaaabaaababbbbbab
abbbababbaabbaabbabbbbbbbaabbbaabbabbbab
ababaaabbbabbbaaaaaaaaaabbaaaaabaaaaaaaaababaaaa
baabbaabaababbaabaaabbab
aaaabbabaaabaabbaabbababaaabbbaaababaaaa
abbababbabaaaaabaabbbaba
abbaababbaabababababbbaaabaabbbbbaabbbbbbbaaaabbbaaaaaababaaaaba
bbabbbbbbbaabbbbbaaaabba
bbbbababaababbaababbaaaa
baaaabbbabaabbbaabbbabbb
bbabaababbbbabababbbbbbb
baaaaaaaabababbaaabaaaba
abbbbabbbaabbbbbaaabbaaabbbababbabbaabbaabbaabbaabbbabbbbaabbaaaabbaaaab
abbbabaaabaabaabaaaabaaa
aaabbbaabbbbabaabaababbb
bbbbabaaababbabaababbbaa
baaaabbbbabababbabbabaab
bbaaabbababaaababaaaaabababbaaabaabbaaabaaaababbbaaababbbbbbabba
bbbbbbababbbabbabaaababa
babaabbbaababbbbaaaabbbb
aaabaababaabbaabaaaabbbb
abbbbaabaabaabaaaaaaabaa
aaabbabaababbababaabaaba
bababaaabaaabbaaaabbbaba
bbabababababaaababaabbabbbaaaabababaaaaa
bbbaaaabaaaababaaaabbaab
aaaaabbbaaababaabbaababaabbbaababbababaaaaaaababaabbabbbbbbabaabbaaaaabb
aaaabaaaaabbaabbbaabbabbbaabaabb
bbbbbabbaaabbbaaaabbbbaaaabbbbaaaaabbabaabbaabbaabbbabbb
aabbabaabaabbaaaaaaaaabb
baaaababbaababaaaaabbabababaaaab
abaaabaaabbaaaabaabbbbabbbbbabaaababaabb
babaaabaaaaaabbbabbbaaaaaabbababbbbabaabbabaabbaaaaabaaabbbababbabbababa
bbaaaabaabbbabbaabbbbaaa
bbabababaaabbbaaabbaaabb
aaabbababaabbaabbabbbaba
bbaaaaabbabaabababaababa
babbbaaabbbbbabaaababbabbaabbbbbbbabaabbbabaaaababbaabbbababaabb
abababbaabbaaaabbbbbaaba
aabbbbbbbbbbaaabaabbbabb
babbbbabaababbbaaaabbabababaaabbabbababbabbaabaabaaaabbaabbbbabbabbababa
baabbbbbbaabbbabaaabaaab
baabababaabaaaaabaaaabba
bababaabaabaababbaaaabba
babbbbbbbbabbbbabaabaaaa
babbaababaabababababaabb
babbbbabaababbbabbaabaabbbbbbbababbbaababaaaabbabbbabbab
ababbbbbbbbbbabbbbbabaabaaaabbbbababaaba
abaabaababababbbbaaaaabb
bbaaaabaabbabbbbaaabaaaa
aaaaabababbabaaabbaaabbbaabbabaabbbababaababbabaaaababbabaababbbbbbbbaba
bbaabbaaaaaababaaaaabbaaabbabbab
baaaabbbabbaababaaaaabab
aaabbabbbabbaabbbbaabababbabaaababbaaababbbabaaabaabbbbaabbaabba
bbbbaabbbaaabbaaabbbbbbb
bababbbababbababbbbababa
abbbbaababbbababbabaabba
abbbbbaaababbbbbbabbbbba
abbbbaabbabbabbbbabbaaab
abbababbaababbaabbbbabba
abbabaaaaabbbaaabbbbaaabaaabbbbbbbbabaababbabbbbbbbaabbbababbbbbbabaaaab
bbbbbabababaababbbbbaaabbababaabababbaababaabbababbbbaaa
abaabaababaaababababbabb
bababaaabbbaaaaaaaababaaaabababa
aaabaabaaaababbbbaaabaabbbbababbbabbbaba
abaaaaababaaabaaabbbaabb
aabbbbaaaababbbaaaaaaabb
abbbabaabababaaaabbbabbb
aaaaaaaaabbabbbbaaaabbbaaaabaaabbaaabaaa
aabaabbbabababbbabbabbab
bbbaaaabbabaabbbababaaaa
bbbbababaabbaabaabbabaaaabbaabbbabaababbbababbbabbbabbaaabbabbaaaabaaaabaabababb
aabbbbabbaaaaaabaaaaaaaaaabbbabbaaabbbba
baaabbbbbbaabbbabbbabbabbabbbabb
bbabbbaaaaababaaaabbabba
abaaabbbabbbabaaabaaaaabababaaabaabbbaabaaabaaabaabaabaabbaabbab
abababbbabaabaaaaabbababbaabaaaaaabbbaaabaaababa
bbabbbbbbbaaabbababbbabb
aabaabababaaaabaabbaaaababbababbaaabaaab
bbbabaababababbaabaaabba
abbaababaaabbbaabaaaabaa
ababbbbbbbabaabbababbbab
aabbbbbbbbbaabaaabababab
babaababbaababaaaabbbbbaaaaaaaaaaaaaabaabaaababa
abaabbababaaaabaababaabb
aaaaabbbbbbbbbababbbbabb
baabababbbbaaaaabababbab
bbbaaaabbbabaabaaabbbaab
bbbabaabaababbabaaabaaabaaaaababbbbbabab
bbbbaaabbaabbaaabbbbaaabbabbbbabaaaabbaabaaabaaaaaabababaaaaabab
aaaabbabbbbababbbabaabbbbbbabbbaaaaabaaa
abbbbaabaababbbaabbabbab
ababbbbbbabbaabbbaabaaabaababaaa
abbaaaaabbbbbbaaabbababbaabbbaaa
abababbaabaaababbababbab
aaaaabbbabbabbbaaaaabbaa
abaabbbbabbbaaaaaaabbbab
bbbabbbabababababbababbabbbaaaba
babaaabababaaabbaabbabba
babaaaaaabaaaaaabaabbabb
babaaabbbabbbbaabaaaaaaaaababaabbaaababb
ababbbbbaabbabaaabaaaabb
bbbabbaaabaabaabaabbbabb
abbbaaabbbbbbbaaaaabbbba
bbaabbaaaababbaabbababaa
baaaaaababaabaaabbabaabaaaabbabbbbabbbaaabbbaabbbaaababbaaabbbab
abbbaaabababbbbbaaababbb
aaaabbbabbbabaabababbabb
aabaabaabbbaabaaaaaaabba
baabbaaaaabaabaabababbbb
baaaaaabbbabababbaaaabba
aabbbbaaababbababbabbababbaabaabaabbbaabaaaabbaa
abbbaaabaabaabaaaabababa
aaabaabaabbbaaababaaaaaa
abbbbbaaabbbaaaabbaababa
baaabbbaaaaaaaaaaaaaabbbbbaaaabbaaaaaabb
baaabbbabbbaabaabbbbabbb
aabbbbaaabaabbbbabababab
baabbbbbbbaabbaabbabbaab
aababbababaabaaaabaabaabbabaabba
abbbabbabbabbabbbbbbaabbabababbaababaaaabbaababbbaaababb
abbbabaabaabababaaaabaaa
bbababbaaabbabaabbbabbab
aabbbbaabaaabbaabbabbbab
bbabbbbbbbababbabbbababb
abbababbabaaabababbbaabb
bababbbaaaaaaaabbbabbbbbaaababaaaaaaabbbbbababbb
babbababbababbbaaaaabaaa
aaabbababaaabbaabbaaabaababaabba
aabaabaabbbbbababaabbbabbaababab
aabaabbbbaaabbaabaaababb
babbaabbaabababbbbbaaaba
bbbaaaaabaaabbbaababaabb
abbaababbbaaaabbbaabbbabaaaaaabababbaabbbbbbbbbababababbbbbbaaba
ababbbbbabbbaaabaaabaaab
abbaaaaabbaaaabaaaaabbbb
baabbbbbaabbbbbababbbbba
ababbabababababaabaabaaaabbaabababaaababbabbbabbaaaaabbababbabba
bbaabaabbabbabbbabbabaaa
aabbbbababaaababbbbbbababababaaaabaaaabb
abbbabaabbbaabbbbbbabbbabaaaaabb
babaaabaabbbbbbabbbaabaababbabaa
abbaaaabbbbbabaabababaabbaababba
aaaabbbabbabbababbaababa
bbaaaaababaabaabaaaaabab
baaabbaabbababababbbbabb
bbbaaaaaaabbabababababbbbbabababababbaabaabaaaabaabbaababaabaaaa
baaaabbbbbababbababbaaaa
baaabaaaaaaaabbababbabaababaabababaaaaabbabbaabb
aaaabbababbbbaabaaaaaaababbaabab
bbabaaaaabaaabaaaaabbbab
aababbabbaaaababbbbabbab
abbbabaaabbaababaababaab
bbbabbbabbbbbababaaabaaa
baabbbbbbabababbaaabaabb
ababbaabaababbbbbbbababa
baabababbbaabbaabbabbbab
bbbabbbaabaabbababaabbaa
bbabaaaaabababaababbbaab
bbbbaabbabaabbbaabbaaabb
abbbabaaaabaaaaabbaabbaabbaaaaabaabbbbaaababbbbbaaaabaaa
abaabbabaababbbbaabaaabb
bbabbabbbabaaababbbaabaabababaabbaababba
abaabbbbbaababababaababa
abbababbabaabbbaaaaabbaa
bbabababbaabbaaababbbbba
babbbaaaaaaaaaaaabbaaabbbbbbbbbbbaaaabbaabaaaabaaabaabbaababababababbbbbbabaaaaa
bbaaabbababbaabbbbabaaab
abaabbbbbababbbaaaabaaaa
babababbaaaabbbabaaababa
bbbbbababbabaabbaaabbaba
baabababaabababbaaabbbaaabbbababbbabbabbbbaabbab
bbbaaaabbaabbaabaaababba
abababbabbabbbbaabaababb
aababbbaabbabbbbababbabb
ababbaabaaaabbabbbaaaabb
bbbbabaaaabbabaabbbbaaaa
abaaaaababbaaababbbbbaabbbabaaaaabaaaaababbaabbb
bbbaabbbabbbbaababbabbab
aaaaaaababbbaaaaaabababbbaaaaababaababba
aaaaabbabaababbaababbabb
aaabbabbabbbbaabaabbaaba
bbabbbbbbbaaaabaaabbbaba
aabababbaaabaabbababbababbabaaab
abaabbaaaabbaabbaaaaababababaabaabbbbbabbaababbaaaababbabaaabaaaabaaaababbbaaabababbbbbbbaabbabb
abaaababaaaababaabaaaaaa
aabbababaaaaaaababbaaabb
bbaabbaababbbbbbbbbabbbb
bbbaabbbaaaaaaabaaabbbbb
bbbbbabbbbabaababbbbbbba
baaabbbabbbbbbaabbbbbaaa
bbabbbbababbbaaaaabbabbb
bbaabaabaabaabbbababbaaa
abbbbaababaaaaabbabaabaa
aaaaabaaabbabaaabaaaaababaaabababbbabababbabbababbbbabaabaaaaabb
bbabbbbaabaabbbaaabbabba
aabababbbbaaabbbbaababba
abbbbaabbbaaaabaababaabb
aabbbbaabaabbaabbabbbabb
baabbbbbbabbababbababababaaaabaa
bbbbabaaabababbbbbabaaab
aaabaabbabaaabaababaabba
abababaaabbbaaabbbabaabababbbaba
bbbbbabaabaaaabaaaaaaaabaabbbaaa
ababbaabaaaaaaaaababbabb
bbaaaabababababababbabba
aaabaababbbaaaaaabababaaabaaaabb
bbbaabbbaaabaabbaabbaaba
abbbabbaabababbaaaaaabba
babaabbbabababaabababababaabababbbaaababbbaaaaaaabaaaabb
bbbbbaabbbaabbaaaababbabababbaabaaababaaaaaabbabbaaabaaaaabbaaab
babaaabbbabababaabaaababbaabababababbabb
ababbaabbbbaababbabbabaa
aababbabbabbbbbbababbbbbaaaaaabbbaaaaaba
aabbbbbaaaabaabbaaabaabaaaaabaaa
bbaaabbbaabaabaaaabbbbbaababbabaaabaaaabbabaabba
abaaababaababbbbbabbaaab
babbbbabbaababababaabaabaabaaaab
abbaabababbbabaaaaabaabbbababbaaabaabbabaabbbbbbbaababbabababbababaaabbb
aabbababababbabababaaaaa
babaaabbbbabababbbbaaaba
aabaabababaabbabbbababab
bbbaabaaabbbbbbaaaababba
baabbbabbaabbaaaababbaababbababa
bbbbabaabbaabbbbaaabaabaabbbbbaaabbbbbaabaabbaba
babaaababbbaaaaaabaabaabbbbaabaabaaaabaa
bababababababbaaaabaaaba
aaabaabbaaaaabaabbbaababbaabbbbb
bbbbbbabaabbbbbaaaabbaaa
babaabbbabaaaabaabbaabaa
bbbabaabbbbbababaabbbbaaabbabbbbbbaaaaaa
bbaabbbbabbbaaaabaaaaabb
bbbabbabaabaabaaabaabbbbbabaaaaabbbababa
baabbbbaaabbbbbabbbabbaaabbaabababbababa
baababaabbabbbbbabbaaabb
bbaaabaaabbaaababbabbbab
babaaababababaaaababbbbbbbbbbbaabbbaabbbaaaabaaa
babababababbbbbbaababbab
abbbaaaabbbabaabaabbaaaa
abaaaaabbaabaaabaaababba
aababbabaabaaaaaabaaaabb
aabbbbabababaababbabbaaaaabbabbaaabbabbb
aaaabbabbababaaabbbabbbaabababaabbababbbbbabbaaabbaabbab
bbaabbaaaaabbabbababbaabaaaabaaa
baaaaaaababaabbbbabbaaab
bbabbbaaabababbbbabbbbabaabbbaaa
aabaaaaaabaaaabaabbbbaba
abbbbbbabababbbababaaabbababbbabaababbbbbaababbaabbabbabbbabbbaabbbbbbab
abaabbabbbaabbaabbaabaaa
bbabababbbaaaababbaababb
abbabbbaaabbabaabaaaaaba
abbabbbbabababaabbbbabbb
bbbbaaabbbabbbbbaaaabaaa
aabaabbbbbbabaababbaabba
abbbaaaaababbbbbaaababaaaabaabba
baaabbaaaabbbbbbabaabbbbaabaabaaaaaaabbaabaababbabaababb
aabaaaaaabbaaaaabbbabbaababababbaababbbaaaabbbbb
babbabbbbababaabaabaaabb
bbbbbbaabaabbbaaabbaabaa
aabbbbaaababbabaabababbbbbbbbabaababaaaa
bbaaabbbaababbbabbaababb
aabbbbbabaabababababbbab
bbabbbbbbbabbbbabbbbaaba
abbbabaaabbbabaabbaaaaaa
bbabaabaabbbbbaaaabababbabbbbbaaaabbbabbabbabaab
abbaaaabaabbbbbabbaabbbabaaababbbbaabaababababbaaaaabbab
aabababbabbbabaababaabba
babbbbbbbbbaabaabbababbb
aaababaabbabbababababbab
aabaabbbbabbaabbabaaaababbaaaaabababbbbbbaaabbabbbaabbab
bbaababbbaaabbabbbbabaaaabaaaabb
aabaababbbababababbabbab
abbbbbaaabaabbbbaaaabaab
abbbbbaaabaabbbaababbbba
baaabbaaaabaabbbbabaabba
bbbaababbaaaabbbabbabaaa
abbbabaaabbaaaaaabaabbabbbbabaaa
bababbbaabaabaaabaabbabb
baabbbbbabbbbaabababbbaa
babaaabaabbabbbbbbbbabab
bbbbaaabaaababaabaabaaba
aaabaabbabbbabbaabbabbab
abaabbabaabbbbbabbaabaaa
baaabbbbbbbbbababbbbbbabbbabaaab
abbbaababaabbbbaaaabbaaabaaaaaabbababbbbaabbaabbbbaabbbaaaabbaab
bbbbababbbabbbaaabaabaaaaaabbababababbaaaabbaabb
bbabaaaaabaabbabbaaaababaabbabbb
bbaabaabbbabbbaababbbaaabbabbaab
"#;
