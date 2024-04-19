use thiserror::Error;

use crate::codegen::{Instruction, Pc};

/// String pointer.
/// This is used to point to the current character in the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Sp(usize);

impl Sp {
    fn inc<E>(&mut self, err: impl Fn() -> E) -> Result<Self, E> {
        if let Some(new) = self.0.checked_add(1) {
            self.0 = new;
            Ok(*self)
        } else {
            Err(err())
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MatchError {
    #[error("program counter overflow occured")]
    PcOverflow,
    #[error("stack pointer overflow occured")]
    SpOverflow,
    #[error("instruction not found")]
    InstructionNotFound,
}

/// Virtual machine for regular expression matching.
#[derive(Debug)]
pub struct Machine {
    instructions: Vec<Instruction>,
}

impl Machine {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    pub fn is_match(&self, text: &[char]) -> Result<bool, MatchError> {
        self.is_matching(text, Pc(0), Sp(0))
    }

    fn is_matching(&self, text: &[char], mut pc: Pc, mut sp: Sp) -> Result<bool, MatchError> {
        loop {
            let instruction = if let Some(i) = self.instructions.get(pc.0) {
                i
            } else {
                return Err(MatchError::InstructionNotFound);
            };

            match *instruction {
                Instruction::Char(c) => {
                    let Some(cc) = text.get(sp.0) else {
                        return Ok(false);
                    };
                    if c == *cc {
                        pc.inc(|| MatchError::PcOverflow)?;
                        sp.inc(|| MatchError::SpOverflow)?;
                    } else {
                        return Ok(false);
                    }
                }
                Instruction::Match => return Ok(true),
                Instruction::Jmp(new_pc) => pc = new_pc,
                Instruction::Split(l1, l2) => {
                    if self.is_matching(text, l1, sp)? || self.is_matching(text, l2, sp)? {
                        return Ok(true);
                    } else {
                        return Ok(false);
                    }
                }
                Instruction::AnyByte => {
                    // The dot matches any character, but does not usually match an empty character.
                    if text.get(sp.0).is_some() {
                        pc.inc(|| MatchError::PcOverflow)?;
                        sp.inc(|| MatchError::SpOverflow)?;
                    } else {
                        return Ok(false);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! chars {
        ($s:expr) => {
            &$s.chars().collect::<Vec<_>>()
        };
    }

    #[test]
    fn concat() {
        let machine = Machine::new(vec![
            Instruction::Char('a'),
            Instruction::Char('b'),
            Instruction::Char('c'),
            Instruction::Match,
        ]);
        assert!(machine.is_match(chars!("abc")).unwrap());
        assert!(!machine.is_match(chars!("")).unwrap());
    }

    #[test]
    fn or() {
        // ab(cd|ef|g)h
        let machine = Machine::new(vec![
            /*     : 0 */ Instruction::Char('a'),
            /*     : 1 */ Instruction::Char('b'),
            /*     : 2 */ Instruction::Split(Pc(3), Pc(6)), // L1, L2
            /* L1  : 3 */ Instruction::Char('c'),
            /*     : 4 */ Instruction::Char('d'),
            /*     : 5 */ Instruction::Jmp(Pc(11)), // L3
            /* L2  : 6 */ Instruction::Split(Pc(7), Pc(10)), // L4, L5
            /* L4  : 7 */ Instruction::Char('e'),
            /*     : 8 */ Instruction::Char('f'),
            /*     : 9 */ Instruction::Jmp(Pc(11)), // L6
            /* L5  :10 */ Instruction::Char('g'),
            /* L6,3:11 */ Instruction::Char('h'),
            /*     :12 */ Instruction::Match,
        ]);
        assert!(machine.is_match(chars!("abcdh")).unwrap());
        assert!(machine.is_match(chars!("abgh")).unwrap());
        assert!(!machine.is_match(chars!("abh")).unwrap());
        assert!(!machine.is_match(chars!("")).unwrap());
    }

    #[test]
    fn question() {
        // a?b
        let machine = Machine::new(vec![
            /*   :0 */ Instruction::Split(Pc(1), Pc(2)),
            /* L1:1 */ Instruction::Char('a'),
            /* L2:2 */ Instruction::Char('b'),
            /*   :3 */ Instruction::Match,
        ]);
        assert!(machine.is_match(chars!("b")).unwrap());
        assert!(machine.is_match(chars!("ab")).unwrap());
        assert!(!machine.is_match(chars!("aab")).unwrap());
        assert!(!machine.is_match(chars!("xc")).unwrap());
        assert!(!machine.is_match(chars!("")).unwrap());
    }

    #[test]
    fn star() {
        // a*b
        let machine = Machine::new(vec![
            /* L1:0 */ Instruction::Split(Pc(1), Pc(3)), // L2, L3
            /* L2:1 */ Instruction::Char('a'),
            /*   :2 */ Instruction::Jmp(Pc(0)), // L1
            /* L3:3 */ Instruction::Char('b'),
            /*   :4 */ Instruction::Match,
        ]);
        assert!(machine.is_match(chars!("b")).unwrap());
        assert!(machine.is_match(chars!("ab")).unwrap());
        assert!(machine.is_match(chars!("aab")).unwrap());
        assert!(!machine.is_match(chars!("xb")).unwrap());
        assert!(!machine.is_match(chars!("")).unwrap());
    }

    #[test]
    fn plus() {
        // a+b
        let machine = Machine::new(vec![
            /* L1:0 */ Instruction::Char('a'),
            /*   :1 */ Instruction::Split(Pc(0), Pc(2)), // L1, L2
            /* L2:2 */ Instruction::Char('b'),
            /*   :3 */ Instruction::Match,
        ]);
        assert!(machine.is_match(chars!("ab")).unwrap());
        assert!(machine.is_match(chars!("aab")).unwrap());
        assert!(machine.is_match(chars!("aaab")).unwrap());
        assert!(!machine.is_match(chars!("b")).unwrap());
        assert!(!machine.is_match(chars!("xb")).unwrap());
        assert!(!machine.is_match(chars!("")).unwrap());
    }

    #[test]
    fn dot() {
        // .
        let machine = Machine::new(vec![
            /*   :0 */ Instruction::AnyByte,
            /*   :1 */ Instruction::Match,
        ]);
        assert!(machine.is_match(chars!("a")).unwrap());
        assert!(machine.is_match(chars!("b")).unwrap());
        assert!(machine.is_match(chars!("abc")).unwrap());
        assert!(!machine.is_match(chars!("")).unwrap());

        // a.b
        let machine = Machine::new(vec![
            /*   :0 */ Instruction::Char('a'),
            /*   :1 */ Instruction::AnyByte,
            /*   :2 */ Instruction::Char('b'),
            /*   :3 */ Instruction::Match,
        ]);
        assert!(machine.is_match(chars!("axb")).unwrap());
        assert!(machine.is_match(chars!("ayb")).unwrap());
        assert!(!machine.is_match(chars!("ab")).unwrap());
        assert!(!machine.is_match(chars!("")).unwrap());
    }
}
