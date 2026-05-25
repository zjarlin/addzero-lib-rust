//!  Commit creation and history traversal
//!
//!  commits are the atomic units of change in Git. In GitDB:
//! - each write operation creates a commit
//! - transactions accumulate commits on a branch
//! - merging branches requires commit ancestry analysis
//!
//! this module handles commit creation, history walking, and diff operations

use std::path::PathBuf;

use az_derive_aliases::{apply, plain_clone_debug};
use chrono::{DateTime, TimeZone, Utc};
use git2::{Delta, Diff, DiffOptions, Repository, Revwalk, Sort};

use crate::storage::error::{StorageError, StorageResult};
use crate::storage::tree::TreeHandle;
use crate::storage::types::{Change, ChangeStatus, CommitId, GitSignature, TreeId};

/// information about a commit
#[apply(plain_clone_debug)]
pub struct CommitInfo {
    pub id: CommitId,
    pub tree_id: TreeId,
    pub parent_ids: Vec<CommitId>,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: DateTime<Utc>,
}

impl CommitInfo {
    /// create CommitInfo from a git2::Commit
    pub(crate) fn from_git2(commit: &git2::Commit<'_>) -> Self {
        let author = commit.author();
        let time = commit.time();
        let timestamp = Utc
            .timestamp_opt(time.seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        Self {
            id: CommitId::new(commit.id()),
            tree_id: TreeId::new(commit.tree_id()),
            parent_ids: commit.parent_ids().map(CommitId::new).collect(),
            message: commit.message().unwrap_or("").to_string(),
            author_name: author.name().unwrap_or("Unknown").to_string(),
            author_email: author.email().unwrap_or("unknown@unknown").to_string(),
            timestamp,
        }
    }

    /// check if this is a merge commit (has multiple parents)
    pub fn is_merge(&self) -> bool {
        self.parent_ids.len() > 1
    }

    /// get the first (or only) parent
    pub fn first_parent(&self) -> Option<CommitId> {
        self.parent_ids.first().copied()
    }

    /// get a short summary of the commit (first line of message)
    pub fn summary(&self) -> &str {
        self.message.lines().next().unwrap_or(&self.message)
    }
}

/// builder for creating commits with a fluent interface
pub struct CommitBuilder<'a> {
    repo: &'a Repository,
    tree_id: Option<TreeId>,
    parents: Vec<CommitId>,
    message: String,
    signature: GitSignature,
    update_ref: Option<String>,
}

impl<'a> CommitBuilder<'a> {
    /// create a new CommitBuilder
    pub fn new(repo: &'a Repository) -> Self {
        Self {
            repo,
            tree_id: None,
            parents: Vec::new(),
            message: String::new(),
            signature: GitSignature::gitdb(),
            update_ref: None,
        }
    }

    /// set the tree for this commit
    pub fn tree(mut self, tree_id: TreeId) -> Self {
        self.tree_id = Some(tree_id);
        self
    }

    /// add a parent commit
    pub fn parent(mut self, parent: CommitId) -> Self {
        self.parents.push(parent);
        self
    }

    /// set multiple parents (for merge commits)
    pub fn parents(mut self, parents: Vec<CommitId>) -> Self {
        self.parents = parents;
        self
    }

    /// set the commit message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// set the author/committer signature
    pub fn signature(mut self, signature: GitSignature) -> Self {
        self.signature = signature;
        self
    }

    /// update a ref (branch) to point to this commit
    pub fn update_ref(mut self, refname: impl Into<String>) -> Self {
        self.update_ref = Some(refname.into());
        self
    }

