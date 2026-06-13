//! Storage error message helpers.

use std::path::{Path, PathBuf};

use anyhow::anyhow;

use crate::storage::types::{RowKey, TableName};

pub fn row_not_found(table: &TableName, key: &RowKey) -> anyhow::Error {
    anyhow!("row not found: table={table}, key={key}")
}

pub fn table_not_found(table: &TableName) -> anyhow::Error {
    anyhow!("table not found: {table}")
}

pub fn row_already_exists(table: &TableName, key: &RowKey) -> anyhow::Error {
    anyhow!("row already exists: table={table}, key={key}")
}

pub fn table_already_exists(table: &TableName) -> anyhow::Error {
    anyhow!("table already exists: {table}")
}

pub fn invalid_blob_key(key: &str) -> anyhow::Error {
    anyhow!("invalid blob key: {key}")
}

pub fn ref_not_found(reference: impl std::fmt::Display) -> anyhow::Error {
    anyhow!("ref not found: {reference}")
}

pub fn merge_conflict(paths: &[PathBuf]) -> anyhow::Error {
    anyhow!("merge conflict: {paths:?}")
}

pub fn corrupted_data(path: &Path, reason: impl std::fmt::Display) -> anyhow::Error {
    anyhow!("corrupted data at {}: {reason}", path.display())
}

pub fn not_initialized(path: &Path) -> anyhow::Error {
    anyhow!("repository not initialized: {}", path.display())
}

pub fn empty_repository() -> anyhow::Error {
    anyhow!("repository is empty: no commits found")
}

pub fn commit_not_found(id: impl std::fmt::Display) -> anyhow::Error {
    anyhow!("commit not found: {id}")
}

pub fn blob_not_found(key: &str) -> anyhow::Error {
    anyhow!("blob not found: {key}")
}

pub fn unexpected_entry_type(
    path: &Path,
    expected: impl std::fmt::Display,
    found: impl std::fmt::Display,
) -> anyhow::Error {
    anyhow!(
        "unexpected entry type at {}: expected {expected}, found {found}",
        path.display()
    )
}

pub fn branch_already_exists(branch: impl std::fmt::Display) -> anyhow::Error {
    anyhow!("branch already exists: {branch}")
}

pub fn concurrent_modification(branch: impl std::fmt::Display) -> anyhow::Error {
    anyhow!("concurrent modification: branch {branch} was updated by another transaction")
}

pub fn schema_violation(message: impl Into<String>) -> anyhow::Error {
    anyhow!("schema violation: {}", message.into())
}

pub fn internal(message: impl Into<String>) -> anyhow::Error {
    anyhow!("internal error: {}", message.into())
}

pub fn is_not_found(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.starts_with("row not found:")
        || message.starts_with("table not found:")
        || message.starts_with("ref not found:")
        || message.starts_with("commit not found:")
        || message.starts_with("blob not found:")
}

pub fn is_conflict(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.starts_with("row already exists:")
        || message.starts_with("table already exists:")
        || message.starts_with("merge conflict:")
        || message.starts_with("concurrent modification:")
}

pub fn is_retriable(error: &anyhow::Error) -> bool {
    error.to_string().starts_with("concurrent modification:")
}
