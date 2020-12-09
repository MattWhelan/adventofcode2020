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
    pub fn exec(&self, m: &mut RegisterFile) {
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

    pub fn parse_prog(input: &str) -> Vec<Self> {
        input.lines()
            .map(|l| l.parse().expect("Could not parse instruction"))
            .collect()
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
pub struct RegisterFile {
    pub ip: usize,
    pub acc: isize,
}

impl RegisterFile {
    pub fn new() -> Self {
        Self { acc: 0, ip: 0 }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Machine {
    reg: RegisterFile
}

pub trait Watcher {
    fn log(&mut self, ins: &Instruction, reg: &RegisterFile);
    fn check_abort(&self, ins: &Instruction, reg: &RegisterFile) -> bool;

    fn dump_log<'a, L: IntoIterator<Item=&'a Instruction>>(log: L) {
        for ins in log {
            println!("{}", ins);
        }
    }
}

impl Machine {
    pub fn new() -> Self {
        Self {
            reg: RegisterFile::new()
        }
    }

    pub fn run_debug<W: Watcher>(&mut self, prog: &[Instruction], watcher: &mut W) -> Result<RegisterFile, RegisterFile> {
        while let Some(ins) = prog.get(self.reg.ip) {
            if watcher.check_abort(ins, &self.reg) {
                return Err(self.reg);
            }

            watcher.log(ins, &self.reg);
            ins.exec(&mut self.reg);
        }

        if self.reg.ip == prog.len() {
            Ok(self.reg)
        } else {
            Err(self.reg)
        }
    }

    pub fn run(&mut self, prog: &[Instruction]) -> Result<RegisterFile, RegisterFile> {
        while let Some(ins) = prog.get(self.reg.ip) {
            ins.exec(&mut self.reg);
        }

        if self.reg.ip == prog.len() {
            Ok(self.reg)
        } else {
            Err(self.reg)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Instruction, Machine, Watcher, RegisterFile};

    const PROG: &str = r#"nop +0
acc +1
jmp +4
acc +3
jmp -3
acc -99
acc +1
nop -4
acc +6"#;

    #[test]
    fn test_parse_run() {
        let prog = Instruction::parse_prog(PROG);
        let mut m = Machine::new();
        if let Ok(reg) = m.run(&prog) {
            assert_eq!(8, reg.acc);
        } else {
            panic!("Failed to run program");
        }
    }

    #[test]
    fn test_run_debug() {
        let prog = Instruction::parse_prog(PROG);
        let mut m = Machine::new();

        struct CountWatch(i32);
        impl Watcher for CountWatch {
            fn log(&mut self, _: &Instruction, _: &RegisterFile) {
                self.0 += 1
            }

            fn check_abort(&self, _: &Instruction, _: &RegisterFile) -> bool {
                false
            }
        }

        let mut w = CountWatch(0);
        if let Ok(reg) = m.run_debug(&prog, &mut w) {
            assert_eq!(8, reg.acc);
            assert_eq!(6, w.0);
        } else {
            panic!("Failed to run program");
        }
    }
}