    /// create the commit and return its ID
    pub fn commit(self) -> StorageResult<CommitId> {
        let tree_id = self
            .tree_id
            .ok_or_else(|| StorageError::Internal("commit requires a tree".to_string()))?;

        let tree = self.repo.find_tree(tree_id.raw())?;
        let sig = self.signature.to_git2_signature()?;

        // collect parent commits
        let parent_commits: Vec<git2::Commit<'_>> = self
            .parents
            .iter()
            .map(|id| self.repo.find_commit(id.raw()))
            .collect::<Result<_, _>>()?;

        let parent_refs: Vec<&git2::Commit<'_>> = parent_commits.iter().collect();

        let oid = self.repo.commit(
            self.update_ref.as_deref(),
            &sig,
            &sig,
            &self.message,
            &tree,
            &parent_refs,
        )?;

        Ok(CommitId::new(oid))
    }
}

/// get information about a commit
pub fn get_commit(repo: &Repository, id: CommitId) -> StorageResult<CommitInfo> {
    let commit = repo
        .find_commit(id.raw())
        .map_err(|_| StorageError::CommitNotFound(id.to_string()))?;

    Ok(CommitInfo::from_git2(&commit))
}

/// get the tree snapshot at a specific commit
pub fn get_tree_at_commit(repo: &Repository, commit_id: CommitId) -> StorageResult<TreeHandle<'_>> {
    let commit = repo
        .find_commit(commit_id.raw())
        .map_err(|_| StorageError::CommitNotFound(commit_id.to_string()))?;

    let tree = commit.tree()?;
    Ok(TreeHandle::new(tree))
}

/// create the initial commit for a new repository
pub fn create_initial_commit(
    repo: &Repository,
    signature: &GitSignature,
) -> StorageResult<CommitId> {
    let tree_id = crate::storage::tree::create_initial_tree(repo)?;

    CommitBuilder::new(repo)
        .tree(tree_id)
        .message("[gitdb] Initialize repository")
        .signature(signature.clone())
        .update_ref("HEAD")
        .commit()
}

/// compute the diff between two commits
///
/// returns a list of changed paths
pub fn diff_commits(repo: &Repository, old: CommitId, new: CommitId) -> StorageResult<Vec<Change>> {
    let old_commit = repo.find_commit(old.raw())?;
    let new_commit = repo.find_commit(new.raw())?;

    let old_tree = old_commit.tree()?;
    let new_tree = new_commit.tree()?;

    let mut opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut opts))?;

    let changes = extract_changes_from_diff(&diff)?;
    Ok(changes)
}

/// compute changes from a diff
fn extract_changes_from_diff(diff: &Diff<'_>) -> StorageResult<Vec<Change>> {
    let mut changes = Vec::new();

    for delta in diff.deltas() {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(PathBuf::from)
            .unwrap_or_default();

        let status = match delta.status() {
            Delta::Added => ChangeStatus::Added,
            Delta::Deleted => ChangeStatus::Deleted,
            Delta::Modified => ChangeStatus::Modified,
            Delta::Renamed => ChangeStatus::Renamed,
            Delta::Copied => ChangeStatus::Copied,
            _ => ChangeStatus::Other,
        };

        changes.push(Change { path, status });
    }

    Ok(changes)
}

/// find the merge base (common ancestor) of two commits
///
/// returns None if there is no common ancestor
pub fn find_merge_base(
    repo: &Repository,
    a: CommitId,
    b: CommitId,
) -> StorageResult<Option<CommitId>> {
    match repo.merge_base(a.raw(), b.raw()) {
        Ok(oid) => Ok(Some(CommitId::new(oid))),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(StorageError::Git(e)),
    }
}

/// iterate over commit history starting from a commit
pub struct HistoryIterator<'repo> {
    repo: &'repo Repository,
    revwalk: Revwalk<'repo>,
}

impl<'repo> HistoryIterator<'repo> {
    /// create a new history iterator
    pub fn new(repo: &'repo Repository, start: CommitId) -> StorageResult<Self> {
        let mut revwalk = repo.revwalk()?;
        revwalk.push(start.raw())?;
        revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;

        Ok(Self { repo, revwalk })
    }

    /// only follow first parents (linear history through merges)
    pub fn first_parent_only(mut self) -> Self {
        self.revwalk.simplify_first_parent().ok();
        self
    }
}

