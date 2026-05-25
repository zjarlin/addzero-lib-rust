//! Sharded blob/object storage backed by multiple Git repositories.
//!
//! This module is intentionally separate from the SQL/query path. It gives
//! higher-level systems a way to store large binary objects in Git while
//! automatically expanding into new repositories once a shard reaches its
//! configured soft size limit.

use std::fs;
use std::path::{Component, Path, PathBuf};

use az_derive_aliases::{apply, plain_clone_debug, serde_eq};
use git2::{ErrorCode, IndexAddOption, Repository, Signature, StatusOptions};
use serde::{Deserialize, Serialize};

use crate::storage::{StorageError, StorageResult};

/// Default soft limit for each blob shard repository.
pub const DEFAULT_MAX_BLOB_SHARD_SIZE_BYTES: u64 = 8 * 1024 * 1024 * 1024;
/// Default shard name prefix.
pub const DEFAULT_BLOB_SHARD_PREFIX: &str = "shard";

const CONTROL_DIR: &str = "control";
const SHARDS_DIR: &str = "shards";

/// Configuration for the sharded blob store.
#[apply(serde_eq)]
pub struct BlobStoreConfig {
    /// Root directory holding shard repositories and control metadata.
    pub root: PathBuf,
    /// Soft size limit for each shard.
    #[serde(default = "default_max_blob_shard_size_bytes")]
    pub max_shard_size_bytes: u64,
    /// Prefix used for automatically created shard names.
    #[serde(default = "default_blob_shard_prefix")]
    pub shard_prefix: String,
}

impl BlobStoreConfig {
    /// Create a new config for the given root.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            max_shard_size_bytes: DEFAULT_MAX_BLOB_SHARD_SIZE_BYTES,
            shard_prefix: default_blob_shard_prefix(),
        }
    }
}

/// Persisted metadata for a blob shard.
#[apply(serde_eq)]
pub struct BlobShardInfo {
    /// Stable shard name.
    pub name: String,
    /// Absolute path of the shard repository.
    pub repo_path: PathBuf,
    /// Soft size limit for this shard.
    pub max_size_bytes: u64,
    /// Current logical payload bytes tracked in this shard.
    pub used_bytes: u64,
}

#[apply(serde_eq)]
struct BlobIndexRecord {
    shard_name: String,
}

/// Multi-repository blob store with automatic shard creation.
#[apply(plain_clone_debug)]
pub struct ShardedBlobStore {
    config: BlobStoreConfig,
}

impl ShardedBlobStore {
    /// Open or initialize the blob store root.
    pub fn open(config: BlobStoreConfig) -> StorageResult<Self> {
        let store = Self { config };
        store.ensure_layout()?;
        Ok(store)
    }

    /// Borrow the current config.
    #[must_use]
    pub fn config(&self) -> &BlobStoreConfig {
        &self.config
    }

