use std::mem;

use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum Ast {
    Char(char),
    Concat(Vec<Ast>),
    Or(Box<Ast>, Box<Ast>),
    Question(Box<Ast>),
    Star(Box<Ast>),
    Plus(Box<Ast>),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseError {
    #[error("missing operand")]
    MissingOperand,
    #[error("empty expression")]
    Empty,
}

/// Extract `concat` as an operand of the Or operator and append it to `concat_or`.
fn append_concat(concat: &mut Vec<Ast>, concat_or: &mut Vec<Ast>) {
    if concat.len() == 1 {
        let c = concat.pop().unwrap();
        concat_or.push(c);
    } else {
        concat_or.push(Ast::Concat(mem::take(concat)));
    }
}

/// Construct an AST for the Or operator.
fn or_ast(mut concat_or: Vec<Ast>) -> Option<Ast> {
    if let Some(mut ast) = concat_or.pop() {
        // There is no the Or operator at top level. e.g. ab(c|d)ef.
        if concat_or.is_empty() {
            return Some(ast);
        }

        concat_or.reverse();
        for operand in concat_or {
            ast = Ast::Or(Box::new(operand), Box::new(ast));
        }

        Some(ast)
    } else {
        None
    }
}

/// Parse a regular expression pattern into an abstraction syntax tree (AST).
pub fn parse(pattern: &str) -> Result<Ast, ParseError> {
    let mut concat = Vec::new();
    let mut concat_or = Vec::new();

    for c in pattern.chars() {
        match c {
            '|' => {
                if concat.is_empty() {
                    return Err(ParseError::MissingOperand);
                }

                // Append the left operand to `concat_or`.
                append_concat(&mut concat, &mut concat_or);
            }
            '?' => todo!(),
            '*' => todo!(),
            '+' => todo!(),
            '(' => todo!(),
            ')' => todo!(),
            _ => concat.push(Ast::Char(c)),
        }
    }

    if concat.is_empty() {
        // Despite the presence of the Or operator, the right operand is missing.
        if !concat_or.is_empty() {
            return Err(ParseError::MissingOperand);
        }
    } else {
        // After going through all characters, append the right(=last) operand to `concat_or`.
        append_concat(&mut concat, &mut concat_or);
    }

    if let Some(ast) = or_ast(concat_or) {
        Ok(ast)
    } else {
        Err(ParseError::Empty)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn concat_or() {
        // Concat operation
        let ast = Ast::Concat(vec![Ast::Char('a'), Ast::Char('b'), Ast::Char('c')]);
        assert_eq!(parse("abc").unwrap(), ast);

        // Or operation
        let ast = Ast::Or(
            Ast::Char('a').into(),
            Ast::Or(Ast::Char('b').into(), Ast::Char('c').into()).into(),
        );
        assert_eq!(parse("a|b|c").unwrap(), ast);

        let ast = Ast::Or(
            Ast::Concat(vec![Ast::Char('x'), Ast::Char('y'), Ast::Char('z')]).into(),
            Ast::Or(Ast::Char('b').into(), Ast::Char('c').into()).into(),
        );
        assert_eq!(parse("xyz|b|c").unwrap(), ast);

        assert_eq!(parse("|b"), Result::Err(ParseError::MissingOperand));
        assert_eq!(parse("a|"), Result::Err(ParseError::MissingOperand));
        assert_eq!(parse("|"), Result::Err(ParseError::MissingOperand));

        // Empty expression
        assert_eq!(parse(""), Result::Err(ParseError::Empty));
    }
}
