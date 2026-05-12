use async_trait::async_trait;
use gitdb::blob_store::{BlobShardInfo, BlobStoreConfig, ShardedBlobStore};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{DriveObjectStore, DriveStoreError, DriveStoreResult};

pub use gitdb::blob_store::{DEFAULT_BLOB_SHARD_PREFIX, DEFAULT_MAX_BLOB_SHARD_SIZE_BYTES};

/// Configuration for the GitDB-backed object store.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GitDbObjectStoreConfig {
    pub root: PathBuf,
    #[serde(default = "default_gitdb_shard_limit_bytes")]
    pub max_shard_size_bytes: u64,
    #[serde(default = "default_gitdb_shard_prefix")]
    pub shard_prefix: String,
}

impl GitDbObjectStoreConfig {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            max_shard_size_bytes: DEFAULT_MAX_BLOB_SHARD_SIZE_BYTES,
            shard_prefix: DEFAULT_BLOB_SHARD_PREFIX.to_owned(),
        }
    }

    fn to_blob_store_config(&self) -> BlobStoreConfig {
        BlobStoreConfig {
            root: self.root.clone(),
            max_shard_size_bytes: self.max_shard_size_bytes,
            shard_prefix: self.shard_prefix.clone(),
        }
    }
}

/// Drive object storage backed by GitDB shard repositories.
#[derive(Clone, Debug)]
pub struct GitDbObjectStore {
    config: GitDbObjectStoreConfig,
    store: ShardedBlobStore,
}

impl GitDbObjectStore {
    pub fn open(config: GitDbObjectStoreConfig) -> DriveStoreResult<Self> {
        let store =
            ShardedBlobStore::open(config.to_blob_store_config()).map_err(map_gitdb_error)?;
        Ok(Self { config, store })
    }

    #[must_use]
    pub fn config(&self) -> &GitDbObjectStoreConfig {
        &self.config
    }

    pub fn list_shards(&self) -> DriveStoreResult<Vec<BlobShardInfo>> {
        self.store.list_shards().map_err(map_gitdb_error)
    }

    pub fn backend_status(&self) -> DriveStoreResult<serde_json::Value> {
        Ok(serde_json::json!({
            "backend": "gitdb",
            "root": self.config.root.clone(),
            "max_shard_size_bytes": self.config.max_shard_size_bytes,
            "shard_prefix": self.config.shard_prefix.clone(),
            "shards": self.list_shards()?,
        }))
    }
}

#[async_trait]
impl DriveObjectStore for GitDbObjectStore {
    async fn put_object(&self, object_key: &str, bytes: &[u8]) -> DriveStoreResult<()> {
        self.store.put(object_key, bytes).map_err(map_gitdb_error)
    }

    async fn get_object(&self, object_key: &str) -> DriveStoreResult<Vec<u8>> {
        self.store
            .get(object_key)
            .map_err(|err| map_gitdb_object_error(err, object_key))
    }

    async fn delete_object(&self, object_key: &str) -> DriveStoreResult<()> {
        self.store
            .delete(object_key)
            .map_err(|err| map_gitdb_object_error(err, object_key))
    }

    async fn object_exists(&self, object_key: &str) -> DriveStoreResult<bool> {
        self.store
            .exists(object_key)
            .map_err(|err| map_gitdb_object_error(err, object_key))
    }
}

fn default_gitdb_shard_limit_bytes() -> u64 {
    DEFAULT_MAX_BLOB_SHARD_SIZE_BYTES
}

fn default_gitdb_shard_prefix() -> String {
    DEFAULT_BLOB_SHARD_PREFIX.to_owned()
}

fn map_gitdb_error(err: gitdb::storage::StorageError) -> DriveStoreError {
    DriveStoreError::ObjectStorage(err.to_string())
}

fn map_gitdb_object_error(err: gitdb::storage::StorageError, object_key: &str) -> DriveStoreError {
    match err {
        gitdb::storage::StorageError::BlobNotFound(_) => {
            DriveStoreError::ObjectNotFound(object_key.to_owned())
        }
        other => map_gitdb_error(other),
    }
}

#[cfg(test)]
mod tests {
    use super::{GitDbObjectStore, GitDbObjectStoreConfig};
    use crate::DriveObjectStore;

    #[tokio::test]
    async fn gitdb_object_store_round_trips_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let store = GitDbObjectStore::open(GitDbObjectStoreConfig::new(
            temp.path().join("gitdb-objects"),
        ))?;

        store.put_object("objects/sha256/ab/demo", b"hello").await?;

        assert!(store.object_exists("objects/sha256/ab/demo").await?);
        assert_eq!(store.get_object("objects/sha256/ab/demo").await?, b"hello");
        Ok(())
    }
}
