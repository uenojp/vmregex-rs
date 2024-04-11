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
    #[error("unclosed parenthesis")]
    UnclosedParenthesis,
    #[error("unexpected parenthesis")]
    UnexpectedParenthesis,
    #[error("invalid escape character {0}")]
    InvalidEscape(char),
    #[error("empty expression")]
    Empty,
}

/// Extract `concat` as an operand of the Or operator and append it to `concat_or`.
fn append_concat(ctx: &mut Context) {
    if ctx.concat.len() == 1 {
        let c = ctx.concat.pop().unwrap();
        ctx.concat_or.push(c);
    } else {
        ctx.concat_or.push(Ast::Concat(mem::take(&mut ctx.concat)));
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

#[derive(Debug, Default)]
struct Context {
    concat: Vec<Ast>,
    concat_or: Vec<Ast>,
    // Stack that holds the previous context `(concat, concat_or)`.
    stack: Vec<(Vec<Ast>, Vec<Ast>)>,
}

/// Parse a regular expression pattern into an abstraction syntax tree (AST).
pub fn parse(pattern: &str) -> Result<Ast, ParseError> {
    let mut ctx = Context::default();
    let mut escaping = false;

    macro_rules! quantifier {
        ($operator:expr) => {
            if let Some(prev_ast) = ctx.concat.pop() {
                ctx.concat.push($operator(Box::new(prev_ast)));
            } else {
                return Err(ParseError::MissingOperand);
            }
        };
    }

    for c in pattern.chars() {
        if escaping {
            if matches!(c, '*' | '+' | '\\' | '?' | '(' | ')' | '|') {
                ctx.concat.push(Ast::Char(c));
            } else {
                return Err(ParseError::InvalidEscape(c));
            }
            escaping = false;
            continue;
        }

        match c {
            '|' => {
                if ctx.concat.is_empty() {
                    return Err(ParseError::MissingOperand);
                }

                // Append the left operand to `concat_or`.
                append_concat(&mut ctx);
            }
            '?' => quantifier!(Ast::Question),
            '*' => quantifier!(Ast::Star),
            '+' => quantifier!(Ast::Plus),
            '(' => {
                // Epilogue: push the current context.
                let prev = (mem::take(&mut ctx.concat), mem::take(&mut ctx.concat_or));
                ctx.stack.push(prev);
            }
            ')' => {
                if let Some((mut prev_concat, prev_concat_or)) = ctx.stack.pop() {
                    // Skip `()`.
                    if ctx.concat.is_empty() {
                        continue;
                    }

                    // Construct the AST of the expression in parentheses.
                    append_concat(&mut ctx);
                    if let Some(inner_ast) = or_ast(ctx.concat_or) {
                        prev_concat.push(inner_ast);
                    }

                    // Prologue: Rewind the context.
                    ctx.concat = prev_concat;
                    ctx.concat_or = prev_concat_or;
                } else {
                    return Err(ParseError::UnexpectedParenthesis);
                }
            }
            '\\' => escaping = true,
            _ => ctx.concat.push(Ast::Char(c)),
        }
    }

    // Check if there are unclosed parentheses.
    if !ctx.stack.is_empty() {
        return Err(ParseError::UnclosedParenthesis);
    }

    // Process the last operand.
    if ctx.concat.is_empty() {
        // Despite the presence of the Or operator, the right operand is missing.
        if !ctx.concat_or.is_empty() {
            return Err(ParseError::MissingOperand);
        }
    } else {
        // After going through all characters, append the right(=last) operand to `concat_or`.
        append_concat(&mut ctx);
    }

    if let Some(ast) = or_ast(ctx.concat_or) {
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

        // Error
        assert_eq!(parse("|b"), Err(ParseError::MissingOperand));
        assert_eq!(parse("a|"), Err(ParseError::MissingOperand));
        assert_eq!(parse("|"), Err(ParseError::MissingOperand));

        // Empty expression
        assert_eq!(parse(""), Err(ParseError::Empty));
    }

    #[test]
    fn parenthesis() {
        let ast = Ast::Concat(vec![
            Ast::Char('a'),
            Ast::Char('b'),
            Ast::Or(
                Ast::Concat(vec![Ast::Char('c'), Ast::Char('d')]).into(),
                Ast::Concat(vec![Ast::Char('e'), Ast::Char('f')]).into(),
            ),
        ]);
        assert_eq!(parse("ab(cd|ef)").unwrap(), ast);

        // Error
        assert_eq!(parse("(ab"), Err(ParseError::UnclosedParenthesis));
        assert_eq!(parse("ab)"), Err(ParseError::UnexpectedParenthesis));
        assert_eq!(parse("("), Err(ParseError::UnclosedParenthesis));
        assert_eq!(parse(")"), Err(ParseError::UnexpectedParenthesis));

        // Empty expression
        assert_eq!(parse("()"), Err(ParseError::Empty));
    }

    #[test]
    fn escape() {
        let ast = Ast::Char('+');
        assert_eq!(parse(r"\+").unwrap(), ast);

        let ast = Ast::Concat(vec![Ast::Char('*'), Ast::Char('b'), Ast::Char('?')]);
        assert_eq!(parse(r"\*b\?").unwrap(), ast);

        let ast = Ast::Concat(vec![Ast::Char('\\'), Ast::Char('\\'), Ast::Char('\\')]);
        assert_eq!(parse(r"\\\\\\").unwrap(), ast);

        // Error
        assert_eq!(parse(r"\a"), Err(ParseError::InvalidEscape('a')));
        assert_eq!(parse(r"a\bc"), Err(ParseError::InvalidEscape('b')));
    }

    #[test]
    fn question() {
        let ast = Ast::Question(Ast::Char('a').into());
        assert_eq!(parse("a?").unwrap(), ast);
        let ast = Ast::Concat(vec![Ast::Question(Ast::Char('a').into()), Ast::Char('b')]);
        assert_eq!(parse("a?b").unwrap(), ast);

        let ast = Ast::Concat(vec![
            Ast::Char('a'),
            Ast::Question(Ast::Concat(vec![Ast::Char('b'), Ast::Char('c')]).into()),
            Ast::Char('d'),
            Ast::Char('e'),
        ]);
        assert_eq!(parse("a(bc)?de").unwrap(), ast);

        // Error
        assert_eq!(parse("?"), Err(ParseError::MissingOperand));
        assert_eq!(parse("?abc"), Err(ParseError::MissingOperand));
    }
}
