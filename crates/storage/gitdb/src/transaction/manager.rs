//! Transaction manager - coordinates all transaction operations.
//!
//! The TransactionManager is the main entry point for transactions.
//! It handles:
//! - Transaction creation and lifecycle
//! - Tracking active transactions
//! - Serializing commits to main
//! - Cleanup of abandoned transactions

use std::collections::HashMap;
use std::sync::Arc;

use az_derive_aliases::{apply, plain_clone};
use parking_lot::{Mutex, RwLock};
use ulid::Ulid;

use crate::storage::{CommitId, GitRepository};
use crate::transaction::context::{Transaction, TransactionMetadata, TxActive};
use crate::transaction::error::{TransactionError, TransactionResult};
use crate::transaction::isolation::IsolationLevel;

/// Transaction manager - coordinates all transaction operations.
///
/// Thread-safe: can be shared across threads via Clone (uses Arc internally).
#[apply(plain_clone)]
pub struct TransactionManager {
    inner: Arc<TransactionManagerInner>,
}

struct TransactionManagerInner {
    /// The underlying repository.
    repo: GitRepository,
    /// Active transactions tracked by ID.
    active: RwLock<HashMap<String, TransactionMetadata>>,
    /// Mutex for serializing commits to main branch.
    commit_lock: Mutex<()>,
}

impl TransactionManager {
    /// Create a new transaction manager for the given repository.
    pub fn new(repo: GitRepository) -> Self {
        Self {
            inner: Arc::new(TransactionManagerInner {
                repo,
                active: RwLock::new(HashMap::new()),
                commit_lock: Mutex::new(()),
            }),
        }
    }

    /// Get a reference to the underlying repository.
    pub fn repo(&self) -> &GitRepository {
        &self.inner.repo
    }

    /// Begin a new transaction with the default isolation level.
    pub fn begin(&self) -> TransactionResult<Transaction<TxActive>> {
        self.begin_with_isolation(IsolationLevel::default())
    }

    /// Begin a new transaction with a specific isolation level.
    pub fn begin_with_isolation(
        &self,
        isolation: IsolationLevel,
    ) -> TransactionResult<Transaction<TxActive>> {
        // Generate unique transaction ID
        let tx_id = Ulid::new().to_string().to_lowercase();

        // Get current main head as base
        let base_commit = self.inner.repo.head()?;

        // Create transaction branch
        let branch = self
            .inner
            .repo
            .create_transaction_branch(&tx_id, base_commit)?;

        // Create transaction object
        let tx = Transaction::new(
            self.inner.repo.clone(),
            tx_id.clone(),
            branch.clone(),
            base_commit,
            isolation,
        );

        // Track in active transactions
        {
            let mut active = self.inner.active.write();
            active.insert(tx_id.clone(), tx.metadata.clone());
        }

        Ok(tx)
    }

    /// Get the number of active transactions.
    pub fn active_count(&self) -> usize {
        self.inner.active.read().len()
    }

    /// List all active transaction IDs.
    pub fn active_transactions(&self) -> Vec<String> {
        self.inner.active.read().keys().cloned().collect()
    }

    /// Check if a transaction is active.
    pub fn is_active(&self, tx_id: &str) -> bool {
        self.inner.active.read().contains_key(tx_id)
    }

    /// Get metadata for an active transaction.
    pub fn get_transaction_info(&self, tx_id: &str) -> Option<TransactionMetadata> {
        self.inner.active.read().get(tx_id).cloned()
    }

    /// Mark a transaction as completed (committed or aborted).
    ///
    /// Called internally when a transaction commits or rolls back.
    pub(crate) fn mark_completed(&self, tx_id: &str) {
        self.inner.active.write().remove(tx_id);
    }