impl<'repo> Iterator for HistoryIterator<'repo> {
    type Item = StorageResult<CommitInfo>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.revwalk.next()? {
            Ok(oid) => match self.repo.find_commit(oid) {
                Ok(commit) => Some(Ok(CommitInfo::from_git2(&commit))),
                Err(e) => Some(Err(StorageError::Git(e))),
            },
            Err(e) => Some(Err(StorageError::Git(e))),
        }
    }
}

/// get history for a commit
pub fn history(repo: &Repository, start: CommitId) -> StorageResult<HistoryIterator<'_>> {
    HistoryIterator::new(repo, start)
}

/// detect conflicting changes between two branches
///
/// both branches must share a common ancestor
/// returns paths that were modified in both branches since the merge base
pub fn detect_conflicts(
    repo: &Repository,
    ours: CommitId,
    theirs: CommitId,
) -> StorageResult<Vec<PathBuf>> {
    let base = find_merge_base(repo, ours, theirs)?.ok_or_else(|| {
        StorageError::Internal("no common ancestor found for conflict detection".to_string())
    })?;

    let our_changes = diff_commits(repo, base, ours)?;
    let their_changes = diff_commits(repo, base, theirs)?;

    // find paths modified in both
    let our_paths: std::collections::HashSet<_> = our_changes.iter().map(|c| &c.path).collect();

    let conflicts: Vec<PathBuf> = their_changes
        .iter()
        .filter(|c| our_paths.contains(&c.path))
        .map(|c| c.path.clone())
        .collect();

    Ok(conflicts)
}

/// message formatting for database operations
pub struct CommitMessage;

impl CommitMessage {
    /// format a message for an INSERT operation
    pub fn insert(table: &str, key: &str, tx_id: Option<&str>) -> String {
        match tx_id {
            Some(id) => format!("[INSERT] {}/{} tx:{}", table, key, id),
            None => format!("[INSERT] {}/{}", table, key),
        }
    }

    /// format a message for an UPDATE operation
    pub fn update(table: &str, key: &str, tx_id: Option<&str>) -> String {
        match tx_id {
            Some(id) => format!("[UPDATE] {}/{} tx:{}", table, key, id),
            None => format!("[UPDATE] {}/{}", table, key),
        }
    }

    /// format a message for a DELETE operation
    pub fn delete(table: &str, key: &str, tx_id: Option<&str>) -> String {
        match tx_id {
            Some(id) => format!("[DELETE] {}/{} tx:{}", table, key, id),
            None => format!("[DELETE] {}/{}", table, key),
        }
    }

    /// format a message for a CREATE TABLE operation
    pub fn create_table(table: &str, tx_id: Option<&str>) -> String {
        match tx_id {
            Some(id) => format!("[CREATE TABLE] {} tx:{}", table, id),
            None => format!("[CREATE TABLE] {}", table),
        }
    }

    /// format a message for a DROP TABLE operation
    pub fn drop_table(table: &str, tx_id: Option<&str>) -> String {
        match tx_id {
            Some(id) => format!("[DROP TABLE] {} tx:{}", table, id),
            None => format!("[DROP TABLE] {}", table),
        }
    }

