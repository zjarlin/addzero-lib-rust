//! Transaction error types.

use std::path::PathBuf;

use thiserror::Error;

use crate::storage::StorageError;

/// Result type for transaction operations.
pub type TransactionResult<T> = Result<T, TransactionError>;

/// Errors that can occur during transaction operations.
#[derive(Debug, Error)]
pub enum TransactionError {
    /// Storage layer error.
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    /// Transaction conflict - another transaction modified the same data.
    #[error("transaction conflict on paths: {}", paths_display(.paths))]
    Conflict {
        /// Paths that were modified by both this transaction and another.
        paths: Vec<PathBuf>,
    },

    /// Transaction was already committed or aborted.
    #[error("transaction {tx_id} is no longer active (state: {state})")]
    NotActive { tx_id: String, state: String },

    /// Transaction not found.
    #[error("transaction not found: {0}")]
    NotFound(String),

    /// Transaction timeout.
    #[error("transaction {tx_id} timed out after {elapsed_secs}s")]
    Timeout { tx_id: String, elapsed_secs: u64 },

    /// Deadlock detected.
    #[error("deadlock detected involving transaction {tx_id}")]
    Deadlock { tx_id: String },

    /// Invalid operation for current transaction state.
    #[error("invalid operation: {0}")]
    InvalidOperation(String),

    /// Serialization failure - retry the transaction.
    #[error("serialization failure, please retry transaction")]
    SerializationFailure,

    /// Internal error.
    #[error("internal transaction error: {0}")]
    Internal(String),
}

fn paths_display(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

impl TransactionError {
    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            TransactionError::Conflict { .. }
                | TransactionError::SerializationFailure
                | TransactionError::Timeout { .. }
        )
    }

    /// Create a conflict error from a list of paths.
    pub fn conflict(paths: Vec<PathBuf>) -> Self {
        Self::Conflict { paths }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryable() {
        let conflict = TransactionError::Conflict {
            paths: vec![PathBuf::from("users/123.json")],
        };
        assert!(conflict.is_retryable());

        let not_active = TransactionError::NotActive {
            tx_id: "tx001".to_string(),
            state: "committed".to_string(),
        };
        assert!(!not_active.is_retryable());
    }
}
