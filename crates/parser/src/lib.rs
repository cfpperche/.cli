//! Parser for the `.cli` language (working title).
//!
//! Implements the grammar in SPEC.md §3: a hand-rolled lexer and
//! recursive-descent parser with no dependencies. Diagnostics carry
//! `line:col` and are written so an agent can self-correct from the
//! message alone.

mod ast;
mod error;
mod lexer;
mod parser;

pub use ast::{Argument, Command, CompareOp, Condition, Pipeline, Statement, Value};
pub use error::ParseError;
pub use lexer::{lex, Spanned, Token};

/// Parse a full script into its statements.
pub fn parse(source: &str) -> Result<Vec<Statement>, ParseError> {
    parser::parse(source)
}
