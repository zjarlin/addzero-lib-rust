//!  tree operations for table management.
//!
//! in Git, a tree is a directory.  In GitDB:
//! - the root tree contains table directories and metadata
//! - each table directory contains row blobs (JSON files)
//!  - the `_schema` directory contains table schema definitions
//!
//! this module provides safe abstractions over Git's tree manipulation,
//! which is notoriously fiddly to get right.

use std::path::Path;

use az_derive_aliases::{apply, plain_debug};
use git2::{FileMode, ObjectType, Repository, Tree, TreeBuilder as Git2TreeBuilder};

use crate::storage::blob::BlobId;
use crate::storage::error::{StorageError, StorageResult};
use crate::storage::types::{RowKey, RowPath, TableName, TreeId};

/// A read only handle to a git tree at a specific commit
///
/// this provides safe, immutable access to the tree structure.
/// think of it as a snapshot - it won't change even if new commits are made.
#[apply(plain_debug)]
pub struct TreeHandle<'repo> {
    tree: Tree<'repo>,
}

impl<'repo> TreeHandle<'repo> {
    /// create a TreeHandle from a git2::Tree
    pub(crate) fn new(tree: Tree<'repo>) -> Self {
        Self { tree }
    }

    /// get the tree ID
    pub fn id(&self) -> TreeId {
        TreeId::new(self.tree.id())
    }

    /// get the underlying git2::Tree (for internal use)
    pub(crate) fn inner(&self) -> &Tree<'repo> {
        &self.tree
    }

    /// list all tables (top-level directories, excluding _schema and other metadata).
    pub fn list_tables(&self) -> Vec<TableName> {
        self.tree
            .iter()
            .filter_map(|entry| {
                // only include tree entries (directories)
                if entry.kind() != Some(ObjectType::Tree) {
                    return None;
                }

                let name = entry.name()?;

                // skip metadata directories
                if name.starts_with('_') {
                    return None;
                }

                // try to create a valid TableName
                TableName::new(name).ok()
            })
            .collect()
    }

    /// check if a table exists
    pub fn table_exists(&self, table: &TableName) -> bool {
        self.tree
            .get_name(table.as_str())
            .map(|entry| entry.kind() == Some(ObjectType::Tree))
            .unwrap_or(false)
    }

    /// get the tree for a specific table
    pub fn get_table_tree(
        &self,
        repo: &'repo Repository,
        table: &TableName,
    ) -> StorageResult<Option<TreeHandle<'repo>>> {
        match self.tree.get_name(table.as_str()) {
            Some(entry) => {
                if entry.kind() != Some(ObjectType::Tree) {
                    return Err(StorageError::UnexpectedEntryType {
                        path: table.as_str().into(),
                        expected: "tree (directory)".to_string(),
                        found: format!("{:?}", entry.kind()),
                    });
                }
                let tree = repo.find_tree(entry.id())?;
                Ok(Some(TreeHandle::new(tree)))
            }
            None => Ok(None),
        }
    }

    /// list all row keys in a table
    pub fn list_rows(&self, repo: &Repository, table: &TableName) -> StorageResult<Vec<RowKey>> {
        let table_tree = match self.get_table_tree(repo, table)? {
            Some(t) => t,
            None => return Err(StorageError::TableNotFound(table.clone())),
        };

        let keys = table_tree
            .tree
            .iter()
            .filter_map(|entry| {
                // only blobs (files)
                if entry.kind() != Some(ObjectType::Blob) {
                    return None;
                }

                let name = entry.name()?;

                // must end with .json
                let key_str = name.strip_suffix(".json")?;

                // try to create valid RowKey
                RowKey::new(key_str).ok()
            })
            .collect();

        Ok(keys)
    }

    /// get the blob ID for a specific row
    pub fn get_row_blob_id(
        &self,
        repo: &Repository,
        table: &TableName,
        key: &RowKey,
    ) -> StorageResult<Option<BlobId>> {
        let table_tree = match self.get_table_tree(repo, table)? {
            Some(t) => t,
            None => return Err(StorageError::TableNotFound(table.clone())),
        };

        let filename = format!("{}.json", key);
        let result = match table_tree.tree.get_name(&filename) {
            Some(entry) => {
                if entry.kind() != Some(ObjectType::Blob) {
                    return Err(StorageError::UnexpectedEntryType {
                        path: RowPath::new(table.clone(), key.clone()).to_path_buf(),
                        expected: "blob (file)".to_string(),
                        found: format!("{:?}", entry.kind()),
                    });
                }
                Ok(Some(BlobId::new(entry.id())))
            }
            None => Ok(None),
        };
        result
    }

    /// check if a row exists
    pub fn row_exists(
        &self,
        repo: &Repository,
        table: &TableName,
        key: &RowKey,
    ) -> StorageResult<bool> {
        Ok(self.get_row_blob_id(repo, table, key)?.is_some())
    }

    /// get entry at an arbitrary path (for internal use)
    pub fn get_entry_at_path(&self, path: &Path) -> Option<git2::TreeEntry<'_>> {
        self.tree.get_path(path).ok()
    }

    /// count total rows across all tables (for stats)
    pub fn count_all_rows(&self, repo: &Repository) -> StorageResult<usize> {
        let mut count = 0;
        for table in self.list_tables() {
            count += self.list_rows(repo, &table)?.len();
        }
        Ok(count)
    }
}

