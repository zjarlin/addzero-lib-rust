//!  Branch and reference management.
//!
//!  Git refs are pointers to commits.  This module handles:
//! - Main branch management
//! - Transaction branch lifecycle (create, update, delete)
//! - Ref resolution and validation
//!
//! Transaction branches use a namespaced pattern: `tx/{transaction_id}`
//! This makes them easy to identify and clean up.

use git2::{BranchType, Repository};

use crate::storage::error::{StorageError, StorageResult};
use crate::storage::types::{BranchName, CommitId};

/// Manages Git references (branches).
pub struct RefManager;

impl RefManager {
    /// Resolve a branch name to its current commit ID.
    pub fn resolve_branch(repo: &Repository, branch: &BranchName) -> StorageResult<CommitId> {
        let reference = repo
            .find_reference(&branch.as_ref_path())
            .map_err(|_| StorageError::RefNotFound(branch.to_string()))?;

        let commit = reference
            .peel_to_commit()
            .map_err(|_| StorageError::RefNotFound(branch.to_string()))?;

        Ok(CommitId::new(commit.id()))
    }

    /// Get the current HEAD commit (usually main branch).
    pub fn head_commit(repo: &Repository) -> StorageResult<CommitId> {
        let head = repo.head().map_err(|e| {
            if e.code() == git2::ErrorCode::UnbornBranch {
                StorageError::EmptyRepository
            } else {
                StorageError::Git(e)
            }
        })?;

        let commit = head.peel_to_commit()?;
        Ok(CommitId::new(commit.id()))
    }

    /// Check if a branch exists.
    pub fn branch_exists(repo: &Repository, branch: &BranchName) -> bool {
        repo.find_reference(&branch.as_ref_path()).is_ok()
    }

    /// Create a new branch pointing to the given commit.
    pub fn create_branch(
        repo: &Repository,
        branch: &BranchName,
        target: CommitId,
    ) -> StorageResult<()> {
        if Self::branch_exists(repo, branch) {
            return Err(StorageError::BranchAlreadyExists(branch.to_string()));
        }

        let commit = repo.find_commit(target.raw())?;
        repo.branch(branch.as_str(), &commit, false)?;

        Ok(())
    }

    /// Update a branch to point to a new commit.
    ///
    /// This is a force update - use `update_branch_if_unchanged` for safe updates.
    pub fn update_branch(
        repo: &Repository,
        branch: &BranchName,
        target: CommitId,
    ) -> StorageResult<()> {
        let mut reference = repo
            .find_reference(&branch.as_ref_path())
            .map_err(|_| StorageError::RefNotFound(branch.to_string()))?;

        reference.set_target(
            target.raw(),
            &format!("update branch to {}", target.short()),
        )?;

        Ok(())
    }

    /// Update a branch only if it still points to the expected commit.
    ///
    /// This provides compare-and-swap semantics for safe concurrent updates.
    /// Returns error if the branch was modified by another transaction.
    pub fn update_branch_if_unchanged(
        repo: &Repository,
        branch: &BranchName,
        expected: CommitId,
        new_target: CommitId,
    ) -> StorageResult<()> {
        let current = Self::resolve_branch(repo, branch)?;

        if current != expected {
            return Err(StorageError::ConcurrentModification {
                branch: branch.to_string(),
            });
        }

        Self::update_branch(repo, branch, new_target)
    }

    /// Delete a branch.
    pub fn delete_branch(repo: &Repository, branch: &BranchName) -> StorageResult<()> {
        let mut git_branch = repo
            .find_branch(branch.as_str(), BranchType::Local)
            .map_err(|_| StorageError::RefNotFound(branch.to_string()))?;

        git_branch.delete()?;

        Ok(())
    }

    /// List all branches with an optional prefix filter.
    pub fn list_branches(
        repo: &Repository,
        prefix: Option<&str>,
    ) -> StorageResult<Vec<BranchName>> {
        let branches = repo.branches(Some(BranchType::Local))?;

        let mut result = Vec::new();
        for branch_result in branches {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                let matches = match prefix {
                    Some(p) => name.starts_with(p),
                    None => true,
                };
                if matches {
                    if let Ok(branch_name) = BranchName::new(name) {
                        result.push(branch_name);
                    }
                }
            }
        }