    /// Store an object if it does not already exist.
    pub fn put(&self, key: &str, bytes: &[u8]) -> StorageResult<()> {
        validate_blob_key(key)?;
        if self.exists(key)? {
            return Ok(());
        }
        let shard = self.select_writable_shard(bytes.len() as u64)?;
        let relative = Path::new(key);
        let path = shard.repo_path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, bytes)?;
        commit_repo_path(
            &shard.repo_path,
            relative,
            &format!("[gitdb-blob] put {key}"),
        )?;
        self.update_shard_used_bytes(&shard.name, bytes.len() as i64)?;
        self.write_index_record(
            key,
            &BlobIndexRecord {
                shard_name: shard.name,
            },
        )?;
        Ok(())
    }

    /// Read an object by key.
    pub fn get(&self, key: &str) -> StorageResult<Vec<u8>> {
        validate_blob_key(key)?;
        let shard = self
            .find_shard_for_key(key)?
            .ok_or_else(|| StorageError::BlobNotFound(key.to_owned()))?;
        fs::read(shard.repo_path.join(key)).map_err(StorageError::from)
    }

    /// Delete an object when present.
    pub fn delete(&self, key: &str) -> StorageResult<()> {
        validate_blob_key(key)?;
        let Some(shard) = self.find_shard_for_key(key)? else {
            return Ok(());
        };
        let relative = Path::new(key);
        let path = shard.repo_path.join(relative);
        let size = match fs::metadata(&path) {
            Ok(metadata) => metadata.len(),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => 0,
            Err(err) => return Err(err.into()),
        };
        if path.exists() {
            fs::remove_file(&path)?;
            remove_empty_parent_dirs(&shard.repo_path, relative)?;
            commit_repo_delete(
                &shard.repo_path,
                relative,
                &format!("[gitdb-blob] delete {key}"),
            )?;
        }
        if size > 0 {
            self.update_shard_used_bytes(&shard.name, -(size as i64))?;
        }
        remove_file_if_exists(&self.index_path(key))?;
        Ok(())
    }

    /// Check whether an object exists.
    pub fn exists(&self, key: &str) -> StorageResult<bool> {
        validate_blob_key(key)?;
        Ok(self.find_shard_for_key(key)?.is_some())
    }

    /// List all known shards.
    pub fn list_shards(&self) -> StorageResult<Vec<BlobShardInfo>> {
        let mut shards: Vec<BlobShardInfo> = Vec::new();
        for path in json_files(&self.control_path().join("shards"))? {
            shards.push(read_json(&path)?);
        }
        shards.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(shards)
    }

    fn ensure_layout(&self) -> StorageResult<()> {
        fs::create_dir_all(self.control_path().join("shards"))?;
        fs::create_dir_all(self.control_path().join("index"))?;
        fs::create_dir_all(self.shards_path())?;
        Ok(())
    }

    fn select_writable_shard(&self, bytes_len: u64) -> StorageResult<BlobShardInfo> {
        let shards = self.list_shards()?;
        if let Some(shard) = shards
            .into_iter()
            .find(|shard| shard.used_bytes.saturating_add(bytes_len) <= shard.max_size_bytes)
        {
            return Ok(shard);
        }
        self.create_shard(bytes_len)
    }

    fn create_shard(&self, bytes_len: u64) -> StorageResult<BlobShardInfo> {
        let name = self.next_shard_name()?;
        let repo_path = self.shards_path().join(&name);
        fs::create_dir_all(&repo_path)?;
        open_or_init_repo(&repo_path)?;
        let shard = BlobShardInfo {
            name: name.clone(),
            repo_path,
            max_size_bytes: self.config.max_shard_size_bytes.max(bytes_len),
            used_bytes: 0,
        };
        write_json(&self.shard_info_path(&name), &shard)?;
        Ok(shard)
    }

    fn next_shard_name(&self) -> StorageResult<String> {
        let prefix = self.config.shard_prefix.trim();
        let prefix = if prefix.is_empty() {
            DEFAULT_BLOB_SHARD_PREFIX
        } else {
            prefix
        };
        for index in 1..=99_999_u32 {
            let name = format!("{prefix}-{index:04}");
            if self.shard_info_path(&name).exists() {
                continue;
            }
            return Ok(name);
        }
        Err(StorageError::Internal(
            "blob shard namespace exhausted".to_owned(),
        ))
    }

    fn find_shard_for_key(&self, key: &str) -> StorageResult<Option<BlobShardInfo>> {
        if let Some(record) = self.read_index_record(key)? {
            let shard = self.read_shard_info(&record.shard_name)?;
            if shard.repo_path.join(key).exists() {
                return Ok(Some(shard));
            }
        }
        for shard in self.list_shards()? {
            if shard.repo_path.join(key).exists() {
                self.write_index_record(
                    key,
                    &BlobIndexRecord {
                        shard_name: shard.name.clone(),
                    },
                )?;
                return Ok(Some(shard));
            }
        }
        Ok(None)
    }

    fn update_shard_used_bytes(&self, shard_name: &str, delta: i64) -> StorageResult<()> {
        let mut shard = self.read_shard_info(shard_name)?;
        shard.used_bytes = if delta.is_negative() {
            shard.used_bytes.saturating_sub(delta.unsigned_abs())
        } else {
            shard.used_bytes.saturating_add(delta as u64)
        };
        write_json(&self.shard_info_path(shard_name), &shard)
    }

    fn read_shard_info(&self, shard_name: &str) -> StorageResult<BlobShardInfo> {
        read_json(&self.shard_info_path(shard_name))
    }

    fn read_index_record(&self, key: &str) -> StorageResult<Option<BlobIndexRecord>> {
        let path = self.index_path(key);
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(read_json(&path)?))
    }

    fn write_index_record(&self, key: &str, record: &BlobIndexRecord) -> StorageResult<()> {
        let path = self.index_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_json(&path, record)
    }

    fn control_path(&self) -> PathBuf {
        self.config.root.join(CONTROL_DIR)
    }

    fn shards_path(&self) -> PathBuf {
        self.config.root.join(SHARDS_DIR)
    }

    fn shard_info_path(&self, shard_name: &str) -> PathBuf {
        self.control_path()
            .join("shards")
            .join(format!("{shard_name}.json"))
    }

    fn index_path(&self, key: &str) -> PathBuf {
        let relative = Path::new(key);
        let mut path = self.control_path().join("index").join(relative);
        if let Some(name) = relative.file_name() {
            let mut file_name = name.to_os_string();
            file_name.push(".json");
            path.set_file_name(file_name);
        }
        path
    }
}

fn open_or_init_repo(path: &Path) -> StorageResult<Repository> {
    match Repository::open(path) {
        Ok(repo) => Ok(repo),
        Err(err) if err.code() == ErrorCode::NotFound => Ok(Repository::init(path)?),
        Err(err) => Err(err.into()),
    }
}

