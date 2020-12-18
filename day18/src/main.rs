use anyhow::Result;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Token {
    Num(i32),
    Mul,
    Add,
    Open,
    Close
}

#[derive(Debug, Clone)]
enum Tree {
    Leaf(i32),
    Branch {
        left: Box<Tree>,
        op: Token,
        right: Box<Tree>,
    }
}

impl Tree {
    fn eval(&self) -> i64 {
        match self {
            Tree::Leaf(n) => *n as i64,
            Tree::Branch { left, op, right } => {
                match op {
                    Token::Mul => {
                        left.eval() * right.eval()
                    },
                    Token::Add => {
                        left.eval() + right.eval()
                    },
                    _ => panic!("Eval error, bad op token")
                }
            }
        }
    }
}

impl FromStr for Tree {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<Token> = s.chars()
            .filter(|ch| !ch.is_whitespace())
            .map(|ch| match ch {
                '*' => Token::Mul,
                '+' => Token::Add,
                '(' => Token::Open,
                ')' => Token::Close,
                d => Token::Num(d as i32 - '0' as i32)
            })
            .collect();

        fn tree(tokens: &[Token]) -> Result<(Tree, usize), anyhow::Error> {
            let mut i = 0;
            let mut left = None;
            let mut op = None;
            let mut right = None;
            while i < tokens.len() {
                let t = tokens[i];
                if left.is_none() {
                    left.replace(match t {
                        Token::Num(n) => Tree::Leaf(n),
                        Token::Mul => {
                            return Err(anyhow::Error::msg("Unexpected Mul"))
                        },
                        Token::Add => {
                            return Err(anyhow::Error::msg("Unexpected Add"))
                        },
                        Token::Open => {
                            let (branch, consumed) = tree(&tokens[i+1..])?;
                            i += consumed;
                            branch
                        }
                        Token::Close => {
                            return Err(anyhow::Error::msg("Unexpected Close"));
                        }
                    });
                } else if op.is_none() {
                    op.replace(match t {
                        Token::Mul => Token::Mul,
                        Token::Add => Token::Add,
                        Token::Close => {
                            return Ok((left.take().unwrap(), i+1));
                        },
                        unexpected => {
                            return Err(anyhow::Error::msg(format!("Expected op, got {:?}", unexpected)));
                        }
                    });
                } else if right.is_none() {
                    right.replace(match t {
                        Token::Num(n) => Tree::Leaf(n),
                        Token::Mul => {
                            return Err(anyhow::Error::msg("Unexpected Mul"))
                        },
                        Token::Add => {
                            return Err(anyhow::Error::msg("Unexpected Add"))
                        },
                        Token::Open => {
                            let (branch, consumed) = tree(&tokens[i+1..])?;
                            i += consumed;
                            branch
                        }
                        Token::Close => {
                            return Err(anyhow::Error::msg("Unexpected Close"));
                        }
                    });
                    let branch = Tree::Branch {
                        left: Box::new(left.take().unwrap()),
                        op: op.unwrap(),
                        right: Box::new(right.take().unwrap()),
                    };
                    left.replace(branch);
                    op = None;
                    right = None;
                }

                i += 1
            }

            Ok((left.unwrap(), i))
        }

        let (ret, _) = tree(&tokens)?;
        Ok(ret)
    }
}

impl Tree {
    fn parse_with_precedence(s: &str) -> Result<Self, anyhow::Error> {
        let tokens: Vec<Token> = s.chars()
            .filter(|ch| !ch.is_whitespace())
            .map(|ch| match ch {
                '*' => Token::Mul,
                '+' => Token::Add,
                '(' => Token::Open,
                ')' => Token::Close,
                d => Token::Num(d as i32 - '0' as i32)
            })
            .collect();

        fn parse_term(tokens: &[Token]) -> Result<(Tree, usize), anyhow::Error> {
            match tokens[0] {
                Token::Num(n) => {
                    Ok((Tree::Leaf(n), 1))
                }
                Token::Open => {
                    let (tree, off) = parse_tree(&tokens[1..])?;
                    Ok((tree, off + 2))
                }
                _ => {
                    return Err(anyhow::Error::msg("Bad token in term"));
                }
            }
        }

        fn parse_op(left: Tree, tokens: &[Token]) -> Result<(Tree, usize), anyhow::Error> {
            match tokens[0] {
                Token::Add => {
                    let (right, right_off) = parse_term(&tokens[1..])?;
                    let sum = Tree::Branch {
                        left: Box::new(left),
                        op: Token::Add,
                        right: Box::new(right)
                    };
                    Ok((sum, 1 + right_off))
                }
                Token::Mul => {
                    let (right, right_off) = parse_tree(&tokens[1..])?;
                    let prod = Tree::Branch {
                        left: Box::new(left),
                        op: Token::Mul,
                        right: Box::new(right)
                    };
                    Ok((prod, 1 + right_off))
                }
                _ => {
                    return Err(anyhow::Error::msg(format!("Bad token in tree: {:?}", &tokens[0])));
                }
            }
        }

        fn parse_tree(tokens: &[Token]) -> Result<(Tree, usize), anyhow::Error> {
            let (mut left, mut off) = parse_term(tokens)?;

            while tokens.len() > off {
                if tokens[off] != Token::Close {
                    let (op, op_off) = parse_op(left, &tokens[off..])?;
                    left = op;
                    off += op_off;
                } else {
                    return Ok((left, off))
                }
            }

            Ok((left, off))
        }

        let (ret, _) = parse_tree(&tokens)?;
        Ok(ret)
    }
}


fn main() -> Result<()> {
    let input: Vec<Tree> = INPUT.lines().map(|l| l.parse().unwrap()).collect();
    println!("Sum of trees: {}", input.iter().map(|t| t.eval() as i64).sum::<i64>());


    let input2: Vec<Tree> = INPUT.lines().map(|l| Tree::parse_with_precedence(l).unwrap()).collect();
    // dbg!(&input2);
    println!("Sum of trees: {}", input2.iter().map(|t| t.eval() as i64).sum::<i64>());

    Ok(())
}

const TEST: &str = "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2";