        Ok(result)
    }

    /// List all transaction branches.
    pub fn list_transaction_branches(repo: &Repository) -> StorageResult<Vec<BranchName>> {
        Self::list_branches(repo, Some(BranchName::TX_PREFIX))
    }

    /// Create a new transaction branch.
    ///
    /// Returns the branch name (with tx/ prefix).
    pub fn create_transaction_branch(
        repo: &Repository,
        tx_id: &str,
        base: CommitId,
    ) -> StorageResult<BranchName> {
        let branch = BranchName::for_transaction(tx_id);
        Self::create_branch(repo, &branch, base)?;
        Ok(branch)
    }

    /// Delete a transaction branch (cleanup after commit/rollback).
    pub fn delete_transaction_branch(repo: &Repository, tx_id: &str) -> StorageResult<()> {
        let branch = BranchName::for_transaction(tx_id);
        Self::delete_branch(repo, &branch)
    }

    /// Clean up old transaction branches (for recovery after crashes).
    ///
    /// In a real implementation, you'd check timestamps and only delete
    /// branches older than a threshold.  For now, this deletes all tx branches.
    pub fn cleanup_abandoned_transactions(repo: &Repository) -> StorageResult<usize> {
        let tx_branches = Self::list_transaction_branches(repo)?;
        let mut deleted = 0;

        for branch in tx_branches {
            if let Some(tx_id) = branch.transaction_id() {
                // In production, you'd check if this transaction is still active
                // For now, we just delete all
                if Self::delete_transaction_branch(repo, tx_id).is_ok() {
                    deleted += 1;
                }
            }
        }

        Ok(deleted)
    }

    /// Initialize the main branch if it doesn't exist.
    ///
    /// This should be called after creating the initial commit.
    /// Also ensures HEAD points to main.
    pub fn init_main_branch(repo: &Repository, initial_commit: CommitId) -> StorageResult<()> {
        let main = BranchName::main();

        if !Self::branch_exists(repo, &main) {
            Self::create_branch(repo, &main, initial_commit)?;
        }

        // Ensure HEAD points to main branch
        repo.set_head(&main.as_ref_path())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::tree::create_initial_tree;
    use tempfile::TempDir;

    fn setup_repo_with_commit() -> (TempDir, Repository, CommitId) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        // Create initial commit (scope the borrows)
        let commit_id = {
            let tree_id = create_initial_tree(&repo).unwrap();
            let tree = repo.find_tree(tree_id.raw()).unwrap();
            let sig = git2::Signature::now("Test", "test@test.com").unwrap();

            let commit_oid = repo
                .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                .unwrap();

            CommitId::new(commit_oid)
        };

        // Ensure main branch exists
        let _ = RefManager::init_main_branch(&repo, commit_id); // Might already exist via HEAD

        (dir, repo, commit_id)
    }

    #[test]
    fn test_head_commit() {
        let (_dir, repo, expected) = setup_repo_with_commit();
        let head = RefManager::head_commit(&repo).unwrap();
        assert_eq!(head, expected);
    }

    #[test]
    fn test_branch_lifecycle() {
        let (_dir, repo, base_commit) = setup_repo_with_commit();

        let branch = BranchName::new("feature").unwrap();

        // Create
        assert!(!RefManager::branch_exists(&repo, &branch));
        RefManager::create_branch(&repo, &branch, base_commit).unwrap();
        assert!(RefManager::branch_exists(&repo, &branch));

        // Resolve
        let resolved = RefManager::resolve_branch(&repo, &branch).unwrap();
        assert_eq!(resolved, base_commit);

        // Delete
        RefManager::delete_branch(&repo, &branch).unwrap();
        assert!(!RefManager::branch_exists(&repo, &branch));
    }

    #[test]
    fn test_duplicate_branch_error() {
        let (_dir, repo, base_commit) = setup_repo_with_commit();
        let branch = BranchName::new("feature").unwrap();

        RefManager::create_branch(&repo, &branch, base_commit).unwrap();
        let result = RefManager::create_branch(&repo, &branch, base_commit);

        assert!(matches!(result, Err(StorageError::BranchAlreadyExists(_))));
    }

    #[test]
    fn test_transaction_branches() {
        let (_dir, repo, base_commit) = setup_repo_with_commit();

        // Create transaction branches
        let branch1 = RefManager::create_transaction_branch(&repo, "tx001", base_commit).unwrap();
        let branch2 = RefManager::create_transaction_branch(&repo, "tx002", base_commit).unwrap();

        assert!(branch1.is_transaction_branch());
        assert!(branch2.is_transaction_branch());
        assert_eq!(branch1.transaction_id(), Some("tx001"));

        // List transaction branches
        let tx_branches = RefManager::list_transaction_branches(&repo).unwrap();
        assert_eq!(tx_branches.len(), 2);

        // Cleanup
        let deleted = RefManager::cleanup_abandoned_transactions(&repo).unwrap();
        assert_eq!(deleted, 2);

        let remaining = RefManager::list_transaction_branches(&repo).unwrap();
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_update_branch_if_unchanged() {
        let (_dir, repo, commit1) = setup_repo_with_commit();
        let branch = BranchName::new("test").unwrap();

        RefManager::create_branch(&repo, &branch, commit1).unwrap();

        // Create another commit
        let tree_id = create_initial_tree(&repo).unwrap();
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();
        let parent = repo.find_commit(commit1.raw()).unwrap();
        let commit2_oid = repo
            .commit(None, &sig, &sig, "Second commit", &tree, &[&parent])
            .unwrap();
        let commit2 = CommitId::new(commit2_oid);

        // Update should succeed
        RefManager::update_branch_if_unchanged(&repo, &branch, commit1, commit2).unwrap();

        // Update with wrong expected should fail
        let result = RefManager::update_branch_if_unchanged(&repo, &branch, commit1, commit2);
        assert!(matches!(
            result,
            Err(StorageError::ConcurrentModification { .. })
        ));
    }
}
