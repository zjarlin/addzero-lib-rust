//!   Core Git repository wrapper.
//!
//!  This is the central component of the storage layer.  It wraps `git2::Repository`
//!   with thread-safe access and provides high-level operations that the rest of
//!  the system uses.
//!
//! All other storage modules use this for Git access.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use git2::Repository;
use parking_lot::RwLock;

use crate::storage::blob::{self, Row};
use crate::storage::commit::{self, CommitBuilder, CommitInfo, CommitMessage};
use crate::storage::error::{StorageError, StorageResult};
use crate::storage::refs::RefManager;
use crate::storage::tree::TreeMutator;
use crate::storage::types::{BranchName, CommitId, GitSignature, RowKey, TableName, TreeId};

/// The main Git repository wrapper.
///
/// This provides thread-safe access to all Git operations.
/// Clone this to share across threads - it uses Arc internally.
#[derive(Clone)]
pub struct GitRepository {
    inner: Arc<GitRepositoryInner>,
}

struct GitRepositoryInner {
    repo: RwLock<Repository>,
    path: PathBuf,
    signature: GitSignature,
}

impl GitRepository {
    /// Open an existing repository.
    pub fn open(path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = path.as_ref();
        let repo =
            Repository::open(path).map_err(|_| StorageError::NotInitialized(path.to_path_buf()))?;

        Ok(Self {
            inner: Arc::new(GitRepositoryInner {
                repo: RwLock::new(repo),
                path: path.to_path_buf(),
                signature: GitSignature::gitdb(),
            }),
        })
    }

    /// Initialize a new repository.
    pub fn init(path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = path.as_ref();
        let repo = Repository::init(path)?;

        let storage = Self {
            inner: Arc::new(GitRepositoryInner {
                repo: RwLock::new(repo),
                path: path.to_path_buf(),
                signature: GitSignature::gitdb(),
            }),
        };

        // Create initial commit
        storage.with_repo(|repo| {
            let commit_id = commit::create_initial_commit(repo, &storage.inner.signature)?;
            RefManager::init_main_branch(repo, commit_id)?;
            Ok(())
        })?;

        Ok(storage)
    }

    /// Open or initialize a repository.
    pub fn open_or_init(path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = path.as_ref();
        if path.join(".git").exists() {
            Self::open(path)
        } else {
            Self::init(path)
        }
    }

    /// Get the repository path.
    pub fn path(&self) -> &Path {
        &self.inner.path
    }

    /// Set the signature for commits.
    pub fn with_signature(mut self, signature: GitSignature) -> Self {
        // We need to recreate the Arc with the new signature
        let inner = Arc::get_mut(&mut self.inner).expect("cannot modify shared repository");
        inner.signature = signature;
        self
    }

    /// Execute a function with read access to the repository.
    pub fn with_repo<F, T>(&self, f: F) -> StorageResult<T>
    where
        F: FnOnce(&Repository) -> StorageResult<T>,
    {
        let repo = self.inner.repo.read();
        f(&repo)
    }

    /// Execute a function with write access to the repository.
    pub fn with_repo_mut<F, T>(&self, f: F) -> StorageResult<T>
    where
        F: FnOnce(&Repository) -> StorageResult<T>,
    {
        let repo = self.inner.repo.write();
        f(&repo)
    }

    // ==================== High-level Operations ====================

    /// Get the current HEAD commit (tip of main branch).
    pub fn head(&self) -> StorageResult<CommitId> {
        self.with_repo(|repo| RefManager::head_commit(repo))
    }

    /// Get the commit ID for a branch.
    pub fn resolve_branch(&self, branch: &BranchName) -> StorageResult<CommitId> {
        self.with_repo(|repo| RefManager::resolve_branch(repo, branch))
    }

    /// Get information about a commit.
    pub fn get_commit(&self, id: CommitId) -> StorageResult<CommitInfo> {
        self.with_repo(|repo| commit::get_commit(repo, id))
    }

    /// Get the tree at a specific commit.
    pub fn tree_at(&self, commit_id: CommitId) -> StorageResult<TreeSnapshot> {
        self.with_repo(|repo| {
            let tree = commit::get_tree_at_commit(repo, commit_id)?;
            Ok(TreeSnapshot {
                tree_id: tree.id(),
                tables: tree.list_tables(),
            })
        })
    }

