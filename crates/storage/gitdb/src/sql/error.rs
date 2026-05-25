//! SQL parsing errors.

use az_derive_aliases::{apply, error_eq, impl_from_match};

/// Result type for parsing operations.
pub type ParseResult<T> = Result<T, ParseError>;

/// SQL parsing errors.
#[apply(error_eq)]
pub enum ParseError {
    #[error("syntax error: {0}")]
    Syntax(String),

    #[error("unsupported statement: {0}")]
    UnsupportedStatement(String),

    #[error("unsupported expression: {0}")]
    UnsupportedExpression(String),

    #[error("unsupported data type: {0}")]
    UnsupportedDataType(String),

    #[error("invalid identifier: {0}")]
    InvalidIdentifier(String),

    #[error("missing required clause: {0}")]
    MissingClause(String),

    #[error("empty query")]
    EmptyQuery,

    #[error("multiple statements not supported")]
    MultipleStatements,
}

impl_from_match!(sqlparser::parser::ParserError => ParseError {
    value => ParseError::Syntax(value.to_string()),
});