    /// Commit a transaction with serialization.
    ///
    /// This acquires a lock to ensure only one transaction commits at a time,
    /// preventing race conditions during the fast-forward operation.
    pub fn commit_transaction(&self, tx: Transaction<TxActive>) -> TransactionResult<CommitId> {
        // Acquire commit lock to serialize commits
        let _guard = self.inner.commit_lock.lock();

        let tx_id = tx.id().to_string();

        // Perform the commit
        let committed = tx.commit()?;

        // Remove from active tracking
        self.mark_completed(&tx_id);

        Ok(committed.final_commit())
    }

    /// Rollback a transaction.
    pub fn rollback_transaction(&self, tx: Transaction<TxActive>) -> TransactionResult<()> {
        let tx_id = tx.id().to_string();

        // Perform the rollback
        tx.rollback()?;

        // Remove from active tracking
        self.mark_completed(&tx_id);

        Ok(())
    }

    /// Clean up abandoned transactions.
    ///
    /// This removes transaction branches for transactions that are no longer
    /// tracked (e.g., due to crashes or improper cleanup).
    pub fn cleanup_abandoned(&self) -> TransactionResult<usize> {
        let active_ids: std::collections::HashSet<_> =
            self.inner.active.read().keys().cloned().collect();

        // List all transaction branches
        let branches = self
            .inner
            .repo
            .with_repo(|repo| crate::storage::RefManager::list_transaction_branches(repo))?;

        let mut cleaned = 0;
        for branch in branches {
            if let Some(tx_id) = branch.transaction_id() {
                if !active_ids.contains(tx_id) {
                    // This branch has no active transaction - clean it up
                    if self.inner.repo.delete_transaction_branch(tx_id).is_ok() {
                        cleaned += 1;
                    }
                }
            }
        }

        Ok(cleaned)
    }

    /// Execute a function within a transaction, automatically committing or rolling back.
    ///
    /// If the function returns Ok, the transaction is committed.
    /// If the function returns Err or panics, the transaction is rolled back.
    pub fn with_transaction<F, T>(&self, f: F) -> TransactionResult<T>
    where
        F: FnOnce(&mut Transaction<TxActive>) -> TransactionResult<T>,
    {
        self.with_transaction_isolation(IsolationLevel::default(), f)
    }

    /// Execute a function within a transaction with a specific isolation level.
    pub fn with_transaction_isolation<F, T>(
        &self,
        isolation: IsolationLevel,
        f: F,
    ) -> TransactionResult<T>
    where
        F: FnOnce(&mut Transaction<TxActive>) -> TransactionResult<T>,
    {
        let mut tx = self.begin_with_isolation(isolation)?;

        match f(&mut tx) {
            Ok(result) => {
                self.commit_transaction(tx)?;
                Ok(result)
            }
            Err(e) => {
                self.rollback_transaction(tx)?;
                Err(e)
            }
        }
    }

    /// Get current head of main branch.
    pub fn head(&self) -> TransactionResult<CommitId> {
        self.inner.repo.head().map_err(TransactionError::from)
    }
}