const INPUT: &str = r#"6 * ((5 * 3 * 2 + 9 * 4) * (8 * 8 + 2 * 3) * 5 * 8) * 2 + (4 + 9 * 5 * 5 + 8) * 4
2 + (3 + 3 + (9 + 3 * 4 * 9) + 2 + 5 * 7) * 7 * (3 * 6 * 5 * 9 + 6) + 6
3 * (7 * 7 + 5 * 2) + 7 * 8 * 9 * 6
9 + 3 * (3 + 3 * 2 + 4) * 2 * (5 + 9 * 9 * (2 + 5 * 2 * 4) * 6)
2 * 3 * (2 + 7 + 3 + 7) + 3 + 7
(4 + (4 + 7 * 6 * 5) + 6) + (3 * 2 + 2) + 3 + (8 * 5 * 6) + 9
(2 + 3 + (8 + 9 + 4 * 8) * (5 * 3 * 7 + 9 * 5 * 8) + 6) * 9 + 6
(4 + (6 * 5) + 4 + 7 + 7) + (2 + 3) * 2
2 + 9 + 7 + (8 + 7 + 2 * 4 * 8 + 2) + (3 + 5 * 5 * (7 * 7 * 2 * 7)) + ((6 * 6 + 7 + 9 * 9 * 7) + 4 + 3)
7 + ((8 + 4) + (4 + 9 + 9) + 3 + 3) + 7
9 + 9 * 6 + 6
(7 * 7 * 7 * 3 + 2) * 5 * 7
3 + 6 + ((9 * 6 * 7 + 9 + 8 * 6) * 5 * 5 * (8 * 6 * 9 + 6)) * 8 * 9 + 8
4 + (7 * 7 + 9 * (5 + 4 * 3 * 4 + 6 + 7) * 6 * 2)
3 + 5 * (4 * 8) * 9 * (2 * 5 * (5 + 9 + 5 * 8 + 6 + 6)) * 5
3 + (8 + 2 * 6 * 7 + 5) + (2 * 2 + 5 * 8) + 6 * 9
((9 * 5 + 9 + 5 + 8 * 2) * 5) + (7 + 6 * 2 + (5 * 9 * 9) * 3) * 5 * 4 * 9
9 * 9 * (3 + 6 + 4 * (3 + 3 + 7 * 6 + 4 * 6)) + 2
4 + 5
8 * (3 + 3) * 9 * ((9 + 2 * 2) * 3 * 8 + 7) + 7 * 3
(5 * 7 + 5 * 9 + 7) * 5 + 5 * 3 * 8 * 6
(6 + 7 + 6 + 8 * (6 * 8)) * 8
4 + 3 + (7 * 7 * 2)
(8 * 7 * (7 * 2 * 2 * 8)) + 4
4 * (4 + 6 + 7 * 2 * (6 + 6 + 3 + 6 * 2 * 5)) * 9 + 7 + 6 + 8
9 * 8 + (7 * (8 * 7 * 4 + 8 * 6 + 2) + 7 * 9) + 8 + 2
(9 + 7 * 8 * 8 * (7 * 4 * 7 * 2 * 6 + 2) + 6) * (2 + (2 + 2 + 2 * 5 + 5 * 9) + 5 + 8 + 6)
4 + 2 * 7 * (9 + 8 + 3 + 9 * 4 + (8 + 7 * 6 + 7)) + 4
7 * 9 * 8 * (5 * 4 + 8 + 2 + 2 + 4) * 5
4 * 4 * (5 * (6 * 2 + 9)) + 5 * 3
2 + 2 * (5 + (9 * 7 * 8 * 9 * 8 * 2) * 6 * (4 + 6 + 6 + 6)) + (3 + 9 + 6 * 2)
2 + ((3 * 4) + 2) * 9 * 4 + 4 * 3
(4 + 2 + (5 * 6 * 2 + 7 + 8 * 2)) * 6 * 8 * (2 + 3 * 2 * 8)
9 * 2 + (5 * 7 + (3 * 9 + 2 + 2 + 5)) + 8
4 * 9 * 7 + ((5 * 4 + 6 * 3 + 3) * (8 * 6 * 3 + 9 * 8 + 7))
4 + (9 * 7 + 4 + 6 + 5 * 2)
7 + 4 * 9 * ((8 + 8 + 8 * 6) * (5 + 2 * 2 * 5 + 8 * 5) + 5 + (8 + 4 * 2 * 5 + 6) + 6)
(7 + (2 + 7 + 8 + 4 + 9 + 4) + 9 * 4 + 5 * 5) + 6 + ((7 + 7 * 5 + 3 * 3 + 3) * 6)
((4 + 7 * 9 + 5 * 4 + 8) * (7 * 9 * 3) + 7) + (5 * 2)
9 * (8 * 8 * 2) * 2 * 5 * 4 + 6
9 + ((9 + 5 + 9) + 2 + 4 + 8) * 3
3 + (2 + 5)
9 + (2 + 4 + 4 * 9 + (7 * 2 + 3 * 8 + 3 * 2) + (4 * 6 * 9 + 7)) + 7
(7 + 6) * (4 * 5 * 4)
(9 * 2 * 7 + (7 * 3 + 4 + 6 * 3 + 9) + (9 + 6)) + 4 + 8
5 * ((6 * 2) * (9 + 7 + 7 + 2)) * (3 + 9 * 6 + 2 * 3) + 4 + 7
6 + (9 + (8 + 6 * 4 + 7)) + 2 * 8 * 7
4 + (8 + 4 * 5 * (2 + 2 * 4) + 7) * 2
3 * 5 * 2 + 4 * 6 + 9
(7 + (2 * 7 * 4 + 5)) * 4
6 + 5 + ((7 + 6 * 4 + 7 + 5 * 5) * 5) * 8
7 * 3 + 4 + 9 * (7 + (9 * 5 * 7 * 3 + 8 * 2)) + 6
(6 * (2 + 8 * 2 + 2) * 2 * (5 * 7)) + 8 + (4 + 4 * 7 + 8 + 8 + 9) * (6 + 7)
6 * (2 * 6 + 6 * 3 + 5 + 8) + 6 + 3
8 * (8 * 3 + 9 * 7) * 7
6 * 7 + 8 * (8 * 9 + (5 + 8 * 5 + 6) * 6 * 2) + 7 * 3
((9 * 4 * 4) + 4 * 2) + 3 + (9 + 3 + 8) * 3 * 2
7 + (2 * 4 * (2 * 6 + 8) * 8) + 8 * 5
6 * 4 * 5 * 8 * (9 * 8)
6 * (6 * 3 * 9 * (2 + 3 + 2))
3 * 9 + 6 + 5
2 * (7 + 8 + 9 * 9 * 7) + 4 * 6 * ((8 * 3 * 2 * 3 + 9 + 9) + 3 * (8 * 5 + 3 * 5 + 7 + 9))
6 + 9 * 7 * 8 + (9 + 5 + 8 * 4 + 4 * 8)
6 * 5 + 2 + (4 + (8 + 7 * 6 * 9 * 2 * 3) * (8 * 6 * 4) + 5 * (7 * 8) * 2) * 9 * 2
4 + (3 + 8 * (2 * 8 + 9 * 6) * 6)
((3 + 7) * 9 * 9 + 9) * 7
2 * 9 * (4 * 9 * (9 * 8 + 6 * 5) + 3)
7 + 4 + 3 + 7 + (8 + 4 * 8 + 8 + 8) * 8
6 * (7 * 6 * (2 + 8 + 9 + 4) * (8 * 2) * 4 + (7 * 9 * 3 * 3 + 2 * 6)) * 6 + 8 + 5
6 * 6 + 8 + 7
3 * ((8 + 6) * 4 * (8 + 7 * 7 * 7) + 8 * 7 * 5) * ((9 + 9 + 3 + 7 + 4) * 7 + 4)
4 * 4 + 5 + 5 + 7
4 + (6 + 4 + 9)
7 * 3 + (5 * 6 + (5 * 3 + 7 + 2)) * 9
4 * ((2 * 4) * 6 + 8) + 6 * 5 + (8 * 9)
2 * (3 * 4) + 2 + 2
(2 * 4 * 5 + 2 + 4 * 5) + 5 + 3 * 6
9 * 2 * 6 + 3 + (3 + 8) * (6 * 4 + 2)
9 * 6 * 4 + (9 * 8 * 5 * 6 + (5 * 6)) + 9 * 7
9 + 2 * 6 * (9 + 3 + (5 + 2 + 3 + 8 * 2 * 6) * 9) * 7 + 9
8 + (7 * 2 + (3 + 6) * (6 + 6 + 8)) + 8 * 5 + 4
3 * (3 * 2 + 5 * 4 + (8 * 3 * 5) + (6 * 5 * 8 + 7 * 4 * 5)) + 6 * 6
6 + 7 + 7 + 2 + (2 + (4 * 3) + 3 + 8)
((2 + 3 + 2) * 9 + 2 * 4 * 7 + 9) * 8 + 7 * 4 + 3 + 2
(5 + (7 + 9 * 3 * 6 + 5 + 8) * 2) * 7
3 + 3 * 3 + (6 + 7 * 3 * 2 * 5 * 7) + (4 * 6 * 5 + 3 * (3 + 6 + 5 + 5) + 8)
4 * 5 + 8 + (5 + 6) + (4 + 2 * 2 + 4)
5 * 6 * 8 * (3 * 2 + 7 + (4 * 8 * 3) * 3 + 3)
5 + 8 * 6 * 4 * (2 * 4 + (3 + 6 + 9 * 6 * 2 * 6) * 9 + 3 + 9) + 9
(6 + (4 + 5 * 8) + 8 * 9) + 7 * 2
(5 + 2 * 3 * 3 * 2 * 8) + 3 + 2 + (7 + 4) + 4
(9 * 8 * 2) * 2 + 2
8 * 6 + 3
6 + 9 * (9 + 2) + (7 + 7 + (5 * 9 * 4) + (2 * 6)) + 5
5 + 7 * 7 * (7 + 8 * (7 * 9 + 2 * 6 + 9) + 3) * 6
2 + (5 * 3 + 7 * 5 * 3 * 3) + 6
4 + 9 * 8 * (4 * 8) + (3 + 5) * (8 * 5 + 6 + 9)
(5 * 6 + (7 + 3)) + 3 + 9 + 7 + 7
(8 * 5 + (3 + 6 * 2 + 3) * 8 + (7 * 3 + 7 * 4 + 9) + 9) + 7 + (8 + 2 + (7 * 7 * 3) * (4 * 5 + 4 * 9 * 8)) * 5 * (3 + 8 * 4)
2 * 5 + (6 + 8 + 8 * 7 * 7 + (7 + 3 + 9 * 6 + 7))
2 * 3 * ((7 + 6) + 8 + 6 * (4 * 3 * 7) * 7) + 3 * 9 * (8 + (2 + 9 + 9))
8 * 8 * (3 + 8 * 3 * 5 * 5 + 3)
(7 * 4 + 7 * 6 * 6 * (2 * 6 + 7)) * 3
8 * (3 * (5 + 8 * 8 + 7 + 2)) + 8
4 + (7 * (9 * 2 + 2 * 7) * 7 + 4 + 9 * 4) + 3
5 + 3 + 5 + 2 * ((9 * 9 + 2 * 3) + 2 * (3 * 8 + 8 * 7 * 6 + 7) + 9 + 5)
5 * 5 + 9 * (4 * 9 * 9 * 3) + 9 + (7 + 9 * 8 * 2)
7 * 7 + (7 + 2 * (9 + 6 * 3 + 2 + 3) + (2 * 6) + 6) * 2 * 5
(7 * (2 + 6 + 2 + 9) * 7) + (8 * 2 + 4 + 6 * 2 + 3) + 3
3 + 3 * 6 * ((3 * 5) * (8 * 6 + 4)) + ((9 + 2 + 5) + (4 + 2 + 2 * 3 + 5 + 6) + 7 + 5 + 6) + 5
5 * 9 + 9
(5 + 3 + (4 + 4) + 7 + 7 * 9) * ((7 + 7) * 5 * 7)
7 * ((7 + 5) * 2 * 7) * 7
9 * ((6 + 4) * (6 * 5 * 8 + 6 * 2 * 3)) * 8 + 5
3 + 2 + 5 * 5 + (8 + (8 * 2 + 9 * 7 * 4)) * 2
(6 + 2 * 5 + 3) + 9 * (5 * 6 + (7 * 6 + 8 + 4 + 8) + 8 + 8)
(3 + (3 * 4 * 8 * 3 + 6 + 8) + 9 + 8 + (3 * 9 * 9 + 4 * 8 + 9) + (7 + 6 + 9 + 6)) + 8 + 5 + 2 * 7
6 * (7 + (2 + 5 + 5 * 6 + 4) * 6 * (2 * 5 * 7 * 4 * 7 * 4)) * 9 * 2
3 * (7 * 4 + 7 + 9) + ((4 * 4 * 2 * 8) + 9 + 6 * 9 * (9 + 9 + 2 * 2) * 5) + 5 * 6 + 9
6 + (4 * 5 + 8 * 8 * 9)
((9 * 2) * 2 * (4 * 8 + 4 * 8) + 6 + 6) * 2
(5 * (3 * 2 * 2 + 4 + 4) + 6 * (8 + 6 * 3 + 6)) * (3 * 7 * (5 * 9 * 7 + 5 * 6 * 2) + 6) + 2 + 8 * ((9 + 5 + 4 + 8 * 5 * 4) + 3) * 5
8 * 4 + 8 + 6 * ((4 * 3 + 5 * 6 + 5) * 2) * 3
3 * (2 * 8 * 8 * 7 * (2 + 7) + 8)
(9 * 8 * (4 + 8 * 3 * 7 * 6 + 8)) + 7
9 * ((6 * 5 + 6 * 5 * 2 * 2) + (8 + 6 + 3 + 4) + 6)
2 * (8 * (5 + 9 * 4 * 5) + 9 * (5 + 5 + 8 * 4) + 8 * 5) + 4 * 3 + 3
(9 + 3 + 9) * (7 + 9 + 7 + 4 + (3 + 4)) * 2
(9 + 7 + 8 * 5) + 3 * 4 * 2 + 3 * 6
(7 + 3 + 3 + 2 * 9) + (6 + 6 + (7 + 4 * 5) + 6 + 3) * 8
5 * 7 * 9 + (6 * 9 * 7 * (3 + 4) + 4 + 6)
(3 + 4 + (8 + 5 * 6) * 9 * 6) * 6 * (8 + 7 * 8) * 3 + 2 * 5
7 + (8 + (3 * 4) + (4 + 9 + 9 * 7 + 4) + (2 * 2 + 7) * (6 * 9)) + (9 * (7 * 4 + 5 + 2 * 3) + 3 + 7) + 8 + 2 + 7
(6 + 8 + 4 * 8 + 9 + 2) * 3 + 4
3 + 7 * (4 * 3 + 2 + 3)
(5 + 8 * (8 + 2 + 3 * 9) + 9 * 7 + 2) * 8 * 7 + 3 + 5
(6 * 2 + 4) + 4
6 * (2 + (5 + 2 + 9 + 8 * 7 + 6) + 7) + 4 * 7 * 4
4 * 6 * ((9 * 2 * 6 * 4 * 2) * 5) + 9
(9 * (3 * 2 + 5 + 9 + 3 * 6) + 5 * 5) + 5 * 3 * 6
(9 * (3 + 6 + 2 * 4 + 8 + 2) * 8) * 9 * 2 * 3
2 + 6 + (7 + 9 + 7 + (6 + 5 * 8 + 3 + 7) + 8) + 3
(4 * 6 * 3) + (6 * 9 + 3 + (3 * 8 + 9 * 9 + 5 + 8) * 8 * 7) + 4
7 * 9 + ((8 + 8 * 2) * 7 + 5 * 2 * 8) + 8 + 9 * 6
3 + ((6 + 9 + 9 * 8 * 5 + 6) * 3) + 4 + (3 + 5 * (6 * 8 * 7 + 5 * 3) * 2 * 7) + (9 * 5)
3 + 6 + (5 + 2) * 6 * 8
(4 + 4 + (8 * 6 * 5 + 5 + 9 * 9) + (7 * 3) + 9) * 7
3 * 5 * 8 * 9 + 4
7 * ((3 + 2) * 9 * (4 * 4 + 5 + 7) * 7)
7 * 5 + 8 * ((8 + 3 + 5 + 3 + 4) + (6 * 7 + 6) + (3 * 9 * 7 + 6 * 6) * (7 * 7 + 9 + 9 * 9)) * 7 + ((3 * 9 + 7 + 6) * 5)
(9 * 5) * 6 + (7 * (3 * 6) * (8 + 6) * 8 + 6)
3 * 5 * 2 * 8 * (5 + 6 + 6) + (9 + 2 * 5 + 4 + 4 + 9)
(2 * 8 * 5 + 5) * 4
7 * (9 * (4 + 4 * 6) * 5) * 5 * 4 + 5 * 7
(6 + 2 + 2 + 4) + 4 * 9 * (6 + 3 * 4 * 9) + 2 * 8
6 + 9 + 9 + (5 + (5 * 3 + 7 * 9 * 3) * 7 + 8 + 3) + 4
(5 * 9 * 7 + 8) + 2 * 7 * 2 * 8
4 + 2 * 5 * 2 * 3 + 7
5 + 6 * 2 + (3 + 2 + 8 * (3 * 6 * 5 * 2 * 2)) + ((8 * 3) + (2 * 9 + 4 + 9 + 7 * 5) + 4 * 9 + (6 + 7 + 5 + 4) + (9 * 8 + 2 + 9 + 2))
((8 * 5 + 6 + 9 * 4 * 6) + 2 + (8 + 9) + 3) + 2 * 2 + (4 + 6 * 9 * 4) + 2 * (3 + 3)
2 + (3 + 3) * (2 * (4 + 4 + 2 * 9) + 9 + 9 + 8 * 9)
3 + (7 * 4 + 5 * (4 * 9) + (7 * 5 * 4 + 7 * 7 + 6)) + 8
5 * 5 * (6 * 3 * 9) * 9 + 6 * 7
9 + 5 * (7 * (2 + 8) + 6) + 8
3 + 3 * (8 + 2) * 2
5 * (8 * 7 + (9 + 4 * 7 + 7 + 2) + (4 + 3 + 5 * 2) * 5 * 5) * 2 * 6
4 + (5 * 5 * 2 * (3 * 5 + 5 * 5) * 5) * 7
5 + (7 * (9 * 3 + 9 * 3 + 5 * 8) * 9 * 3 * 5 * 7) * 7 + (6 + 4 * 8) + 9 * 2
6 * 7 + 4 + (2 + 2 * 4 + 9 * 4 + (5 * 5 + 4 * 5 * 6))
2 + (4 + 3 * (6 * 4 + 2 * 4) * 5 + 8 * 6) + (6 * (9 * 4) + 5 + 4 * 7)
4 + 6 + 6 + (5 + 3 * (3 + 4 * 8) + 9)
7 + 7 + 5 + 8 + ((2 + 2 + 5 + 2 * 5) * 3 * 8 * 9)
3 * (5 + 4 + 3) * (4 + 2 * 5 + 8) * 6 + (4 * 9 + 2 + 6) * 5
(9 + 7 * 9 * 2 * 7) * 7 + 2 + 4 + 8 * (7 * 9 + 9 + 8)
(7 * 7 * 5 + 4 + 3) + 9 * 8 + 9 + 4
(2 * 3 * 8 + 6 * 6 * 7) * (7 * 5 * (8 + 6 * 3 * 4 * 7 * 8)) * 3 * (7 * 4 * 7 * 8)
((4 * 5 + 4) + 8 * 3 + (9 * 9 * 4)) * (7 * 9)
6 * (4 * 9 * 9) * (5 + (5 * 8 * 3 + 3) * 9) * 8 * 3 + 6
(4 + 8 + 9 * 7 * 7) * 5 + (2 + 9 + 5 + 5) + 3 * 7
2 * 6 + 8 * (2 * 6 * 8)
5 + 7 * 2 + 7 + (3 * (7 * 9 + 7) + (9 + 3 * 5 * 9 * 5 + 7) + 8 * 5)
8 + 3 * 9 * 6 + 9 * 2
5 + (6 + (7 + 5 * 4 * 4) + 6)
(3 + 9 + 9 * (6 * 3 * 8 * 9 + 4)) + 2 + 9 * 3 * 8
(2 * (9 + 3 * 8 + 9 + 7) + 4 * 6 * 7) + (9 + 4 * (2 * 8 * 2 + 3 + 6) * (5 * 2) + (6 + 4 + 6 + 5 * 3 * 9) + 8) * (8 * 2 + 9 * 6 * 8) + 6
4 + (4 + (7 + 9) * (7 * 7 * 6 + 3 * 8) + 6 + 9) * 5 * (5 + 9 + (6 * 8 * 8 * 2 * 2 + 6) + 3 + 6 * (4 + 7 * 7))
4 + 8 + 9 + 3
9 * 7 + 9 * 3 * ((8 * 7 * 4 * 5) * 6 * (6 + 6 + 4 + 8 + 4 + 6) * 9 * 6 * (2 * 5 * 8)) + 6
(3 + 8 * 6 + 8 * 9) + 2
7 + 8 + 5 + 3 + 6
2 + 4 + 4 * 4
((8 * 8 * 9 * 4 * 8) + (4 + 6 + 7 * 2 * 4 + 6) + 9) * 4 * 6 * 7
9 * (6 + (3 + 5 + 9 * 6) + 9 * (2 * 2 * 9) * (8 + 5 * 4 + 4 * 2)) * 9 * 6 * 2
4 + 9 + 2 + 6 + 5 * (7 * 3 + 7 * 6 * 7 * 4)
5 + 9 * 6 + (6 * 6 + 8) * (6 * 3 * 3 * 7 * 5)
8 + 9 + (7 + 3 + 4 * 9) + 7
2 + ((5 * 9 + 7 + 4 * 6) * 5 * 3) + 5 * 9 * ((5 * 7 * 2) + 4 + 5 + 4) * 5
8 + (2 * (4 * 3 + 2 * 7 * 6) + 5 * 6) + (7 + 3 + 8 * 9)
3 + (7 * 3 + 6) + (3 * (7 * 5 + 5 * 3 * 7)) + 9 + 6
8 * 9
3 * 3
6 * 7 + 6 + 2 + 6 + (5 + (3 + 4 * 9 * 2 + 9) + 9 + 9 * 3 + 7)
4 * ((4 * 3 + 8 * 3 * 7 + 9) + 3) * (5 + 9 * 6 + (6 + 7)) + 2 + 6 * 7
(2 * 8 + 9 + 2 + 4 + 6) + ((3 + 3 * 7 + 7 + 2 * 9) * 4 + 9)
3 + 4 * 8 + 7 * (3 * 5 + (5 * 6) * 3)
5 * ((2 + 9 * 5 + 3 + 8 * 9) * 8 * 2 + 7 + 5 * 5) + 7 + (7 * 6 * 4 * 2 + 5) * 2
2 * 3 + 2 * 4 * 6 + 2
3 * 4 * 6 + 7 * (4 * (5 * 7 + 8 * 2) * 4 + 5 * 6 + (2 * 2 * 2))
(6 + 5 + (8 + 4 + 2 * 5 + 5 + 4)) + ((8 + 8) + 6 + (4 + 6)) + 7 * (3 + 2 + 9 + 2 * 9) + 8
2 * 6 * 3 + 6 * (4 + (4 + 4 + 7 + 8 + 9) * 8 * (4 + 8 * 4 + 5 + 9) * 7) * 3
8 * 8 + 8 + 3 * 7
4 + 9 * ((9 * 3 + 7 * 2) * 8) * (7 + 5 * 9)
6 + 2 * (7 + (9 + 3 * 4 + 8 * 2) + 7)
4 * 5 + 6 + 6 + 6 * (4 * 8 + 7)
(8 * 2) * (4 * 4 * (8 + 3 * 3 + 8 * 8) * 6 * 4)
2 + (9 + 4 * 7 + 9 * 2 + 8) + 8 + 5 + (4 * 2)
((5 * 3) + 3 * 8 + 9) * (9 * 8) * 3
((9 + 4 * 5 + 8 + 5) * (7 * 4 * 9 + 4 + 7 + 8) * 4 + 9) + 4
3 * (2 * 2 + 9 * (2 + 5 + 6 + 4) * 4) + 6 + ((8 * 3 + 6 + 9) * 5 + 4) + 6 * (9 + (2 + 3 + 8) + (3 + 7 * 3 * 3 * 4 + 8) + 4)
((9 * 2 + 3 * 6) + 2 + 7) + 7 + ((5 + 3 * 4 * 2 + 7 * 9) * 9 + 7 * 2) + (3 + 9 + 5)
2 * 3 + 2 * (2 + 7 * 2 + 3) * 6 * 7
(9 * (4 * 7 * 8 + 9 * 5 + 6) + 6 * 5 + 9) * (6 + 8 + 6 * 9 + 8 + 9) * 9 + 2 + 2
7 + (2 * 4 + 4) * 3 + 8
2 * 6 + ((8 * 7 * 2 * 3 * 5 * 3) + 5 + 5 + 4 * (6 * 7 * 9 + 5) * 7) + 2
8 + (8 + 6 + 8 * 2) * (2 * 9 + 5 + 2)
8 + (8 + (4 + 3 * 6)) * 6 * 6
5 + 5 + (4 + 2 * 8) + 9 * 5
(8 + 6 * 9 * 7 * 9) * (7 * 3) * 8 + 4 + 6
((5 + 4 + 3) * (5 * 4 + 9 * 2 + 9) * 4 * 2 * 3) + 3 + 6
(7 + 9 * 5 + 5) + (8 * 3 * 3 * (6 * 3 + 8 * 6 * 9 + 5)) * 5 + (2 * 8 * 2 * 8)
6 + (9 * 6 + 7 * (3 * 3 + 2) + 4 * 4)
(9 + 4 * (2 * 4) * 5) + 9 * 7 * 7
(7 + 6 + 4 + 6 * 8) * 7 * 3 + (4 * 9) + 3 + 5
6 + 7 + 4 + 7 * 6 * (3 + 5 * (5 + 4 * 9 + 5 + 6) * 7 + 9)
((8 * 5 * 3) + 5 + 3 * 8 + 3) + (7 + 6 * 7 + 6 * (7 * 8 + 4)) + 4 + 8 + 4
8 * 5 + (7 * 4 * 2 + 5 + (3 * 6 + 8 + 3) + (5 * 3))
7 * 9
5 + 6 * 4 * 5 * (2 + 3 + 5 + (4 + 9 + 8 * 8 + 7) * 5 * (7 * 7 * 6 + 5))
5 + 7 * (3 + 6) * ((3 + 6 + 6) * (8 + 6 * 6 * 4 + 7 + 2) + (2 + 9 * 2 * 6 * 3 + 9) + 7 + 7) + ((7 + 3 * 3 * 7 * 4) + 2 * 2) * 7
6 * 3 * ((8 * 8 + 7 * 8 * 4 + 5) + 9)
2 + (7 + (6 + 7 + 9 + 2 * 9 + 2)) + 4 + (4 * 6 * 3) + 3
5 * 6 + (2 + 8 + 9 + (6 + 8 + 3) * 2) + 8
5 + (4 + 8 + 7 * 7 + 3 + 8) * (4 + 4 * 3 + (9 * 5 * 4 * 9) * 9) * 5 * ((6 * 4) * (4 * 6 + 9 + 9 + 8) * (8 + 3 + 2 + 7 * 9) + 6) + 2
3 + (4 + 4 * 6 * 4 + 6 + 3) * (5 + (7 * 6 * 4 * 6 + 9 * 8)) * (9 * 7 + 5 * 2 * 4 + 8) + 9
6 + 3 * (3 + 3 * (9 * 4 * 3 * 9 * 7) * 3 + 4) * 3
4 * 3 * 3 * 4 * 2 + (2 * 3 * 9 * (9 * 9 * 6) * 6)
9 + 4 + (7 + 6 + (3 + 4 * 7 * 3) + (5 * 8 + 9 + 6) * (4 + 8 * 4 + 6) * 8) + 8 * (2 + (5 + 5))
7 + 6 + 5 + 2 + 2 + 8
(6 * 2 + 6 + 5 * (9 * 4)) * 8 + 5
5 + 2 + (6 * 4 + 3 + 3 * 3 + 9) * (9 + 2 * 2) + 9
6 + (9 + 7 * 7 + 3 * 7 * 2) + (8 * 5)
9 * (3 * 6 * 2 + 5) * 4 + 6 * (3 * 8 + 9 * 2 + 8 + 9) + 2
6 + (9 * 2 * 3 * (3 * 8 + 7 * 4))
8 + (6 + 6 + (2 + 8 * 9) + 7 + 6 * (6 + 8 * 2)) * (9 * 7) * 3 + 3
(4 * (6 * 8 + 3 * 8 * 5) * 8 + 3 + (8 * 9 * 3 * 3)) + 5 * 7 * 9 + 9
7 * 5 * (5 * 9 * (6 * 2 * 4) + 8 + 9) * 7 + (8 * 7 + 7 + (7 * 4) + 7) + (7 + 3)
8 * (5 * (5 + 2)) + 5 + 6
8 + 9 * (3 + (3 * 7 * 2))
(5 + 3 * 8 * 7) * 2 + 5 * 9
7 * 4 * 2 * 5 + (7 + 8 + 4 + 9 * 8 * 8) * 3
7 * 8 + 7 * 8 * 2
(9 + (3 + 5 + 3 * 8 + 8 + 3)) + 8 + 6 * 5 + (5 * 2 * 3 + 3 * (2 * 7 * 4 * 4 + 7 + 4)) + (7 + (8 + 4 * 4 + 9 + 8 * 9) * 7)
4 + 4 + 8 + ((5 * 3) * 8 * 3 * 4 * 7) + 5
2 + 9 * (5 * 6 * (7 * 5 + 4 + 7) * (9 * 2 * 7 * 7 * 2 + 7) * 4)
9 + (6 * 6) * 2 + 2 * 8 * 2
4 + 2 * (2 * 9 * 4 * 2 + 9 + 4) + (7 + 4 + 8 + 9) * 8 + 5
7 * ((5 * 9 * 8) * 8) + 6 + (7 * 5 * 4 * 6 * 5) + 6 * 3
7 * 8 * (7 + 3) * 2 * 9 * 2
9 + ((7 * 9 * 7 + 5 * 8 * 9) * 2 + 6 * 9 + (4 * 3 + 8)) + 2 * 9 * 3 + 5
6 * 2 + (7 + (4 + 8 + 7 + 6) * 2 + 7 * (7 + 3 * 8 * 5)) * 6
(9 * 8 + 3 * 9 * 5) * 9 + 3 * 5
8 + (7 + 2) + (9 + 9 * 7 + 3 * 4 + 6)
7 + 8 * 4 + 8 * (8 * 4) * 8
(3 * 4 * 2 * (2 + 2) + 9) + 8 + 9
5 * (9 * 6) + 7
2 + ((9 + 4 * 7) + 5 + 2 + (4 + 4 + 7 + 4 * 4) + 7 * 4) + 6
((6 + 3 + 6 * 8 + 8 * 6) + 5 + 6 + 2 + 7 + 5) * ((9 * 6) + 2 * 3 * 8)
(5 + 7 + 3 + 6 + (9 + 2 + 3) + 4) * 6 * (9 + 5) * 2 + 6
4 * 6 + 4 * (9 * 4 * (3 + 5) * 8 + (7 + 8 + 9 + 3)) + 4 * 4
9 * 4 + 8 + 9 * (4 + 6 + 9 + 2 + 4)
(8 * 5) * 6 * 6 + 8 + 5
(7 + 4 + 7 * 3 + 7 + 7) * 2
9 + 5 * (4 * 2 + 4 * 9 * 3)
5 * 6 * 2 * ((6 + 8 * 5 * 7) + (2 + 7 * 5 + 7) * 7 * 2 * (3 * 2 + 6 * 7 * 3) + 5) + 3 * 3
6 * 2 + 7 + 5 + 5 * ((7 + 2 * 7 + 6 + 2) * 4)
((8 * 3 * 4 * 7 * 8) * 9 * 8 * 2) + 8 * 9 * (3 * 9) * 3 * (2 * 6 + 7 + 9 * 4)
2 + (4 * 4 * 8 * (4 + 8 * 3 * 2 * 9 + 4) * 5 * (7 * 7 + 3 + 2 + 3)) * 5 + 6
5 + 5 + 9 * (8 * 9 + 9 * 9 + (2 + 9) + 9) * 2
(5 * 6 + 8) + 9 + (2 * 6 + 5 + 5 * (3 + 9 + 2) + 2) + 3 * ((3 * 7) + 8 * 5) + (3 + 2)
(9 * 7 + 4 + 2 * (2 * 2 + 3 + 8)) + 6
4 * 5 * ((7 * 7 * 2 + 5 + 9 * 4) * (9 + 4 * 7 + 8) * 8 * 3)
9 * 4 + (8 + (4 + 8 + 3) * 6 * 9 + 3 * 7) * 5 * 4
8 + (7 + 3 * (8 * 6 * 8 + 7 * 8) + (6 * 2 * 3 + 7 * 9))
(6 * 4 * 4 * 8 + (6 + 6)) * 8 + 2 + 4 + 9 * 7
7 + (9 + 5 * (2 + 5 + 8 + 2 * 2 * 4) * 2 + 6) + 5 + 6 + 3 * (6 * (9 + 3))
2 * (8 * 4) * 7 + 3 * 9
3 + 6 + 2 + 6 + 3 * (3 + (2 * 6 + 6 * 3) * 6 + 2 * (7 * 7 + 7))
6 + 2 * 7 * (8 * 5 * 8 * 9 + 8)
2 * 8 + 3 * (5 * 6 + 3 * 2 * 5 * 8)
(7 + (5 * 8 + 5 + 4 + 9) + 4 + 3 * 3 * (9 + 5 * 3 * 7 * 6)) + 3 + (6 * 2 * 3 * 4) + ((2 * 2) + 9 + (9 + 3 + 6 * 9 * 4 + 4) + 8)
(8 * 5 + (3 + 3 * 5 + 9 * 2) + 4 + (5 + 5 + 7 * 3) + 6) * ((2 + 2 * 4) * 3 + 5)
9 * 9 * (3 + 2 + 7 * 2 + 6 * 5) + 2 + 4 * (2 + 3 * (8 * 4) + 2)
(6 * 8 + (5 * 7 * 6 + 3 + 4 * 4) + (6 + 7) + 9) * (7 * 7 * 5 + 4) + 2 + 9 * 9
4 * 9 * 2 * (7 * 3 * (6 + 8 * 5))
8 + 2 + 2 * (7 * 9 + 4 + (4 + 5 + 3 * 7 + 4 + 8)) + 4 * 2
(4 + (7 * 3) * 8 * 2) * 5 * 9 + 9 * 4 + 3
(6 * 6 + (8 * 4 + 3 + 2 * 6) * 6) * 5 * 2 + 7 * (6 + (7 * 3)) + (6 + 6 * (6 * 9 * 8 + 3 * 3) * 3)
(8 + 8 + 5 * 6) + 7 + 4 + ((9 + 7) * (2 * 9 * 2) * 8) + 5
5 * 9 * (8 * (9 * 8 * 4) * 9 * 8)
7 + (3 * 6 + (6 * 2 * 9) * 2)
7 + 9 + 6 + (6 + 4 + 7) + 4
3 + 6 * 3
2 * 7 * 7 + (2 + 2 + 7 + (5 * 9 * 2 * 7 * 9)) + 5 * 9
2 * (2 + 7 * 2 * (5 + 7 + 7) * 7) + 8 + 5
9 + 7 + 5
8 + 8 + 3 * 2 * 5 * (5 * (6 + 9) * (4 * 5 * 8 * 2 + 4))
7 + 8 * (6 * 8) * 8 * (3 * 2 + 8 + 5 + 8 + 8)
7 * 8 * (2 * 3 + 9 * 6) * 8 + ((9 * 3 + 9 + 3 * 9 + 6) + 7 * 2 + 9 + (7 + 3 * 9 * 6) * (5 * 6 + 4 * 2 + 4 + 3))
3 * 9 + (4 * 6 + 3 * 3 + 4 * (5 * 5))
((5 * 9 + 7 + 9) + 4 + 9 + 8 + 8 + 5) * 9 + (8 + 7 * (4 * 5 + 5 * 7 + 7)) + 4 * 2 * ((4 + 9 * 6 + 2) * 3 * 2 * 8 * 9)
4 + 6 + 3 * (8 + 9 + 9) * 7 + 5
((5 * 8 * 9 * 9 + 5) + (5 * 8 * 5 * 3 * 8) * 5 * 3 + (6 * 6 * 8 + 2 + 5 + 6)) + 2 * 6 + 3 + 3 + 2
7 * 6 * 3 + (6 + 7 * 3 + (7 * 8 * 7 + 9 * 7) * 7) * 7 + 4
(7 * 4 + 8 + 8) * (3 * 9 + 9 + 9 + 9 + 7) * (6 * 6 * 3 + 8 * (6 + 3 + 8 + 7 + 3)) * 2 + 5
(9 + 3) * 5 * 9
((2 + 2 + 7 * 2 * 4) + 5 + 2 + 5) * 5 + 2 * 7 + (6 * (6 + 8 * 5 + 6) * 5 * 6 * 3) + 8
(8 * 4 * 9 + 7) + (8 + 4)
8 * (4 + 5 + (9 * 2 + 8 + 4 * 8) * 2) + 8
(9 + 9) + 5
7 * (7 + 3 + 7) + 5 + 8 * 3
(6 * 6 * (3 + 4 + 6 + 7 + 5 * 2) * 4 * 2 * 7) + 2 + (4 * 8 + 8) + 2 + 7
2 + 5 * 5 + ((3 * 7 + 5 + 2 * 3) + 5 * 2 * 6 * 8)
9 + 9 * (2 + (9 + 4) + 9) * 8 + 3
7 * 5
7 + 5 * 7 * (9 * 8 * 7 * 5) * 8
(8 + (5 + 2 + 8 * 7 + 8)) + 3 + 2 + 9 + (8 + (9 * 4 + 4 * 4 * 3) + 4 * 4) + 9
(7 * 8) + (6 + 4 * 9 + 8 + 4 + (6 + 6 * 8)) * (2 + 6 * 3 + (3 * 2 * 7 + 2) * 5 + (4 * 4 + 7 + 4)) * 9
3 * 9 * ((3 * 7 * 8 * 9) + 7 * (7 + 5 * 9 * 7) * 8 + 4 + 2) + 9 * 5
(8 * 6 * 3 + (9 + 5)) * 4 + 9 * 7 + ((6 * 7) + 9 + 8 + 9) + 3
8 * ((4 + 8) * 7 + 8 + 2 * 8) * 5 * 7 + (6 + 3 * 6) * 9
((6 + 2 * 7 + 2 + 9 * 5) * 5 * 9 + (7 * 7)) * 9 * 2 * 6 + 4
8 + (7 * 8 + 6 * 8 * 5) + 2 * (4 + 4 * 9 + 9 + 9) * 3 * 8
9 * ((9 + 3) * 3 * 3 * 5 * 2 * 5) * 5 * (8 * 3 * (6 * 4 * 6 * 7 * 9) + 2)
9 + 6 + 7 + 6 * ((2 + 8 * 6 + 7 + 6) + 6 + (9 + 9 * 9 + 5)) * 8
(7 * 4 + (4 + 5 + 2 + 3)) + 7 * 7 * 3 + 3 * (5 + 4 + 2 + 3)
(4 * 6) + 6 * 8 + 3 + 8 + (5 * 7 * 5)
(9 + 2 * (6 + 4 * 9 + 7) * 3) * 4
(8 + 2 * 4 * 7 * (9 * 4) + 4) * (3 + 4 * 5) + (4 * 2 * 8 * (6 + 6 + 9) + 3 * (6 + 6 + 7 + 3)) * 3
3 * 5 + 6
5 + 2 + 8 * 6 + (4 + (9 * 3) * (6 + 9 * 6 + 3 * 7) + 4 * 5 + 8)
(3 + 5 * (6 * 2 * 5 + 3 + 2)) * 8 * 7 * 7
7 * 2 + 8 * ((6 + 2 * 6 + 8 + 9 + 3) + 6 * 4)
7 + 7 + 3 * 6 + ((2 * 9 * 2 * 5 + 6) + 6 + 6)
(9 + 4) + 3 + 5 * ((9 + 4 * 8 * 5) + 7 + 3 + 6 * 6 + (5 * 5 + 6))
(8 + 4 + 2) + (7 + 2)
6 * (4 * 9 + 7 + 8 * (8 + 8)) + 5 + 6 + 5 * 9
3 * (4 + 8 + 2 + 2) * 9 + 5 * 4
(5 * 8 * (9 * 4 * 7 + 3 * 3 * 8) + 5) * (6 + (3 * 3 * 3 * 4)) + 4 * ((7 * 3 * 4 + 3) * 4 * 8 * (9 + 3) + 4)
4 * ((8 * 8 + 6 + 7 * 6) * 9 * 5 * 5 + 4)
(3 + 7 * 8 + (7 * 7 * 7 * 7)) * 6 + 8 + (8 + 5) + 8 + 5
(7 * (8 * 3 + 3 + 8) + 2 * 8 + 6) * 8 + 6
4 + (3 * 6 + 5) * (6 + 3 + (6 + 8 + 6 * 9 * 8) * (3 * 6) + 6 * 3)
6 + (9 * (7 + 9 * 6 + 9 + 8)) * 9 * 8 * 5 * (9 + (3 + 4 + 2 + 9 + 6) * (3 * 5 * 2 + 6 * 8 * 6) + (8 + 7 * 6) + (7 * 4 * 8 * 7 + 3 * 9) + (3 + 7 + 9))
3 + 9 * 7 + ((9 + 8 + 3 * 2) + 6 + 5 + (4 + 8 * 5 * 4 * 7) + (9 * 3 * 8))
4 * 2 * 2 + (6 + 8) + 6
3 * 7 * (8 * 3)
5 + 5 + 6 + (2 * 5 * 5) + (6 + 6)
9 * 6 * (3 + (6 * 2 + 4 + 8 * 3) * 6 + (3 * 9 * 9 + 9)) * 9
(3 * 3) * 8 * 2 * 3 * 2 * 8
((4 * 5 * 3) * 8 * (9 * 2 + 4 * 3 * 7 * 6) * 9) * (6 + 6 + 6 * 8) + 8 + 5 * 2
7 * (5 * 7 + 4 + 8) * 2 * 4
(6 + 3 * (2 * 8 + 4 + 2 + 9 + 6) + 7) * 3 * 2 * 9
9 * (8 + 9 + (9 + 8) * 5) + (4 + 3 * 7 + 5 + (6 * 2 + 5)) * 2
(5 * 7 + 6 + (3 + 3) + 9) + 6
8 + (6 * (6 + 7 * 3))
(7 + 7) * 3 + 5 * (9 + 7 + (5 + 9 + 2) * 3 * 7)
4 * (3 + 8 + 2 + 3) * 8 + (7 * 6 * 9 + (4 * 7 * 5) * 3 * 3) * 5 + 7
4 + 7 * (9 + 5 * 2 * 2 * 2 + 7) * 2
4 + 3 + 5 + (6 * (5 * 7 + 2 * 7)) + 4 + (2 * 8 + (3 + 2 + 3 * 2) * 5 * (2 * 2))
6 + 6 + 2 + 6 + (9 * 2 * 9)
"#;
