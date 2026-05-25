//! SQL parsing errors.

use az_derive_aliases::{apply, error_eq};

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

impl From<sqlparser::parser::ParserError> for ParseError {
    fn from(e: sqlparser::parser::ParserError) -> Self {
        ParseError::Syntax(e.to_string())
    }
}
