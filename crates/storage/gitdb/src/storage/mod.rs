//! storage layer for GitDB
//!
//! this module provides a complete abstraction over git for database storage.
//! The upper layers (transaction manager, query engine) use this API and never
//! touch git2 directly.
//!
//!  # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     GitRepository                           │
//! │  (High-level API: tables, rows, branches, transactions)     │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!        ┌─────────────────────┼─────────────────────┐
//!        │                     │                     │
//!        ▼                     ▼                     ▼
//!  ┌─────────────┐       ┌─────────────┐       ┌─────────────┐
//!  │    tree     │       │    blob     │       │    refs     │
//!  │  (tables)   │       │   (rows)    │       │ (branches)  │
//!  └─────────────┘       └─────────────┘       └─────────────┘
//!         │                     │                     │
//!         └─────────────────────┼─────────────────────┘
//!                               │
//!                               ▼
//!                        ┌─────────────┐
//!                        │   commit    │
//!                        │  (history)  │
//!                        └─────────────┘
//!  ```
//!
//! # Usage
//!
//! ```ignore
//! use gitdb::storage::{GitRepository, TableName, RowKey, Row};
//!
//! // Initialize or open
//! let repo = GitRepository::open_or_init("./my_database")?;
//!
//! // Get current state
//! let head = repo.head()?;
//!
//! // Create a table
//!  let table = TableName::new("users")?;
//!  let head = repo. create_table(&table, head, None)? ;
//!
//! // Insert a row
//! let key = RowKey::generate();
//! let row = Row::from_value(key, json!({"name": "Alice", "age": 30}))?;
//! let head = repo.insert_row(&table, row, head, None)?;
//!
//! // Read back
//! let user = repo.read_row(&table, &key, head)? ;
//! ```

mod blob;
mod commit;
mod error;
mod refs;
mod repository;
mod tree;
mod types;

// Re-export public API
pub use blob::Row;
pub use commit::{CommitInfo, CommitMessage};
pub use error::{StorageError, StorageResult};
pub use repository::{GitRepository, RepositoryStats, TreeSnapshot};
pub use types::{
    BlobId, BranchName, Change, ChangeStatus, CommitId, GitSignature, InvalidNameError, RowKey,
    RowPath, TableName, TreeId,
};

// Re-export for internal use by other modules
pub(crate) use refs::RefManager;
