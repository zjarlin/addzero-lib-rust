//! Transaction management for GitDB.
//!
//! This module implements ACID transactions using Git branches.
//! Each transaction gets its own branch (`tx/{ulid}`) where changes accumulate.
//! On commit, changes are merged to main; on rollback, the branch is deleted.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   TransactionManager                        │
//! │  (Coordinates transactions, tracks active tx, serializes)   │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!        ┌─────────────────────┼─────────────────────┐
//!        │                     │                     │
//!        ▼                     ▼                     ▼
//!  ┌─────────────┐       ┌─────────────┐       ┌─────────────┐
//!  │ Transaction │       │ Isolation   │       │   Lock      │
//!  │  (Context)  │       │   Level     │       │  Manager    │
//!  └─────────────┘       └─────────────┘       └─────────────┘
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use gitdb::transaction::{TransactionManager, IsolationLevel};
//!
//! let manager = TransactionManager::new(repo);
//!
//! // Begin a transaction
//! let tx = manager.begin(IsolationLevel::ReadCommitted)?;
//!
//! // Perform operations
//! tx.insert("users", row)?;
//! tx.update("users", updated_row)?;
//!
//! // Commit or rollback
//! tx.commit()?;  // or tx.rollback();
//! ```

mod context;
mod error;
mod isolation;
mod manager;

pub use context::{Transaction, TxAborted, TxActive, TxCommitted};
pub use error::{TransactionError, TransactionResult};
pub use isolation::IsolationLevel;
pub use manager::TransactionManager;
