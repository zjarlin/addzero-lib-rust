//! SQL parsing and AST types for GitDB.
//!
//! Uses `sqlparser` crate for parsing, then converts to our internal AST
//! representation for execution.

automod::dir!("src/sql");

pub use ast::*;
pub use error::{ParseError, ParseResult};
pub use parser::Parser;