    /// format a message for a transaction commit (merge to main)
    pub fn transaction_commit(tx_id: &str) -> String {
        format!("[COMMIT] Transaction {} merged to main", tx_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::tree::{TreeMutator, create_initial_tree};
    use crate::storage::types::TableName;
    use tempfile::TempDir;

    fn setup_repo() -> (TempDir, Repository) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        (dir, repo)
    }

    #[test]
    fn test_initial_commit() {
        let (_dir, repo) = setup_repo();
        let sig = GitSignature::gitdb();

        let commit_id = create_initial_commit(&repo, &sig).unwrap();
        let info = get_commit(&repo, commit_id).unwrap();

        assert!(info.message.contains("Initialize"));
        assert!(info.parent_ids.is_empty()); // initial commit has no parents
    }

    #[test]
    fn test_commit_builder() {
        let (_dir, repo) = setup_repo();
        let sig = GitSignature::gitdb();

        // create initial commit
        let initial = create_initial_commit(&repo, &sig).unwrap();

        // create second commit
        let tree_id = create_initial_tree(&repo).unwrap();
        let second = CommitBuilder::new(&repo)
            .tree(tree_id)
            .parent(initial)
            .message("Second commit")
            .commit()
            .unwrap();

        let info = get_commit(&repo, second).unwrap();
        assert_eq!(info.parent_ids.len(), 1);
        assert_eq!(info.parent_ids[0], initial);
        assert_eq!(info.summary(), "Second commit");
    }

    #[test]
    fn test_history_iteration() {
        let (_dir, repo) = setup_repo();
        let sig = GitSignature::gitdb();

        let c1 = create_initial_commit(&repo, &sig).unwrap();

        let tree_id = create_initial_tree(&repo).unwrap();
        let c2 = CommitBuilder::new(&repo)
            .tree(tree_id)
            .parent(c1)
            .message("Second")
            .commit()
            .unwrap();

        let c3 = CommitBuilder::new(&repo)
            .tree(tree_id)
            .parent(c2)
            .message("Third")
            .commit()
            .unwrap();

        let commits: Vec<_> = history(&repo, c3)
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(commits.len(), 3);
        assert_eq!(commits[0].id, c3);
        assert_eq!(commits[1].id, c2);
        assert_eq!(commits[2].id, c1);
    }

    #[test]
    fn test_diff_commits() {
        let (_dir, repo) = setup_repo();
        let sig = GitSignature::gitdb();

        let c1 = create_initial_commit(&repo, &sig).unwrap();

        // create a commit that adds a table with a row (Git doesn't track empty directories)
        let tree1 = get_tree_at_commit(&repo, c1).unwrap();
        let table = TableName::new("users").unwrap();
        let mut mutator = TreeMutator::from_tree(&repo, &tree1).unwrap();
        mutator.create_table(&table).unwrap();

        // Add a row to make it visible in diff (Git ignores empty dirs)
        let blob_content = b"{\"_pk\":\"row1\",\"_version\":1}";
        let blob_id = crate::storage::types::BlobId::new(repo.blob(blob_content).unwrap());
        let key = crate::storage::types::RowKey::new("row1").unwrap();
        mutator.upsert_row(&table, &key, blob_id).unwrap();

        let new_tree_id = mutator.write().unwrap();

        let c2 = CommitBuilder::new(&repo)
            .tree(new_tree_id)
            .parent(c1)
            .message("Add users table")
            .commit()
            .unwrap();

        let changes = diff_commits(&repo, c1, c2).unwrap();
        assert!(!changes.is_empty());
        assert!(
            changes
                .iter()
                .any(|c| c.path.to_string_lossy().contains("users"))
        );
    }

    #[test]
    fn test_merge_base() {
        let (_dir, repo) = setup_repo();
        let sig = GitSignature::gitdb();

        let base = create_initial_commit(&repo, &sig).unwrap();
        let tree_id = create_initial_tree(&repo).unwrap();

        // create two branches from base
        let branch_a = CommitBuilder::new(&repo)
            .tree(tree_id)
            .parent(base)
            .message("Branch A")
            .commit()
            .unwrap();

        let branch_b = CommitBuilder::new(&repo)
            .tree(tree_id)
            .parent(base)
            .message("Branch B")
            .commit()
            .unwrap();

        let merge_base = find_merge_base(&repo, branch_a, branch_b).unwrap();
        assert_eq!(merge_base, Some(base));
    }

    #[test]
    fn test_commit_messages() {
        assert_eq!(
            CommitMessage::insert("users", "123", Some("tx001")),
            "[INSERT] users/123 tx:tx001"
        );
        assert_eq!(
            CommitMessage::delete("users", "123", None),
            "[DELETE] users/123"
        );
        assert_eq!(
            CommitMessage::transaction_commit("tx001"),
            "[COMMIT] Transaction tx001 merged to main"
        );
    }
}