/// a mutable tree builder for making changes
///
/// this adds up changes and produces a new tree when its final
/// the original tree is not modified
///
/// # Usage Pattern
///
/// ```ignore
/// let mut builder = TreeMutator::from_tree(repo, tree)?;
/// builder.upsert_row("users", "123", blob_id)?;
/// builder.delete_row("users", "456")?;
/// let new_tree_id = builder.write()?;
/// ```
pub struct TreeMutator<'repo> {
    repo: &'repo Repository,
    /// the root tree we're modifying
    root_builder: Git2TreeBuilder<'repo>,
    /// cache of modified subtrees (table -> builder)
    /// we need to track which tables have been modified
    modified_tables: std::collections::HashMap<String, Git2TreeBuilder<'repo>>,
    /// original table tree IDs for tables we haven't modified
    original_tables: std::collections::HashMap<String, git2::Oid>,
}

impl<'repo> TreeMutator<'repo> {
    /// create a new TreeMutator from an existing tree
    pub fn from_tree(repo: &'repo Repository, tree: &TreeHandle<'_>) -> StorageResult<Self> {
        let root_builder = repo.treebuilder(Some(tree.inner()))?;

        // cache all existing table tree IDs
        let mut original_tables = std::collections::HashMap::new();
        for entry in tree.inner().iter() {
            if entry.kind() == Some(ObjectType::Tree) {
                if let Some(name) = entry.name() {
                    original_tables.insert(name.to_string(), entry.id());
                }
            }
        }

        Ok(Self {
            repo,
            root_builder,
            modified_tables: std::collections::HashMap::new(),
            original_tables,
        })
    }

    /// create a new TreeMutator for an empty tree
    pub fn empty(repo: &'repo Repository) -> StorageResult<Self> {
        let root_builder = repo.treebuilder(None)?;
        Ok(Self {
            repo,
            root_builder,
            modified_tables: std::collections::HashMap::new(),
            original_tables: std::collections::HashMap::new(),
        })
    }