    // ==================== Table Operations ====================

    /// List all tables at a commit.
    pub fn list_tables(&self, at: CommitId) -> StorageResult<Vec<TableName>> {
        self.with_repo(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;
            Ok(tree.list_tables())
        })
    }

    /// Check if a table exists at a commit.
    pub fn table_exists(&self, table: &TableName, at: CommitId) -> StorageResult<bool> {
        self.with_repo(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;
            Ok(tree.table_exists(table))
        })
    }

    /// Create a new table.
    ///
    /// Returns the new commit ID.
    pub fn create_table(
        &self,
        table: &TableName,
        at: CommitId,
        tx_id: Option<&str>,
    ) -> StorageResult<CommitId> {
        self.with_repo_mut(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;
            let mut mutator = TreeMutator::from_tree(repo, &tree)?;
            mutator.create_table(table)?;
            let new_tree_id = mutator.write()?;

            let message = CommitMessage::create_table(table.as_str(), tx_id);
            CommitBuilder::new(repo)
                .tree(new_tree_id)
                .parent(at)
                .message(message)
                .signature(self.inner.signature.clone())
                .commit()
        })
    }

    /// Drop a table.
    ///
    /// Returns the new commit ID.
    pub fn drop_table(
        &self,
        table: &TableName,
        at: CommitId,
        tx_id: Option<&str>,
    ) -> StorageResult<CommitId> {
        self.with_repo_mut(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;
            let mut mutator = TreeMutator::from_tree(repo, &tree)?;
            mutator.drop_table(table)?;
            let new_tree_id = mutator.write()?;

            let message = CommitMessage::drop_table(table.as_str(), tx_id);
            CommitBuilder::new(repo)
                .tree(new_tree_id)
                .parent(at)
                .message(message)
                .signature(self.inner.signature.clone())
                .commit()
        })
    }

    // ==================== Row Operations ====================

    /// List all row keys in a table.
    pub fn list_rows(&self, table: &TableName, at: CommitId) -> StorageResult<Vec<RowKey>> {
        self.with_repo(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;
            tree.list_rows(repo, table)
        })
    }

    /// Read a row from a table.
    pub fn read_row(
        &self,
        table: &TableName,
        key: &RowKey,
        at: CommitId,
    ) -> StorageResult<Option<Row>> {
        self.with_repo(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;

            // Get the blob ID for the row
            let blob_id = match tree.get_row_blob_id(repo, table, key)? {
                Some(id) => id,
                None => return Ok(None),
            };

            // Read and deserialize the blob
            let bytes = blob::read_blob(repo, blob_id)?;
            let row = blob::deserialize_row(&bytes, key)?;
            Ok(Some(row))
        })
    }

    /// Insert a new row into a table.
    ///
    /// Fails if the row already exists.
    /// Returns the new commit ID.
    pub fn insert_row(
        &self,
        table: &TableName,
        row: Row,
        at: CommitId,
        tx_id: Option<&str>,
    ) -> StorageResult<CommitId> {
        self.with_repo_mut(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;

            // Check if row already exists
            if tree.row_exists(repo, table, &row.key)? {
                return Err(StorageError::RowAlreadyExists {
                    table: table.clone(),
                    key: row.key.clone(),
                });
            }

            // Write the row as a blob
            let blob_id = blob::write_blob(repo, &row)?;

            // Update the tree
            let mut mutator = TreeMutator::from_tree(repo, &tree)?;
            mutator.upsert_row(table, &row.key, blob_id)?;
            let new_tree_id = mutator.write()?;

            // Create commit
            let message = CommitMessage::insert(table.as_str(), row.key.as_str(), tx_id);
            CommitBuilder::new(repo)
                .tree(new_tree_id)
                .parent(at)
                .message(message)
                .signature(self.inner.signature.clone())
                .commit()
        })
    }

    /// Update an existing row.
    ///
    /// Fails if the row doesn't exist.
    /// Returns the new commit ID.
    pub fn update_row(
        &self,
        table: &TableName,
        row: Row,
        at: CommitId,
        tx_id: Option<&str>,
    ) -> StorageResult<CommitId> {
        self.with_repo_mut(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;

            // Check if row exists
            if !tree.row_exists(repo, table, &row.key)? {
                return Err(StorageError::RowNotFound {
                    table: table.clone(),
                    key: row.key.clone(),
                });
            }

            // Write the row as a blob
            let blob_id = blob::write_blob(repo, &row)?;

            // Update the tree
            let mut mutator = TreeMutator::from_tree(repo, &tree)?;
            mutator.upsert_row(table, &row.key, blob_id)?;
            let new_tree_id = mutator.write()?;

            // Create commit
            let message = CommitMessage::update(table.as_str(), row.key.as_str(), tx_id);
            CommitBuilder::new(repo)
                .tree(new_tree_id)
                .parent(at)
                .message(message)
                .signature(self.inner.signature.clone())
                .commit()
        })
    }

    /// Insert or update a row (upsert).
    ///
    /// Returns the new commit ID.
    pub fn upsert_row(
        &self,
        table: &TableName,
        row: Row,
        at: CommitId,
        tx_id: Option<&str>,
    ) -> StorageResult<CommitId> {
        self.with_repo_mut(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;
            let exists = tree.row_exists(repo, table, &row.key)?;

            // Write the row as a blob
            let blob_id = blob::write_blob(repo, &row)?;

            // Update the tree
            let mut mutator = TreeMutator::from_tree(repo, &tree)?;
            mutator.upsert_row(table, &row.key, blob_id)?;
            let new_tree_id = mutator.write()?;

            // Create commit with appropriate message
            let message = if exists {
                CommitMessage::update(table.as_str(), row.key.as_str(), tx_id)
            } else {
                CommitMessage::insert(table.as_str(), row.key.as_str(), tx_id)
            };

            CommitBuilder::new(repo)
                .tree(new_tree_id)
                .parent(at)
                .message(message)
                .signature(self.inner.signature.clone())
                .commit()
        })
    }

    /// Delete a row from a table.
    ///
    /// Fails if the row doesn't exist.
    /// Returns the new commit ID.
    pub fn delete_row(
        &self,
        table: &TableName,
        key: &RowKey,
        at: CommitId,
        tx_id: Option<&str>,
    ) -> StorageResult<CommitId> {
        self.with_repo_mut(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;

            // Update the tree (delete_row checks existence)
            let mut mutator = TreeMutator::from_tree(repo, &tree)?;
            mutator.delete_row(table, key)?;
            let new_tree_id = mutator.write()?;

            // Create commit
            let message = CommitMessage::delete(table.as_str(), key.as_str(), tx_id);
            CommitBuilder::new(repo)
                .tree(new_tree_id)
                .parent(at)
                .message(message)
                .signature(self.inner.signature.clone())
                .commit()
        })
    }

    /// Scan all rows in a table.
    ///
    /// Warning: This reads all rows into memory.  Use with caution on large tables.
    pub fn scan_table(&self, table: &TableName, at: CommitId) -> StorageResult<Vec<Row>> {
        self.with_repo(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;
            let keys = tree.list_rows(repo, table)?;

            let mut rows = Vec::with_capacity(keys.len());
            for key in keys {
                let blob_id = tree.get_row_blob_id(repo, table, &key)?.ok_or_else(|| {
                    StorageError::RowNotFound {
                        table: table.clone(),
                        key: key.clone(),
                    }
                })?;

                let bytes = blob::read_blob(repo, blob_id)?;
                let row = blob::deserialize_row(&bytes, &key)?;
                rows.push(row);
            }

            Ok(rows)
        })
    }

    // ==================== Branch Operations ====================

    /// Create a new branch at the given commit.
    pub fn create_branch(&self, branch: &BranchName, at: CommitId) -> StorageResult<()> {
        self.with_repo_mut(|repo| RefManager::create_branch(repo, branch, at))
    }

    /// Delete a branch.
    pub fn delete_branch(&self, branch: &BranchName) -> StorageResult<()> {
        self.with_repo_mut(|repo| RefManager::delete_branch(repo, branch))
    }

    /// Update a branch to point to a new commit.
    pub fn update_branch(&self, branch: &BranchName, target: CommitId) -> StorageResult<()> {
        self.with_repo_mut(|repo| RefManager::update_branch(repo, branch, target))
    }

    /// Check if a branch exists.
    pub fn branch_exists(&self, branch: &BranchName) -> StorageResult<bool> {
        self.with_repo(|repo| Ok(RefManager::branch_exists(repo, branch)))
    }

    /// List all branches.
    pub fn list_branches(&self) -> StorageResult<Vec<BranchName>> {
        self.with_repo(|repo| RefManager::list_branches(repo, None))
    }

    /// Create a transaction branch.
    pub fn create_transaction_branch(
        &self,
        tx_id: &str,
        base: CommitId,
    ) -> StorageResult<BranchName> {
        self.with_repo_mut(|repo| RefManager::create_transaction_branch(repo, tx_id, base))
    }

    /// Delete a transaction branch.
    pub fn delete_transaction_branch(&self, tx_id: &str) -> StorageResult<()> {
        self.with_repo_mut(|repo| RefManager::delete_transaction_branch(repo, tx_id))
    }

    // ==================== Merge Operations ====================

    /// Fast-forward main to a transaction branch.
    ///
    /// Only succeeds if main hasn't moved since the transaction started.
    /// Returns error if there are conflicts (main was updated).
    pub fn fast_forward_main(
        &self,
        tx_branch: &BranchName,
        expected_main: CommitId,
    ) -> StorageResult<CommitId> {
        self.with_repo_mut(|repo| {
            let tx_commit = RefManager::resolve_branch(repo, tx_branch)?;
            let main = BranchName::main();

            RefManager::update_branch_if_unchanged(repo, &main, expected_main, tx_commit)?;

            Ok(tx_commit)
        })
    }

    /// Detect conflicts between a transaction branch and main.
    ///
    /// Returns the list of conflicting paths.
    pub fn detect_conflicts(
        &self,
        tx_branch: &BranchName,
        main_head: CommitId,
    ) -> StorageResult<Vec<PathBuf>> {
        self.with_repo(|repo| {
            let tx_commit = RefManager::resolve_branch(repo, tx_branch)?;
            commit::detect_conflicts(repo, tx_commit, main_head)
        })
    }

    /// Get the merge base between a transaction branch and main.
    pub fn merge_base(&self, tx_branch: &BranchName) -> StorageResult<Option<CommitId>> {
        self.with_repo(|repo| {
            let tx_commit = RefManager::resolve_branch(repo, tx_branch)?;
            let main_commit = RefManager::resolve_branch(repo, &BranchName::main())?;
            commit::find_merge_base(repo, tx_commit, main_commit)
        })
    }

    /// Get commit history.
    pub fn history(&self, from: CommitId, limit: Option<usize>) -> StorageResult<Vec<CommitInfo>> {
        self.with_repo(|repo| {
            let iter = commit::history(repo, from)?;
            let commits: Result<Vec<_>, _> = match limit {
                Some(n) => iter.take(n).collect(),
                None => iter.collect(),
            };
            commits
        })
    }

    /// Get diff between two commits.
    pub fn diff(
        &self,
        old: CommitId,
        new: CommitId,
    ) -> StorageResult<Vec<crate::storage::types::Change>> {
        self.with_repo(|repo| commit::diff_commits(repo, old, new))
    }

    // ==================== Utility Operations ====================

    /// Count rows in a table.
    pub fn count_rows(&self, table: &TableName, at: CommitId) -> StorageResult<usize> {
        Ok(self.list_rows(table, at)?.len())
    }

    /// Get statistics about the repository.
    pub fn stats(&self, at: CommitId) -> StorageResult<RepositoryStats> {
        self.with_repo(|repo| {
            let tree = commit::get_tree_at_commit(repo, at)?;
            let tables = tree.list_tables();
            let mut total_rows = 0;

            for table in &tables {
                total_rows += tree.list_rows(repo, table)?.len();
            }

            let branches = RefManager::list_branches(repo, None)?;
            let tx_branches = RefManager::list_transaction_branches(repo)?;

            Ok(RepositoryStats {
                table_count: tables.len(),
                total_rows,
                branch_count: branches.len(),
                active_transactions: tx_branches.len(),
            })
        })
    }
}

