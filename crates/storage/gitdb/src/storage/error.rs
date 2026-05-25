//! Storage layer error types
//!
//! All errors that can occur during storage operations are defined here
//! We use `thiserror` for ergonomic error definition and better error messages

use std::path::PathBuf;

use az_derive_aliases::{apply, error};

use crate::storage::types::{InvalidNameError, RowKey, TableName};

/// the main error type for storage operations
#[apply(error)]
pub enum StorageError {
    /// error from the underlying Git library
    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    /// the requested row was not found
    #[error("row not found: table={table}, key={key}")]
    RowNotFound { table: TableName, key: RowKey },

    /// the requested table was not found
    #[error("table not found: {0}")]
    TableNotFound(TableName),

    /// the row already exists (duplicate primary key)
    #[error("row already exists: table={table}, key={key}")]
    RowAlreadyExists { table: TableName, key: RowKey },

    /// the table already exists
    #[error("table already exists: {0}")]
    TableAlreadyExists(TableName),

    /// invalid table name
    #[error("invalid table name: {0}")]
    InvalidTableName(#[from] InvalidNameError),

    /// invalid blob/object key
    #[error("invalid blob key: {0}")]
    InvalidBlobKey(String),

    /// JSON serialization or deserialization failed
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// the specified branch/ref was not found
    #[error("ref not found: {0}")]
    RefNotFound(String),

    /// merge conflict detected during commit
    #[error("merge conflict: {conflicting_paths:?}")]
    MergeConflict { conflicting_paths: Vec<PathBuf> },

    /// data integrity check failed
    #[error("corrupted data at {path}: {reason}")]
    CorruptedData { path: PathBuf, reason: String },

    /// I/O error (filesystem level)
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// repo is not initialized
    #[error("repository not initialized: {0}")]
    NotInitialized(PathBuf),

    /// repo is empty (no commits)
    #[error("repository is empty: no commits found")]
    EmptyRepository,

    /// the commit was not found
    #[error("commit not found: {0}")]
    CommitNotFound(String),

    /// invalid UTF-8 in blob content
    #[error("invalid utf-8 in blob: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    /// the requested blob/object was not found
    #[error("blob not found: {0}")]
    BlobNotFound(String),

    /// the tree entry has an unexpected type
    #[error("unexpected entry type at {path}: expected {expected}, found {found}")]
    UnexpectedEntryType {
        path: PathBuf,
        expected: String,
        found: String,
    },

    /// branch already exists
    #[error("branch already exists: {0}")]
    BranchAlreadyExists(String),

    /// branch update failed due to concurrent modification
    #[error("concurrent modification: branch {branch} was updated by another transaction")]
    ConcurrentModification { branch: String },

    /// the row data doesn't match the expected schema
    #[error("schema violation: {0}")]
    SchemaViolation(String),

    /// internal error that shouldn't happen
    #[error("internal error: {0}")]
    Internal(String),
}

impl StorageError {
    /// check if this error indicates the resource doesn't exist
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            StorageError::RowNotFound { .. }
                | StorageError::TableNotFound(_)
                | StorageError::RefNotFound(_)
                | StorageError::CommitNotFound(_)
                | StorageError::BlobNotFound(_)
        )
    }

    /// check if this error is a conflict
    pub fn is_conflict(&self) -> bool {
        matches!(
            self,
            StorageError::RowAlreadyExists { .. }
                | StorageError::TableAlreadyExists(_)
                | StorageError::MergeConflict { .. }
                | StorageError::ConcurrentModification { .. }
        )
    }

    /// check if this error is recoverable by retry
    pub fn is_retriable(&self) -> bool {
        matches!(self, StorageError::ConcurrentModification { .. })
    }
}

/// result type alias for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        let not_found = StorageError::TableNotFound(TableName::new("users").unwrap());
        assert!(not_found.is_not_found());
        assert!(!not_found.is_conflict());

        let conflict = StorageError::RowAlreadyExists {
            table: TableName::new("users").unwrap(),
            key: RowKey::new("123").unwrap(),
        };
        assert!(!conflict.is_not_found());
        assert!(conflict.is_conflict());
    }
}
