//! Transaction context using typestate pattern.
//!
//! The typestate pattern ensures at compile time that transactions
//! are used correctly:
//! - Only active transactions can perform operations
//! - Committed/aborted transactions cannot be reused
//! - Resources are properly cleaned up

use std::collections::BTreeMap;
use std::marker::PhantomData;

use az_derive_aliases::{apply, plain_clone_debug, plain_debug};
use serde_json::Value;

use crate::storage::{BranchName, CommitId, GitRepository, Row, RowKey, StorageError, TableName};
use crate::transaction::error::{TransactionError, TransactionResult};
use crate::transaction::isolation::IsolationLevel;

/// Marker type for active transactions.
#[apply(plain_debug)]
pub struct TxActive;

/// Marker type for committed transactions.
#[apply(plain_debug)]
pub struct TxCommitted;

/// Marker type for aborted transactions.
#[apply(plain_debug)]
pub struct TxAborted;

/// Transaction metadata stored in the manager.
#[apply(plain_clone_debug)]
pub struct TransactionMetadata {
    /// Unique transaction ID.
    pub tx_id: String,
    /// Transaction branch name.
    pub branch: BranchName,
    /// Commit where transaction started (base commit).
    pub base_commit: CommitId,
    /// Current head of the transaction branch.
    pub current_commit: CommitId,
    /// Isolation level for this transaction.
    pub isolation: IsolationLevel,
    /// When the transaction started.
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// A database transaction with typestate for lifecycle safety.
///
/// The `State` parameter tracks whether the transaction is:
/// - `TxActive`: Can perform operations
/// - `TxCommitted`: Successfully committed, no more operations allowed
/// - `TxAborted`: Rolled back, no more operations allowed
pub struct Transaction<State> {
    /// Transaction metadata.
    pub(crate) metadata: TransactionMetadata,
    /// Reference to the repository.
    pub(crate) repo: GitRepository,
    /// Phantom data for typestate.
    _state: PhantomData<State>,
}

impl<State> Transaction<State> {
    /// Get the transaction ID.
    pub fn id(&self) -> &str {
        &self.metadata.tx_id
    }

    /// Get the transaction's base commit (where it started).
    pub fn base_commit(&self) -> CommitId {
        self.metadata.base_commit
    }

    /// Get the isolation level.
    pub fn isolation(&self) -> IsolationLevel {
        self.metadata.isolation
    }

    /// Get the branch name for this transaction.
    pub fn branch(&self) -> &BranchName {
        &self.metadata.branch
    }
}

impl Transaction<TxActive> {
    /// Create a new active transaction.
    pub(crate) fn new(
        repo: GitRepository,
        tx_id: String,
        branch: BranchName,
        base_commit: CommitId,
        isolation: IsolationLevel,
    ) -> Self {
        Self {
            metadata: TransactionMetadata {
                tx_id,
                branch,
                base_commit,
                current_commit: base_commit,
                isolation,
                started_at: chrono::Utc::now(),
            },
            repo,
            _state: PhantomData,
        }
    }

    /// Get the commit to read from based on isolation level.
    ///
    /// For both isolation levels, we read from the transaction's current commit
    /// to see our own writes. The difference is:
    /// - ReadCommitted: If we haven't modified a row, we'd see the latest main state
    ///   (but our implementation reads from tx branch, which is simpler)
    /// - RepeatableRead: Always see snapshot from transaction start
    ///
    /// For simplicity, we always read from the transaction's current commit.
    /// This means we see our own writes but not concurrent modifications.
    fn read_commit(&self) -> TransactionResult<CommitId> {
        // Always read from transaction's current state to see own writes
        Ok(self.metadata.current_commit)
    }

    /// Get the current head of the transaction branch (for writes).
    pub fn current_commit(&self) -> CommitId {
        self.metadata.current_commit
    }

    // ==================== Table Operations ====================

