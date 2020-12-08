use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Instruction {
    Acc(isize),
    Jmp(isize),
    Nop(isize),
}

impl Instruction {
    pub fn exec(&self, m: &mut Machine) {
        match self {
            Instruction::Nop(_) => m.ip += 1,
            Instruction::Acc(x) => {
                m.acc += x;
                m.ip += 1;
            }
            Instruction::Jmp(x) => {
                m.ip = (m.ip as isize + x) as usize;
            }
        };
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Acc(x) => write!(f, "acc {}", x),
            Instruction::Jmp(x) => write!(f, "jmp {}", x),
            Instruction::Nop(x) => write!(f, "nop {}", x),
        }
    }
}

impl FromStr for Instruction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, " ");
        let op = parts
            .next()
            .ok_or_else(|| Self::Err::msg("Missing instruction"))?;
        let arg = parts.next().unwrap().parse()?;
        match op {
            "acc" => Ok(Instruction::Acc(arg)),
            "jmp" => Ok(Instruction::Jmp(arg)),
            "nop" => Ok(Instruction::Nop(arg)),
            s => Err(Self::Err::msg(format!("Bad input {}", s))),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Machine {
    acc: isize,
    ip: usize,
}

impl Machine {
    pub fn new() -> Self {
        Self { acc: 0, ip: 0 }
    }

    pub fn dump_ins(log: &[&Instruction]) {
        for ins in log {
            println!("{}", ins);
        }
    }

    pub fn discover_loop(&mut self, prog: &[Instruction]) -> Result<isize, (isize, usize)> {
        let mut ip_set: HashSet<usize> = HashSet::new();
        let mut ins_list = Vec::new();

        while self.ip < prog.len() {
            if ip_set.contains(&self.ip) {
                // Self::dump_ins(&ins_list);
                return Err((self.acc, self.ip));
            }
            ip_set.insert(self.ip);

            let ins = &prog[self.ip];
            ins_list.push(ins);
            ins.exec(self);
        }

        if self.ip == prog.len() {
            Ok(self.acc)
        } else {
            // Self::dump_ins(&ins_list);
            return Err((self.acc, self.ip));
        }
    }

    pub fn run(&mut self, prog: &[Instruction]) -> (isize, usize) {
        while self.ip < prog.len() {
            let ins = &prog[self.ip];
            ins.exec(self);
        }

        (self.acc, self.ip)
    }

    pub fn get_acc(&self) -> isize {
        self.acc
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
