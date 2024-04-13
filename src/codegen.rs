use thiserror::Error;

use crate::parser::Ast;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Pc(usize);

impl Pc {
    fn inc(&mut self) -> Result<Self, GenerateCodeError> {
        if let Some(new) = self.0.checked_add(1) {
            self.0 = new;
            Ok(*self)
        } else {
            Err(GenerateCodeError::PcOverflow)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Instruction {
    Char(char),
    Match,
    Jmp(Pc),
    Split(Pc, Pc),
}

#[derive(Error, Debug)]
enum GenerateCodeError {
    #[error("program counter overflow occured")]
    PcOverflow,
}

#[derive(Debug, Default)]
struct CodeGenerator {
    pc: Pc,
    instractions: Vec<Instruction>,
}

impl CodeGenerator {
    fn generate_code(mut self, ast: Ast) -> Result<Vec<Instruction>, GenerateCodeError> {
        self.expr(ast)?;
        self.instractions.push(Instruction::Match);
        Ok(self.instractions)
    }

    fn expr(&mut self, ast: Ast) -> Result<(), GenerateCodeError> {
        match ast {
            Ast::Char(c) => self.char(c)?,
            Ast::Concat(concat) => self.concat(concat)?,
            Ast::Or(lhs, rhs) => self.or(*lhs, *rhs)?,
            Ast::Question(e) => self.question(*e)?,
            Ast::Star(e) => self.star(*e)?,
            Ast::Plus(e) => self.plus(*e)?,
        };
        Ok(())
    }

    /// Generate char instruction.
    fn char(&mut self, c: char) -> Result<(), GenerateCodeError> {
        self.instractions.push(Instruction::Char(c));
        self.pc.inc()?;
        Ok(())
    }

    /// Generate code for Concatenation operator.
    ///
    /// e1e2
    /// ```txt
    /// e1
    /// e2
    /// ```
    fn concat(&mut self, concat: Vec<Ast>) -> Result<(), GenerateCodeError> {
        for ast in concat {
            self.expr(ast)?;
        }
        Ok(())
    }

    /// Generate code for OR operator.
    ///
    /// e1|e2
    /// ```txt
    ///     split L1, L2
    /// L1: e1 code
    ///     jmp L3
    /// L2: e2 code
    /// L3:
    /// ```
    fn or(&mut self, lhs: Ast, rhs: Ast) -> Result<(), GenerateCodeError> {
        assert_eq!(self.instractions.len(), self.pc.0);

        let split_pc = self.pc;
        // split L1, L2
        let l1 = self.pc.inc()?;
        self.instractions.push(Instruction::Split(l1, Pc(0))); // L2 TBD.
        assert_eq!(self.instractions.len(), self.pc.0);

        // e1
        self.expr(lhs)?;
        // jmp L3
        let jmp_pc = self.pc;
        self.pc.inc()?;
        self.instractions.push(Instruction::Jmp(Pc(0))); // L3 TBD.
        assert_eq!(self.instractions.len(), self.pc.0);

        if let Some(Instruction::Split(_, l2)) = self.instractions.get_mut(split_pc.0) {
            *l2 = self.pc;
        } else {
            unreachable!(
                "instruction at PC {} should be Instruction::Split",
                split_pc.0
            );
        }

        // e2
        self.expr(rhs)?;
        assert_eq!(self.instractions.len(), self.pc.0);

        if let Some(Instruction::Jmp(l3)) = self.instractions.get_mut(jmp_pc.0) {
            *l3 = self.pc;
        } else {
            unreachable!("instruction at PC {} should be Instruction::Jmp", jmp_pc.0);
        }

        Ok(())
    }

    /// Generate code for Question operator.
    ///
    /// e?
    /// ```txt
    ///     split L1, L2
    /// L1: e code
    /// L2:
    /// ```
    fn question(&mut self, e: Ast) -> Result<(), GenerateCodeError> {
        assert_eq!(self.instractions.len(), self.pc.0);

        let split_pc = self.pc;
        let l1 = self.pc.inc()?;
        self.instractions.push(Instruction::Split(l1, Pc(0))); // L2 TBD.
        self.expr(e)?;
        assert_eq!(self.instractions.len(), self.pc.0);

        if let Some(Instruction::Split(_, l2)) = self.instractions.get_mut(split_pc.0) {
            *l2 = self.pc;
        } else {
            unreachable!(
                "instruction at PC {} should be Instruction::Split",
                split_pc.0
            );
        }

        Ok(())
    }

    /// Generate code for Start operator.
    ///
    /// e*
    /// ```txt
    /// L1: split L2, L3
    /// L2: e code
    ///     jmp L1
    /// L3:
    /// ```
    fn star(&mut self, e: Ast) -> Result<(), GenerateCodeError> {
        assert_eq!(self.instractions.len(), self.pc.0);

        let l1 = self.pc;
        let l2 = self.pc.inc()?;
        self.instractions.push(Instruction::Split(l2, Pc(0))); // L3 TBD
        self.expr(e)?;
        assert_eq!(self.instractions.len(), self.pc.0);

        self.pc.inc()?;
        self.instractions.push(Instruction::Jmp(l1));
        assert_eq!(self.instractions.len(), self.pc.0);

        if let Some(Instruction::Split(_, l3)) = self.instractions.get_mut(l1.0) {
            *l3 = self.pc;
        } else {
            unreachable!("instruction at PC {} should be Instruction::Split", l1.0);
        }

        Ok(())
    }

    /// Generate code for Plus operator.
    ///
    /// e+
    /// ```txt
    /// L1: e code
    ///     split L1, L2
    /// L2:
    /// ```
    fn plus(&mut self, e: Ast) -> Result<(), GenerateCodeError> {
        assert_eq!(self.instractions.len(), self.pc.0);

        let l1 = self.pc;
        self.expr(e)?;
        assert_eq!(self.instractions.len(), self.pc.0);

        let l2 = self.pc.inc()?;
        self.instractions.push(Instruction::Split(l1, l2));
        assert_eq!(self.instractions.len(), self.pc.0);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn concat() {
        let gen = CodeGenerator::default();
        let ast = Ast::Concat(vec![Ast::Char('a'), Ast::Char('b'), Ast::Char('c')]);
        assert_eq!(
            gen.generate_code(ast).unwrap(),
            vec![
                Instruction::Char('a'),
                Instruction::Char('b'),
                Instruction::Char('c'),
                Instruction::Match,
            ]
        );
    }

    #[test]
    fn or() {
        // a|b
        let gen = CodeGenerator::default();
        let ast = Ast::Or(Ast::Char('a').into(), Ast::Char('b').into());
        assert_eq!(
            gen.generate_code(ast).unwrap(),
            vec![
                /*   :0 */ Instruction::Split(Pc(1), Pc(3)), // L1, L2
                /* L1:1 */ Instruction::Char('a'),
                /*   :2 */ Instruction::Jmp(Pc(4)), // L3
                /* L2:3 */ Instruction::Char('b'),
                /* L3:4 */ Instruction::Match,
            ]
        );

        // ab(cd|ef|g)h
        let gen = CodeGenerator::default();
        let ast = Ast::Concat(vec![
            Ast::Char('a'),
            Ast::Char('b'),
            Ast::Or(
                Ast::Concat(vec![Ast::Char('c'), Ast::Char('d')]).into(),
                Ast::Or(
                    Ast::Concat(vec![Ast::Char('e'), Ast::Char('f')]).into(),
                    Ast::Char('g').into(),
                )
                .into(),
            ),
            Ast::Char('h'),
        ]);
        assert_eq!(
            gen.generate_code(ast).unwrap(),
            vec![
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
            ]
        );
    }

    #[test]
    fn question() {
        // a?b
        let gen = CodeGenerator::default();
        let ast = Ast::Concat(vec![Ast::Question(Ast::Char('a').into()), Ast::Char('b')]);
        assert_eq!(
            gen.generate_code(ast).unwrap(),
            vec![
                /*   :0 */ Instruction::Split(Pc(1), Pc(2)),
                /* L1:1 */ Instruction::Char('a'),
                /* L2:2 */ Instruction::Char('b'),
                /*   :3 */ Instruction::Match,
            ]
        );
    }

    #[test]
    fn star() {
        // a*b
        let gen = CodeGenerator::default();
        let ast = Ast::Concat(vec![Ast::Star(Ast::Char('a').into()), Ast::Char('b')]);
        assert_eq!(
            gen.generate_code(ast).unwrap(),
            vec![
                /* L1:0 */ Instruction::Split(Pc(1), Pc(3)), // L2, L3
                /* L2:1 */ Instruction::Char('a'),
                /*   :2 */ Instruction::Jmp(Pc(0)), // L1
                /* L3:3 */ Instruction::Char('b'),
                /*   :4 */ Instruction::Match,
            ]
        );
    }

    #[test]
    fn plus() {
        // a+b
        let gen = CodeGenerator::default();
        let ast = Ast::Concat(vec![Ast::Plus(Ast::Char('a').into()), Ast::Char('b')]);
        assert_eq!(
            gen.generate_code(ast).unwrap(),
            vec![
                /* L1:0 */ Instruction::Char('a'),
                /*   :1 */ Instruction::Split(Pc(0), Pc(2)), // L1, L2
                /* L2:2 */ Instruction::Char('b'),
                /*   :3 */ Instruction::Match,
            ]
        );
    }
}
