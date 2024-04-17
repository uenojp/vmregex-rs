mod codegen;
mod machine;
mod parser;

use codegen::GenerateCodeError;
use machine::{Machine, MatchError};
use parser::ParseError;

use thiserror::Error;

/// Regular expression.
///
/// # Example
/// ```
/// use vmregex::Regex;
/// 
/// let re = Regex::new("a(b|c)*d+").unwrap();
/// assert!(re.is_match("abbd").unwrap());
/// assert!(re.is_match("acd").unwrap());
/// assert!(re.is_match("abcbcd").unwrap());
/// assert!(re.is_match("add").unwrap());
/// assert!(!re.is_match("abce").unwrap());
/// ```
pub struct Regex {
    machine: Machine,
}

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),
    #[error("codegen error: {0}")]
    GenerateCodeError(#[from] GenerateCodeError),
}

impl Regex {
    /// Compile a regular expression.
    pub fn new(pattern: &str) -> Result<Self, SyntaxError> {
        let ast = parser::parse(pattern)?;
        let instructions = codegen::CodeGenerator::default().generate_code(ast)?;
        let machine = Machine::new(instructions);
        Ok(Self { machine })
    }

    /// Check if the text matches the regular expression.
    pub fn is_match(&self, text: &str) -> Result<bool, MatchError> {
        let chars = text.chars().collect::<Vec<_>>();
        self.machine.is_match(&chars)
    }
}