    /// get or create a builder for a table's subtree
    fn get_table_builder(&mut self, table: &str) -> StorageResult<&mut Git2TreeBuilder<'repo>> {
        if !self.modified_tables.contains_key(table) {
            // first modification to this table - create builder from original or empty
            let builder = if let Some(original_id) = self.original_tables.get(table) {
                let original_tree = self.repo.find_tree(*original_id)?;
                self.repo.treebuilder(Some(&original_tree))?
            } else {
                self.repo.treebuilder(None)?
            };
            self.modified_tables.insert(table.to_string(), builder);
        }
        Ok(self.modified_tables.get_mut(table).unwrap())
    }

    /// create a new table (empty directory)
    pub fn create_table(&mut self, table: &TableName) -> StorageResult<()> {
        let table_str = table.as_str();

        // check if table already exists
        if self.modified_tables.contains_key(table_str)
            || self.original_tables.contains_key(table_str)
        {
            return Err(StorageError::TableAlreadyExists(table.clone()));
        }

        // create empty tree for the table
        let empty_builder = self.repo.treebuilder(None)?;
        let empty_tree_id = empty_builder.write()?;

        // add to root
        self.root_builder
            .insert(table_str, empty_tree_id, FileMode::Tree.into())?;

        // track as original (since we just created it as empty)
        self.original_tables
            .insert(table_str.to_string(), empty_tree_id);

        Ok(())
    }

    /// drop a table (remove directory)
    pub fn drop_table(&mut self, table: &TableName) -> StorageResult<()> {
        let table_str = table.as_str();

        // check if table exists
        if !self.modified_tables.contains_key(table_str)
            && !self.original_tables.contains_key(table_str)
        {
            return Err(StorageError::TableNotFound(table.clone()));
        }

        // remove from tracking
        self.modified_tables.remove(table_str);
        self.original_tables.remove(table_str);

        // remove from root builder
        self.root_builder.remove(table_str)?;

        Ok(())
    }

    /// insert or update a row in a table
    pub fn upsert_row(
        &mut self,
        table: &TableName,
        key: &RowKey,
        blob_id: BlobId,
    ) -> StorageResult<()> {
        let table_str = table.as_str();

        // Ensure table exists
        if !self.modified_tables.contains_key(table_str)
            && !self.original_tables.contains_key(table_str)
        {
            return Err(StorageError::TableNotFound(table.clone()));
        }

        let table_builder = self.get_table_builder(table_str)?;
        let filename = format!("{}.json", key);

        table_builder.insert(&filename, blob_id.raw(), FileMode::Blob.into())?;

        Ok(())
    }

    /// insert a row, failing if it already exists
    pub fn insert_row(
        &mut self,
        repo: &Repository,
        current_tree: &TreeHandle<'_>,
        table: &TableName,
        key: &RowKey,
        blob_id: BlobId,
    ) -> StorageResult<()> {
        // check if row already exists
        if current_tree.row_exists(repo, table, key)? {
            return Err(StorageError::RowAlreadyExists {
                table: table.clone(),
                key: key.clone(),
            });
        }

        self.upsert_row(table, key, blob_id)
    }

    /// delete a row from a table
    pub fn delete_row(&mut self, table: &TableName, key: &RowKey) -> StorageResult<()> {
        let table_str = table.as_str();

        // Ensure table exists
        if !self.modified_tables.contains_key(table_str)
            && !self.original_tables.contains_key(table_str)
        {
            return Err(StorageError::TableNotFound(table.clone()));
        }

        let table_builder = self.get_table_builder(table_str)?;
        let filename = format!("{}.json", key);

        // git2 returns error if entry doesn't exist, but we want to verify it existed
        table_builder
            .remove(&filename)
            .map_err(|_| StorageError::RowNotFound {
                table: table.clone(),
                key: key.clone(),
            })?;

        Ok(())
    }

    /// write all changes and return the new root tree ID
    ///
    /// this is where the magic happens - we rebuild the tree hierarchy
    pub fn write(mut self) -> StorageResult<TreeId> {
        // First, write all modified table trees and update root builder
        for (table_name, table_builder) in self.modified_tables {
            let table_tree_id = table_builder.write()?;
            self.root_builder
                .insert(&table_name, table_tree_id, FileMode::Tree.into())?;
        }

        // Write the root tree
        let root_id = self.root_builder.write()?;
        Ok(TreeId::new(root_id))
    }
}

