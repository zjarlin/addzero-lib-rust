//! Query execution errors.

use thiserror::Error;

use crate::catalog::SchemaError;
use crate::sql::ParseError;
use crate::storage::{InvalidNameError, StorageError};
use crate::transaction::TransactionError;

/// Result type for query execution.
pub type ExecuteResult<T> = Result<T, ExecuteError>;

/// Query execution errors.
#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),

    #[error("transaction error: {0}")]
    Transaction(#[from] TransactionError),

    #[error("invalid name: {0}")]
    InvalidName(#[from] InvalidNameError),

    #[error("table not found: {0}")]
    TableNotFound(String),

    #[error("column not found: {0}")]
    ColumnNotFound(String),

    #[error("type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("null value in non-nullable column: {0}")]
    NullValue(String),

    #[error("duplicate key: {0}")]
    DuplicateKey(String),

    #[error("missing required column: {0}")]
    MissingColumn(String),

    #[error("invalid expression: {0}")]
    InvalidExpression(String),

    #[error("division by zero")]
    DivisionByZero,

    #[error("no active transaction")]
    NoTransaction,

    #[error("internal error: {0}")]
    Internal(String),
}

impl ExecuteError {
    /// Check if error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, ExecuteError::Transaction(t) if t.is_retryable())
    }
}