    /// Create a new table.
    pub fn create_table(&mut self, table: &TableName) -> TransactionResult<()> {
        let new_commit = self.repo.create_table(
            table,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// Drop a table.
    pub fn drop_table(&mut self, table: &TableName) -> TransactionResult<()> {
        let new_commit = self.repo.drop_table(
            table,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// List all tables.
    pub fn list_tables(&self) -> TransactionResult<Vec<TableName>> {
        let commit = self.read_commit()?;
        self.repo
            .list_tables(commit)
            .map_err(TransactionError::from)
    }

    /// Check if a table exists.
    pub fn table_exists(&self, table: &TableName) -> TransactionResult<bool> {
        let commit = self.read_commit()?;
        self.repo
            .table_exists(table, commit)
            .map_err(TransactionError::from)
    }

    // ==================== Row Operations ====================

    /// Insert a new row.
    pub fn insert(&mut self, table: &TableName, row: Row) -> TransactionResult<()> {
        let new_commit = self.repo.insert_row(
            table,
            row,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// Insert a row from raw data.
    pub fn insert_data(
        &mut self,
        table: &TableName,
        key: RowKey,
        data: BTreeMap<String, Value>,
    ) -> TransactionResult<()> {
        let row = Row::new(key, data);
        self.insert(table, row)
    }

    /// Update an existing row.
    pub fn update(&mut self, table: &TableName, row: Row) -> TransactionResult<()> {
        let new_commit = self.repo.update_row(
            table,
            row,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// Insert or update a row (upsert).
    pub fn upsert(&mut self, table: &TableName, row: Row) -> TransactionResult<()> {
        let new_commit = self.repo.upsert_row(
            table,
            row,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// Delete a row.
    pub fn delete(&mut self, table: &TableName, key: &RowKey) -> TransactionResult<()> {
        let new_commit = self.repo.delete_row(
            table,
            key,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// Read a single row.
    pub fn read(&self, table: &TableName, key: &RowKey) -> TransactionResult<Option<Row>> {
        let commit = self.read_commit()?;
        self.repo
            .read_row(table, key, commit)
            .map_err(TransactionError::from)
    }

    /// Scan all rows in a table.
    pub fn scan(&self, table: &TableName) -> TransactionResult<Vec<Row>> {
        let commit = self.read_commit()?;
        self.repo
            .scan_table(table, commit)
            .map_err(TransactionError::from)
    }

    /// List all row keys in a table.
    pub fn list_keys(&self, table: &TableName) -> TransactionResult<Vec<RowKey>> {
        let commit = self.read_commit()?;
        self.repo
            .list_rows(table, commit)
            .map_err(TransactionError::from)
    }

    // ==================== Transaction Control ====================

    /// Update the transaction branch to point to current commit.
    fn update_branch(&self) -> TransactionResult<()> {
        self.repo
            .update_branch(&self.metadata.branch, self.metadata.current_commit)
            .map_err(TransactionError::from)
    }

    /// Commit the transaction.
    ///
    /// This attempts to fast-forward main to include our changes.
    /// Returns error if there are conflicts with concurrent transactions.
    pub fn commit(self) -> TransactionResult<Transaction<TxCommitted>> {
        // Check for conflicts by seeing if main has moved
        let main_head = self.repo.head()?;

        if main_head != self.metadata.base_commit {
            // Main has moved - check for conflicts
            let conflicts = self
                .repo
                .detect_conflicts(&self.metadata.branch, main_head)?;
            if !conflicts.is_empty() {
                // Clean up the branch before returning error
                let _ = self.repo.delete_transaction_branch(&self.metadata.tx_id);
                return Err(TransactionError::Conflict { paths: conflicts });
            }
        }

        // Fast-forward main to our commit
        match self
            .repo
            .fast_forward_main(&self.metadata.branch, self.metadata.base_commit)
        {
            Ok(_) => {}
            Err(StorageError::ConcurrentModification { .. }) => {
                // Another transaction just committed - retry detection
                let main_head = self.repo.head()?;
                let conflicts = self
                    .repo
                    .detect_conflicts(&self.metadata.branch, main_head)?;
                let _ = self.repo.delete_transaction_branch(&self.metadata.tx_id);
                return Err(TransactionError::Conflict { paths: conflicts });
            }
            Err(e) => {
                let _ = self.repo.delete_transaction_branch(&self.metadata.tx_id);
                return Err(TransactionError::Storage(e));
            }
        }

        // Clean up the transaction branch
        let _ = self.repo.delete_transaction_branch(&self.metadata.tx_id);

        Ok(Transaction {
            metadata: self.metadata,
            repo: self.repo,
            _state: PhantomData,
        })
    }

    /// Rollback the transaction.
    ///
    /// This simply deletes the transaction branch, discarding all changes.
    pub fn rollback(self) -> TransactionResult<Transaction<TxAborted>> {
        // Clean up the transaction branch
        let _ = self.repo.delete_transaction_branch(&self.metadata.tx_id);

        Ok(Transaction {
            metadata: self.metadata,
            repo: self.repo,
            _state: PhantomData,
        })
    }
}

impl Transaction<TxCommitted> {
    /// Get the final commit ID of the committed transaction.
    pub fn final_commit(&self) -> CommitId {
        self.metadata.current_commit
    }
}

impl Transaction<TxAborted> {
    /// Get the reason for abort (if available).
    pub fn was_rolled_back(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, GitRepository) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path()).unwrap();
        (dir, repo)
    }

    #[test]
    fn test_transaction_insert_read() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        // Create a transaction
        let branch = repo.create_transaction_branch("tx001", head).unwrap();
        let mut tx = Transaction::<TxActive>::new(
            repo.clone(),
            "tx001".to_string(),
            branch,
            head,
            IsolationLevel::ReadCommitted,
        );

        // Create table and insert row
        let table = TableName::new("users").unwrap();
        tx.create_table(&table).unwrap();

        let key = RowKey::new("user1").unwrap();
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), Value::String("Alice".to_string()));
        tx.insert_data(&table, key.clone(), data).unwrap();

        // Read within transaction
        let row = tx.read(&table, &key).unwrap().unwrap();
        assert_eq!(row.get("name"), Some(&Value::String("Alice".to_string())));

        // Commit
        let _committed = tx.commit().unwrap();

        // Verify on main
        let main_head = repo.head().unwrap();
        let row = repo.read_row(&table, &key, main_head).unwrap().unwrap();
        assert_eq!(row.get("name"), Some(&Value::String("Alice".to_string())));
    }

    #[test]
    fn test_transaction_rollback() {
        let (_dir, repo) = setup();
        let initial_head = repo.head().unwrap();

        // Create table and update main branch to include it
        let table = TableName::new("users").unwrap();
        let head_with_table = repo.create_table(&table, initial_head, None).unwrap();
        // Update main branch to point to the new commit with the table
        let main_branch = BranchName::main();
        repo.update_branch(&main_branch, head_with_table).unwrap();

        // Start transaction from the commit that has the table
        let branch = repo
            .create_transaction_branch("tx001", head_with_table)
            .unwrap();
        let mut tx = Transaction::<TxActive>::new(
            repo.clone(),
            "tx001".to_string(),
            branch,
            head_with_table,
            IsolationLevel::ReadCommitted,
        );

        // Insert row
        let key = RowKey::new("user1").unwrap();
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), Value::String("Alice".to_string()));
        tx.insert_data(&table, key.clone(), data).unwrap();

        // Rollback
        let _aborted = tx.rollback().unwrap();

        // Verify row doesn't exist on main (table exists, but row shouldn't)
        let main_head = repo.head().unwrap();
        let row = repo.read_row(&table, &key, main_head).unwrap();
        assert!(row.is_none());
    }

    #[test]
    fn test_isolation_levels() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        // Create table
        let table = TableName::new("users").unwrap();
        let head = repo.create_table(&table, head, None).unwrap();

        // Start transaction with ReadCommitted
        let branch = repo.create_transaction_branch("tx001", head).unwrap();
        let tx = Transaction::<TxActive>::new(
            repo.clone(),
            "tx001".to_string(),
            branch,
            head,
            IsolationLevel::ReadCommitted,
        );

        // Insert row outside transaction (simulating another committed transaction)
        let key = RowKey::new("user1").unwrap();
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), Value::String("Alice".to_string()));
        let row = Row::new(key.clone(), data);
        let _ = repo.insert_row(&table, row, head, None).unwrap();

        // Transaction reads from its own branch (snapshot isolation in practice)
        // So it won't see the row inserted outside
        let row = tx.read(&table, &key).unwrap();
        assert!(row.is_none());

        tx.rollback().unwrap();
    }

    #[test]
    fn test_transaction_sees_own_writes() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        // Create table
        let table = TableName::new("users").unwrap();
        let head = repo.create_table(&table, head, None).unwrap();

        // Start transaction
        let branch = repo.create_transaction_branch("tx001", head).unwrap();
        let mut tx = Transaction::<TxActive>::new(
            repo.clone(),
            "tx001".to_string(),
            branch,
            head,
            IsolationLevel::RepeatableRead,
        );

        // Insert a row in the transaction
        let key = RowKey::new("user1").unwrap();
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), Value::String("Alice".to_string()));
        tx.insert_data(&table, key.clone(), data).unwrap();

        // Transaction should see its own writes
        let row = tx.read(&table, &key).unwrap();
        assert!(row.is_some());
        assert_eq!(
            row.unwrap().get("name"),
            Some(&Value::String("Alice".to_string()))
        );

        tx.rollback().unwrap();
    }
}