/// helper function to create an initial empty tree with _schema directory
pub fn create_initial_tree(repo: &Repository) -> StorageResult<TreeId> {
    let mut builder = TreeMutator::empty(repo)?;

    // create _schema directory (we'll allow this one since we control it)
    let schema_builder = repo.treebuilder(None)?;
    let schema_tree_id = schema_builder.write()?;

    // we need to directly insert since _schema would fail TableName validation
    builder
        .root_builder
        .insert("_schema", schema_tree_id, FileMode::Tree.into())?;

    builder.write()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_repo() -> (TempDir, Repository) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        (dir, repo)
    }

    fn create_initial_commit(repo: &Repository) -> git2::Oid {
        let tree_id = create_initial_tree(repo).unwrap();
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap()
    }

    #[test]
    fn test_list_tables_empty() {
        let (_dir, repo) = setup_repo();
        let tree_id = create_initial_tree(&repo).unwrap();
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);

        let tables = handle.list_tables();
        assert!(tables.is_empty()); // _schema is excluded
    }

    #[test]
    fn test_create_table() {
        let (_dir, repo) = setup_repo();
        let tree_id = create_initial_tree(&repo).unwrap();
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);

        let mut mutator = TreeMutator::from_tree(&repo, &handle).unwrap();
        mutator
            .create_table(&TableName::new("users").unwrap())
            .unwrap();
        let new_tree_id = mutator.write().unwrap();

        let new_tree = repo.find_tree(new_tree_id.raw()).unwrap();
        let new_handle = TreeHandle::new(new_tree);

        let tables = new_handle.list_tables();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].as_str(), "users");
    }

    #[test]
    fn test_create_duplicate_table() {
        let (_dir, repo) = setup_repo();
        let tree_id = create_initial_tree(&repo).unwrap();
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);

        let mut mutator = TreeMutator::from_tree(&repo, &handle).unwrap();
        mutator
            .create_table(&TableName::new("users").unwrap())
            .unwrap();
        let new_tree_id = mutator.write().unwrap();

        let new_tree = repo.find_tree(new_tree_id.raw()).unwrap();
        let new_handle = TreeHandle::new(new_tree);

        let mut mutator2 = TreeMutator::from_tree(&repo, &new_handle).unwrap();
        let result = mutator2.create_table(&TableName::new("users").unwrap());
        assert!(matches!(result, Err(StorageError::TableAlreadyExists(_))));
    }

    #[test]
    fn test_drop_table() {
        let (_dir, repo) = setup_repo();
        let tree_id = create_initial_tree(&repo).unwrap();
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);

        // create table
        let mut mutator = TreeMutator::from_tree(&repo, &handle).unwrap();
        mutator
            .create_table(&TableName::new("users").unwrap())
            .unwrap();
        let tree_id = mutator.write().unwrap();

        // drop table
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);
        let mut mutator = TreeMutator::from_tree(&repo, &handle).unwrap();
        mutator
            .drop_table(&TableName::new("users").unwrap())
            .unwrap();
        let tree_id = mutator.write().unwrap();

        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);
        assert!(handle.list_tables().is_empty());
    }

    #[test]
    fn test_upsert_and_list_rows() {
        let (_dir, repo) = setup_repo();
        let tree_id = create_initial_tree(&repo).unwrap();
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);

        // create table
        let mut mutator = TreeMutator::from_tree(&repo, &handle).unwrap();
        let table = TableName::new("users").unwrap();
        mutator.create_table(&table).unwrap();
        let tree_id = mutator.write().unwrap();

        // add rows
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);

        // create a dummy blob for testing
        let blob_content = b"{\"_pk\":\"row1\",\"_version\":1}";
        let blob_id = BlobId::new(repo.blob(blob_content).unwrap());

        let mut mutator = TreeMutator::from_tree(&repo, &handle).unwrap();
        let key1 = RowKey::new("row1").unwrap();
        let key2 = RowKey::new("row2").unwrap();
        mutator.upsert_row(&table, &key1, blob_id).unwrap();
        mutator.upsert_row(&table, &key2, blob_id).unwrap();
        let tree_id = mutator.write().unwrap();

        // verify
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);
        let rows = handle.list_rows(&repo, &table).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|k| k.as_str() == "row1"));
        assert!(rows.iter().any(|k| k.as_str() == "row2"));
    }

    #[test]
    fn test_delete_row() {
        let (_dir, repo) = setup_repo();
        let tree_id = create_initial_tree(&repo).unwrap();
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);

        // create table with row
        let table = TableName::new("users").unwrap();
        let key = RowKey::new("row1").unwrap();
        let blob_id = BlobId::new(repo.blob(b"test").unwrap());

        let mut mutator = TreeMutator::from_tree(&repo, &handle).unwrap();
        mutator.create_table(&table).unwrap();
        let tree_id = mutator.write().unwrap();

        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);
        let mut mutator = TreeMutator::from_tree(&repo, &handle).unwrap();
        mutator.upsert_row(&table, &key, blob_id).unwrap();
        let tree_id = mutator.write().unwrap();

        // delete row
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);
        let mut mutator = TreeMutator::from_tree(&repo, &handle).unwrap();
        mutator.delete_row(&table, &key).unwrap();
        let tree_id = mutator.write().unwrap();

        // verify
        let tree = repo.find_tree(tree_id.raw()).unwrap();
        let handle = TreeHandle::new(tree);
        let rows = handle.list_rows(&repo, &table).unwrap();
        assert!(rows.is_empty());
    }
}
