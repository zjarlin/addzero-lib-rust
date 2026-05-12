//! core type-safe wrappers around git primitives for the storage layer.

use git2::Oid;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;

/// This makes sure we don't accidentally pass a blob ID where a commit ID
/// is expected. The inner Oid is only accessible within the storage module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommitId(pub(crate) Oid);

impl CommitId {
    pub(crate) fn new(oid: Oid) -> Self {
        Self(oid)
    }

    /// raw Oid (for internal use only)
    pub(crate) fn raw(&self) -> Oid {
        self.0
    }

    /// parse CommitId from a hex string
    pub fn from_hex(hex: &str) -> Result<Self, git2::Error> {
        Oid::from_str(hex).map(CommitId)
    }
    /// short form of the commit ID
    pub fn short(&self) -> String {
        self.0.to_string()[..7].to_string()
    }
}

impl fmt::Display for CommitId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Git blob identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlobId(pub(crate) Oid);

impl BlobId {
    pub(crate) fn new(oid: Oid) -> Self {
        Self(oid)
    }
    pub(crate) fn raw(&self) -> Oid {
        self.0
    }
}

impl fmt::Display for BlobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Git tree identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TreeId(pub(crate) Oid);

impl TreeId {
    pub(crate) fn new(oid: Oid) -> Self {
        Self(oid)
    }

    pub(crate) fn raw(&self) -> Oid {
        self.0
    }
}

impl fmt::Display for TreeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A validated table name.
///
/// Table names are restricted to prevent path traversal attacks and
/// ensure compatibility with filesystem and Git constraints.
///
/// Valid names:
/// - 1-64 characters
/// - Alphanumeric, underscores, hyphens only
/// - Must start with a letter or underscore
/// - Cannot be reserved names (_schema, _meta, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableName(String);

impl TableName {
    /// reserved table names that can't be used
    const RESERVED: &'static [&'static str] = &["_schema", "_meta", "_system", "_git"];

    /// create a new TableName, validating the input
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidNameError> {
        let name = name.into();
        Self::validate(&name)?;
        Ok(Self(name))
    }

    /// Validate a table name.
    fn validate(name: &str) -> Result<(), InvalidNameError> {
        if name.is_empty() {
            return Err(InvalidNameError::Empty);
        }

        if name.len() > 64 {
            return Err(InvalidNameError::TooLong(name.len()));
        }

        let first_char = name.chars().next().unwrap();

        // Must start with a letter or underscore (not a digit)
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return Err(InvalidNameError::InvalidStart(first_char));
        }

        for (i, c) in name.chars().enumerate() {
            if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
                return Err(InvalidNameError::InvalidCharacter {
                    char: c,
                    position: i,
                });
            }
        }

        if Self::RESERVED.contains(&name.to_lowercase().as_str()) {
            return Err(InvalidNameError::Reserved(name.to_string()));
        }

        Ok(())
    }
    /// get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for TableName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TableName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A validated row key (primary key)
///
/// row keys are used as filenames, so they have similar restrictions
/// to table names but are typically auto generated (ULIDs, UUIDs)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RowKey(String);

impl RowKey {
    pub fn new(key: impl Into<String>) -> Result<Self, InvalidNameError> {
        let key = key.into();
        Self::validate(&key)?;
        Ok(Self(key))
    }

    /// Validate a row name.
    fn validate(key: &str) -> Result<(), InvalidNameError> {
        if key.is_empty() {
            return Err(InvalidNameError::Empty);
        }

        if key.len() > 128 {
            return Err(InvalidNameError::TooLong(key.len()));
        }

        for (i, c) in key.chars().enumerate() {
            // alphanumeric, underscore, hyphen allowed
            if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
                return Err(InvalidNameError::InvalidCharacter {
                    char: c,
                    position: i,
                });
            }
        }

        Ok(())
    }

    /// get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }

    /// Generate a new ULID-based row key.
    pub fn generate() -> Self {
        Self(ulid::Ulid::new().to_string().to_lowercase())
    }
}

impl fmt::Display for RowKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for RowKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Full path to a row in the repository.
///
/// Format: `{table}/{row_key}. json`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowPath {
    pub table: TableName,
    pub key: RowKey,
}

impl RowPath {
    /// create a new RowPath
    pub fn new(table: TableName, key: RowKey) -> Self {
        Self { table, key }
    }

    /// convert to a PathBuf for filesystem operations
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(format!("{}/{}. json", self.table, self.key))
    }

    /// get the path as a string
    pub fn as_string(&self) -> String {
        format!("{}/{}.json", self.table, self.key)
    }
}

impl fmt::Display for RowPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}. json", self.table, self.key)
    }
}

/// a branch name, with special handling for transaction branches
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BranchName(String);

impl BranchName {
    /// the main branch name
    pub const MAIN: &'static str = "main";