fn commit_repo_path(repo_path: &Path, relative: &Path, message: &str) -> StorageResult<()> {
    let repo = open_or_init_repo(repo_path)?;
    let mut index = repo.index()?;
    index.add_path(relative)?;
    index.write()?;
    commit_index(&repo, &mut index, message)
}

fn commit_repo_delete(repo_path: &Path, relative: &Path, message: &str) -> StorageResult<()> {
    let repo = open_or_init_repo(repo_path)?;
    let mut index = repo.index()?;
    if index.remove_path(relative).is_err() {
        return Ok(());
    }
    index.write()?;
    commit_index(&repo, &mut index, message)
}

fn commit_index(repo: &Repository, index: &mut git2::Index, message: &str) -> StorageResult<()> {
    let mut options = StatusOptions::new();
    options.include_untracked(true).recurse_untracked_dirs(true);
    if repo.statuses(Some(&mut options))?.is_empty() {
        return Ok(());
    }

    index.add_all(["*"], IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let signature = Signature::now("gitdb", "gitdb@local")?;

    let parents = match repo.head() {
        Ok(head) => match head.target() {
            Some(target) => vec![repo.find_commit(target)?],
            None => Vec::new(),
        },
        Err(err) if matches!(err.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound) => {
            Vec::new()
        }
        Err(err) => return Err(err.into()),
    };
    let parent_refs = parents.iter().collect::<Vec<_>>();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parent_refs,
    )?;
    Ok(())
}

fn validate_blob_key(key: &str) -> StorageResult<()> {
    if key.trim().is_empty() {
        return Err(StorageError::InvalidBlobKey(key.to_owned()));
    }
    let path = Path::new(key);
    if path.is_absolute() {
        return Err(StorageError::InvalidBlobKey(key.to_owned()));
    }
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(StorageError::InvalidBlobKey(key.to_owned()));
        }
    }
    Ok(())
}

fn remove_empty_parent_dirs(repo_root: &Path, relative: &Path) -> StorageResult<()> {
    let mut current = relative.parent().map(|path| repo_root.join(path));
    while let Some(path) = current {
        if path == repo_root {
            break;
        }
        match fs::remove_dir(&path) {
            Ok(()) => {
                current = path.parent().map(Path::to_path_buf);
            }
            Err(err) if err.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                current = path.parent().map(Path::to_path_buf);
            }
            Err(err) => return Err(err.into()),
        }
    }
    Ok(())
}

fn json_files(path: &Path) -> StorageResult<Vec<PathBuf>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry.file_type()?.is_dir() {
            files.extend(json_files(&entry_path)?);
        } else if entry_path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            files.push(entry_path);
        }
    }
    files.sort();
    Ok(files)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> StorageResult<T> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> StorageResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(value)?;
    fs::write(path, raw)?;
    Ok(())
}

fn remove_file_if_exists(path: &Path) -> StorageResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}

fn default_max_blob_shard_size_bytes() -> u64 {
    DEFAULT_MAX_BLOB_SHARD_SIZE_BYTES
}

fn default_blob_shard_prefix() -> String {
    DEFAULT_BLOB_SHARD_PREFIX.to_owned()
}

#[cfg(test)]
mod tests {
    use super::{BlobStoreConfig, DEFAULT_BLOB_SHARD_PREFIX, ShardedBlobStore};

    #[test]
    fn blob_store_round_trips_objects() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let store = ShardedBlobStore::open(BlobStoreConfig::new(temp.path().join("store")))?;

        store.put("objects/sha256/ab/cdef", b"hello")?;

        assert!(store.exists("objects/sha256/ab/cdef")?);
        assert_eq!(store.get("objects/sha256/ab/cdef")?, b"hello");
        Ok(())
    }

    #[test]
    fn blob_store_auto_expands_when_shard_capacity_is_exhausted()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let mut config = BlobStoreConfig::new(temp.path().join("store"));
        config.max_shard_size_bytes = 4;
        let store = ShardedBlobStore::open(config)?;

        store.put("objects/sha256/aa/one", b"one!")?;
        store.put("objects/sha256/bb/two", b"larger than four bytes")?;

        let shards = store.list_shards()?;
        assert_eq!(shards.len(), 2);
        assert_eq!(shards[0].name, format!("{DEFAULT_BLOB_SHARD_PREFIX}-0001"));
        assert_eq!(shards[1].name, format!("{DEFAULT_BLOB_SHARD_PREFIX}-0002"));
        Ok(())
    }

    #[test]
    fn blob_store_deletes_objects_and_updates_index() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let store = ShardedBlobStore::open(BlobStoreConfig::new(temp.path().join("store")))?;
        let key = "objects/sha256/cc/three";

        store.put(key, b"bye")?;
        store.delete(key)?;

        assert!(!store.exists(key)?);
        Ok(())
    }
}