// Ensure TransactionManager can be safely shared across threads
impl std::fmt::Debug for TransactionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionManager")
            .field("active_count", &self.active_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    use crate::storage::{RowKey, TableName};

    fn setup() -> (TempDir, TransactionManager) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path()).unwrap();
        let manager = TransactionManager::new(repo);
        (dir, manager)
    }

    #[test]
    fn test_begin_and_commit() {
        let (_dir, manager) = setup();

        // Begin transaction
        let mut tx = manager.begin().unwrap();
        assert!(manager.is_active(tx.id()));

        // Create table
        let table = TableName::new("users").unwrap();
        tx.create_table(&table).unwrap();

        // Commit
        let commit_id = manager.commit_transaction(tx).unwrap();
        assert!(manager.active_count() == 0);

        // Verify table exists on main
        let repo = manager.repo();
        assert!(repo.table_exists(&table, commit_id).unwrap());
    }

    #[test]
    fn test_begin_and_rollback() {
        let (_dir, manager) = setup();
        let initial_head = manager.head().unwrap();

        // Begin transaction
        let mut tx = manager.begin().unwrap();

        // Create table
        let table = TableName::new("users").unwrap();
        tx.create_table(&table).unwrap();

        // Rollback
        manager.rollback_transaction(tx).unwrap();
        assert!(manager.active_count() == 0);

        // Verify table doesn't exist on main
        let repo = manager.repo();
        let head = repo.head().unwrap();
        assert_eq!(head, initial_head);
        assert!(!repo.table_exists(&table, head).unwrap());
    }

    #[test]
    fn test_with_transaction() {
        let (_dir, manager) = setup();
        let table = TableName::new("users").unwrap();
        let key = RowKey::new("user1").unwrap();

        // Execute with transaction
        let result = manager.with_transaction(|tx| {
            tx.create_table(&table)?;
            let mut data = BTreeMap::new();
            data.insert("name".to_string(), Value::String("Alice".to_string()));
            tx.insert_data(&table, key.clone(), data)?;
            Ok(())
        });

        assert!(result.is_ok());

        // Verify changes persisted
        let repo = manager.repo();
        let head = repo.head().unwrap();
        let row = repo.read_row(&table, &key, head).unwrap().unwrap();
        assert_eq!(row.get("name"), Some(&Value::String("Alice".to_string())));
    }

    #[test]
    fn test_with_transaction_rollback_on_error() {
        let (_dir, manager) = setup();
        let table = TableName::new("users").unwrap();

        // Execute with transaction that errors
        let result: TransactionResult<()> = manager.with_transaction(|tx| {
            tx.create_table(&table)?;
            // Simulate an error
            Err(TransactionError::Internal("test error".to_string()))
        });

        assert!(result.is_err());

        // Verify table doesn't exist
        let repo = manager.repo();
        let head = repo.head().unwrap();
        assert!(!repo.table_exists(&table, head).unwrap());
    }

    #[test]
    fn test_active_transactions() {
        let (_dir, manager) = setup();

        assert_eq!(manager.active_count(), 0);

        let tx1 = manager.begin().unwrap();
        assert_eq!(manager.active_count(), 1);

        let tx2 = manager.begin().unwrap();
        assert_eq!(manager.active_count(), 2);

        manager.rollback_transaction(tx1).unwrap();
        assert_eq!(manager.active_count(), 1);

        manager.rollback_transaction(tx2).unwrap();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_cleanup_abandoned() {
        let (_dir, manager) = setup();
        let head = manager.head().unwrap();

        // Create a branch that looks like a transaction but isn't tracked
        let repo = manager.repo();
        repo.create_transaction_branch("abandoned123", head)
            .unwrap();

        // Cleanup should find and remove it
        let cleaned = manager.cleanup_abandoned().unwrap();
        assert_eq!(cleaned, 1);
    }

    #[test]
    fn test_concurrent_commit_serialization() {
        let (_dir, manager) = setup();

        // Create a table first using a transaction (proper way)
        let table = TableName::new("counter").unwrap();
        manager
            .with_transaction(|tx| {
                tx.create_table(&table)?;
                Ok(())
            })
            .unwrap();

        // Start two transactions from the same base
        let mut tx1 = manager.begin().unwrap();
        let mut tx2 = manager.begin().unwrap();

        // Both try to insert to the same table (different keys)
        let mut data1 = BTreeMap::new();
        data1.insert("value".to_string(), Value::Number(1.into()));
        tx1.insert_data(&table, RowKey::new("key1").unwrap(), data1)
            .unwrap();

        let mut data2 = BTreeMap::new();
        data2.insert("value".to_string(), Value::Number(2.into()));
        tx2.insert_data(&table, RowKey::new("key2").unwrap(), data2)
            .unwrap();

        // First commit should succeed
        let commit1 = manager.commit_transaction(tx1);
        assert!(commit1.is_ok());

        // Second commit might fail with conflict (depends on whether paths overlap)
        // In this case, different keys so should succeed
        let commit2 = manager.commit_transaction(tx2);
        // This will fail because the base commit has moved
        assert!(commit2.is_err());
    }
}