    /// prefix for transaction branches
    pub const TX_PREFIX: &'static str = "tx/";

    /// create a new BranchName
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidNameError> {
        let name = name.into();
        // basic validation , git is more permissive but we gon be restrictive
        if name.is_empty() {
            return Err(InvalidNameError::Empty);
        }
        if name.contains("..") || name.ends_with('/') || name.starts_with('/') {
            return Err(InvalidNameError::InvalidPath(name));
        }
        Ok(Self(name))
    }

    /// create the main branch reference
    pub fn main() -> Self {
        Self(Self::MAIN.to_string())
    }

    /// create a transaction branch name
    pub fn for_transaction(tx_id: &str) -> Self {
        Self(format!("{}{}", Self::TX_PREFIX, tx_id))
    }

    /// check if this is a transaction branch
    pub fn is_transaction_branch(&self) -> bool {
        self.0.starts_with(Self::TX_PREFIX)
    }

    /// extract transaction ID if this is a transaction branch
    pub fn transaction_id(&self) -> Option<&str> {
        if self.is_transaction_branch() {
            Some(&self.0[Self::TX_PREFIX.len()..])
        } else {
            None
        }
    }

    /// get the full ref path (e.g., "refs/heads/main")
    pub fn as_ref_path(&self) -> String {
        format!("refs/heads/{}", self.0)
    }

    /// get the short name
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// git signature (author/committer info)
#[derive(Debug, Clone)]
pub struct GitSignature {
    pub name: String,
    pub email: String,
}

impl GitSignature {
    /// create a new signature
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
        }
    }

    /// default signature for GitDB operations
    pub fn gitdb() -> Self {
        Self::new("GitDB", "gitdb@localhost")
    }

    /// convert to git2::Signature
    pub(crate) fn to_git2_signature(&self) -> Result<git2::Signature<'static>, git2::Error> {
        git2::Signature::now(&self.name, &self.email)
    }
}

impl Default for GitSignature {
    fn default() -> Self {
        Self::gitdb()
    }
}

/// error type for invalid names (tables, rows, branches)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidNameError {
    Empty,
    TooLong(usize),
    InvalidStart(char),
    InvalidCharacter { char: char, position: usize },
    Reserved(String),
    InvalidPath(String),
}

impl fmt::Display for InvalidNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "name cannot be empty"),
            Self::TooLong(len) => write!(f, "name too long: {} characters", len),
            Self::InvalidStart(c) => write!(f, "name cannot start with '{}'", c),
            Self::InvalidCharacter { char, position } => {
                write!(f, "invalid character '{}' at position {}", char, position)
            }
            Self::Reserved(name) => write!(f, "'{}' is a reserved name", name),
            Self::InvalidPath(path) => write!(f, "invalid path: '{}'", path),
        }
    }
}

impl std::error::Error for InvalidNameError {}

/// represents a change in a diff between commits
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub path: PathBuf,
    pub status: ChangeStatus,
}

/// the type of change in a diff
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_name_valid() {
        assert!(TableName::new("users").is_ok());
        assert!(TableName::new("user_accounts").is_ok());
        assert!(TableName::new("User123").is_ok());
        assert!(TableName::new("_private").is_ok());
        assert!(TableName::new("my-table").is_ok());
    }

    #[test]
    fn test_table_name_invalid() {
        assert!(TableName::new("").is_err());
        assert!(TableName::new("123users").is_err()); // starts with number
        assert!(TableName::new("users/admin").is_err()); // contains slash
        assert!(TableName::new("_schema").is_err()); // reserved
        assert!(TableName::new("a".repeat(65)).is_err()); // too long
    }

    #[test]
    fn test_row_key_valid() {
        assert!(RowKey::new("abc123").is_ok());
        assert!(RowKey::new("01ARZ3NDEKTSV4RRFFQ69G5FAV").is_ok()); // ULID
        assert!(RowKey::new("550e8400-e29b-41d4-a716-446655440000").is_ok()); // UUID with hyphens is valid
        assert!(RowKey::new("simple_key").is_ok());
    }

    #[test]
    fn test_row_key_generate() {
        let key1 = RowKey::generate();
        let key2 = RowKey::generate();
        assert_ne!(key1, key2);
        assert_eq!(key1.as_str().len(), 26); // ULID length
    }

    #[test]
    fn test_branch_name_transaction() {
        let branch = BranchName::for_transaction("abc123");
        assert!(branch.is_transaction_branch());
        assert_eq!(branch.transaction_id(), Some("abc123"));
        assert_eq!(branch.as_ref_path(), "refs/heads/tx/abc123");
    }

    #[test]
    fn test_branch_name_main() {
        let branch = BranchName::main();
        assert!(!branch.is_transaction_branch());
        assert_eq!(branch.transaction_id(), None);
        assert_eq!(branch.as_ref_path(), "refs/heads/main");
    }
}
