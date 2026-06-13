//! GitDB 的底层存储抽象。
//!
//! 本模块把 `git2` 的提交、树、blob、ref 操作包装成数据库语义。事务管理器、
//! 查询引擎和上层 API 都只依赖这里的类型，不直接触碰 `git2`。
//!
//! # 架构
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
//! # 用法
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

automod::dir!("src/storage");

// 重新导出公开存储 API。
pub use blob::Row;
pub use commit::{CommitInfo, CommitMessage};
pub use error::{StorageError, StorageResult};
pub use repository::{GitRepository, RepositoryStats, TreeSnapshot};
pub use types::{
    BlobId, BranchName, Change, ChangeStatus, CommitId, GitSignature, InvalidNameError, RowKey,
    RowPath, TableName, TreeId,
};

// 重新导出给 crate 内其他模块使用。
pub(crate) use refs::RefManager;
