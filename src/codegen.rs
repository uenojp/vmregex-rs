use thiserror::Error;

use crate::parser::Ast;

#[derive(Debug, Default, PartialEq, Eq)]
struct Pc(usize);

impl Pc {
    fn inc(&mut self) -> Result<(), GenerateCodeError> {
        if let Some(new) = self.0.checked_add(1) {
            self.0 = new;
            Ok(())
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
    Split(Pc),
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
        Ok(self.instractions)
    }

    fn expr(&mut self, ast: Ast) -> Result<(), GenerateCodeError> {
        match ast {
            Ast::Char(c) => self.char(c)?,
            Ast::Concat(concat) => self.concat(concat)?,
            Ast::Or(_, _) => todo!(),
            Ast::Question(_) => todo!(),
            Ast::Star(_) => todo!(),
            Ast::Plus(_) => todo!(),
        };
        Ok(())
    }

    fn char(&mut self, c: char) -> Result<(), GenerateCodeError> {
        self.instractions.push(Instruction::Char(c));
        self.pc.inc()?;
        Ok(())
    }

    fn concat(&mut self, concat: Vec<Ast>) -> Result<(), GenerateCodeError> {
        for ast in concat {
            self.expr(ast)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gen_char() {
        let gen = CodeGenerator::default();
        let ast = Ast::Concat(vec![Ast::Char('a'), Ast::Char('b'), Ast::Char('c')]);
        assert_eq!(
            gen.generate_code(ast).unwrap(),
            vec![
                Instruction::Char('a'),
                Instruction::Char('b'),
                Instruction::Char('c')
            ]
        );
    }
}