/// A snapshot of the tree structure at a commit.
#[derive(Debug, Clone)]
pub struct TreeSnapshot {
    pub tree_id: TreeId,
    pub tables: Vec<TableName>,
}

/// Statistics about the repository.
#[derive(Debug, Clone)]
pub struct RepositoryStats {
    pub table_count: usize,
    pub total_rows: usize,
    pub branch_count: usize,
    pub active_transactions: usize,
}

impl std::fmt::Display for RepositoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Repository Statistics:")?;
        writeln!(f, "  Tables: {}", self.table_count)?;
        writeln!(f, "  Total Rows: {}", self.total_rows)?;
        writeln!(f, "  Branches: {}", self.branch_count)?;
        writeln!(f, "  Active Transactions: {}", self.active_transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    fn setup() -> (TempDir, GitRepository) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path()).unwrap();
        (dir, repo)
    }

    #[test]
    fn test_init_and_open() {
        let dir = TempDir::new().unwrap();

        // Init
        let repo = GitRepository::init(dir.path()).unwrap();
        let head1 = repo.head().unwrap();

        // Open
        drop(repo);
        let repo = GitRepository::open(dir.path()).unwrap();
        let head2 = repo.head().unwrap();

        assert_eq!(head1, head2);
    }

    #[test]
    fn test_open_or_init() {
        let dir = TempDir::new().unwrap();

        // First call inits
        let repo1 = GitRepository::open_or_init(dir.path()).unwrap();
        let head1 = repo1.head().unwrap();

        // Second call opens
        drop(repo1);
        let repo2 = GitRepository::open_or_init(dir.path()).unwrap();
        let head2 = repo2.head().unwrap();

        assert_eq!(head1, head2);
    }

    #[test]
    fn test_table_crud() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        // Create table
        let table = TableName::new("users").unwrap();
        let new_head = repo.create_table(&table, head, None).unwrap();

        // List tables
        let tables = repo.list_tables(new_head).unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].as_str(), "users");

        // Table exists
        assert!(repo.table_exists(&table, new_head).unwrap());

        // Drop table
        let final_head = repo.drop_table(&table, new_head, None).unwrap();
        let tables = repo.list_tables(final_head).unwrap();
        assert!(tables.is_empty());
    }

    #[test]
    fn test_row_crud() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        // Create table
        let table = TableName::new("users").unwrap();
        let head = repo.create_table(&table, head, None).unwrap();

        // Insert row
        let key = RowKey::new("user1").unwrap();
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), serde_json::json!("Alice"));
        data.insert("age".to_string(), serde_json::json!(30));
        let row = Row::new(key.clone(), data);

        let head = repo.insert_row(&table, row, head, None).unwrap();

        // Read row
        let read_row = repo.read_row(&table, &key, head).unwrap().unwrap();
        assert_eq!(read_row.key, key);
        assert_eq!(read_row.get("name"), Some(&serde_json::json!("Alice")));

        // Update row
        let mut new_data = BTreeMap::new();
        new_data.insert("name".to_string(), serde_json::json!("Alice Smith"));
        new_data.insert("age".to_string(), serde_json::json!(31));
        let updated_row = read_row.with_update(new_data);

        let head = repo.update_row(&table, updated_row, head, None).unwrap();

        // Verify update
        let read_row = repo.read_row(&table, &key, head).unwrap().unwrap();
        assert_eq!(
            read_row.get("name"),
            Some(&serde_json::json!("Alice Smith"))
        );
        assert_eq!(read_row.version, 2);

        // Delete row
        let head = repo.delete_row(&table, &key, head, None).unwrap();

        // Verify delete
        let read_row = repo.read_row(&table, &key, head).unwrap();
        assert!(read_row.is_none());
    }

    #[test]
    fn test_scan_table() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        let table = TableName::new("items").unwrap();
        let mut head = repo.create_table(&table, head, None).unwrap();

        // Insert multiple rows
        for i in 0..5 {
            let key = RowKey::new(format!("item{}", i)).unwrap();
            let mut data = BTreeMap::new();
            data.insert("value".to_string(), serde_json::json!(i));
            let row = Row::new(key, data);
            head = repo.insert_row(&table, row, head, None).unwrap();
        }

        // Scan
        let rows = repo.scan_table(&table, head).unwrap();
        assert_eq!(rows.len(), 5);
    }

    #[test]
    fn test_branch_operations() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        let branch = BranchName::new("feature").unwrap();

        // Create branch
        repo.create_branch(&branch, head).unwrap();
        assert!(repo.branch_exists(&branch).unwrap());

        // Resolve branch
        let resolved = repo.resolve_branch(&branch).unwrap();
        assert_eq!(resolved, head);

        // List branches
        let branches = repo.list_branches().unwrap();
        assert!(branches.iter().any(|b| b.as_str() == "feature"));

        // Delete branch
        repo.delete_branch(&branch).unwrap();
        assert!(!repo.branch_exists(&branch).unwrap());
    }

    #[test]
    fn test_transaction_branch() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        // Create transaction branch
        let tx_branch = repo.create_transaction_branch("tx001", head).unwrap();
        assert!(tx_branch.is_transaction_branch());
        assert_eq!(tx_branch.transaction_id(), Some("tx001"));

        // Do work on transaction branch
        let table = TableName::new("test").unwrap();
        let tx_head = repo.create_table(&table, head, Some("tx001")).unwrap();
        repo.update_branch(&tx_branch, tx_head).unwrap();

        // Fast-forward main
        repo.fast_forward_main(&tx_branch, head).unwrap();

        // Cleanup
        repo.delete_transaction_branch("tx001").unwrap();

        // Verify main has the table
        let main_head = repo.head().unwrap();
        assert!(repo.table_exists(&table, main_head).unwrap());
    }

    #[test]
    fn test_history() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        // Create some commits
        let table = TableName::new("test").unwrap();
        let head = repo.create_table(&table, head, None).unwrap();

        let key = RowKey::new("row1").unwrap();
        let row = Row::new(key, BTreeMap::new());
        let head = repo.insert_row(&table, row, head, None).unwrap();

        // Get history
        let history = repo.history(head, Some(10)).unwrap();
        assert!(history.len() >= 3); // init + create table + insert

        // First should be most recent
        assert_eq!(history[0].id, head);
    }

    #[test]
    fn test_stats() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        let stats = repo.stats(head).unwrap();
        assert_eq!(stats.table_count, 0);
        assert_eq!(stats.total_rows, 0);

        // Add data
        let table = TableName::new("test").unwrap();
        let head = repo.create_table(&table, head, None).unwrap();

        let key = RowKey::new("row1").unwrap();
        let row = Row::new(key, BTreeMap::new());
        let head = repo.insert_row(&table, row, head, None).unwrap();

        let stats = repo.stats(head).unwrap();
        assert_eq!(stats.table_count, 1);
        assert_eq!(stats.total_rows, 1);
    }

    #[test]
    fn test_insert_duplicate_fails() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        let table = TableName::new("test").unwrap();
        let head = repo.create_table(&table, head, None).unwrap();

        let key = RowKey::new("row1").unwrap();
        let row = Row::new(key.clone(), BTreeMap::new());
        let head = repo.insert_row(&table, row.clone(), head, None).unwrap();

        // Try to insert again
        let result = repo.insert_row(&table, Row::new(key, BTreeMap::new()), head, None);
        assert!(matches!(result, Err(StorageError::RowAlreadyExists { .. })));
    }

    #[test]
    fn test_update_nonexistent_fails() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        let table = TableName::new("test").unwrap();
        let head = repo.create_table(&table, head, None).unwrap();

        let key = RowKey::new("nonexistent").unwrap();
        let row = Row::new(key, BTreeMap::new());

        let result = repo.update_row(&table, row, head, None);
        assert!(matches!(result, Err(StorageError::RowNotFound { .. })));
    }

    #[test]
    fn test_delete_nonexistent_fails() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        let table = TableName::new("test").unwrap();
        let head = repo.create_table(&table, head, None).unwrap();

        let key = RowKey::new("nonexistent").unwrap();

        let result = repo.delete_row(&table, &key, head, None);
        assert!(matches!(result, Err(StorageError::RowNotFound { .. })));
    }

    #[test]
    fn test_concurrent_modification_detection() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        let tx_branch = repo.create_transaction_branch("tx001", head).unwrap();

        // Simulate main moving forward
        let table = TableName::new("main_table").unwrap();
        let new_main = repo.create_table(&table, head, None).unwrap();
        repo.update_branch(&BranchName::main(), new_main).unwrap();

        // Try to fast-forward with stale expected_main
        let result = repo.fast_forward_main(&tx_branch, head);
        assert!(matches!(
            result,
            Err(StorageError::ConcurrentModification { .. })
        ));
    }
}
