//! Planning errors.

use thiserror::Error;

use crate::catalog::SchemaError;
use crate::sql::ParseError;
use crate::storage::StorageError;

/// Result type for planning operations.
pub type PlanResult<T> = Result<T, PlanError>;

/// Query planning errors.
#[derive(Debug, Error)]
pub enum PlanError {
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),

    #[error("table not found: {0}")]
    TableNotFound(String),

    #[error("column not found: {0}")]
    ColumnNotFound(String),

    #[error("ambiguous column: {0}")]
    AmbiguousColumn(String),

    #[error("invalid join condition: {0}")]
    InvalidJoin(String),

    #[error("type mismatch: {0}")]
    TypeMismatch(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),

    #[error("optimization failed: {0}")]
    OptimizationFailed(String),

    #[error("internal error: {0}")]
    Internal(String),
}
