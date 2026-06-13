#![cfg(not(target_arch = "wasm32"))]

use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use az_rustfs::S3StorageClient;
use sha2::{Digest, Sha256};

use crate::{
    error::{SyncError, SyncResult},
    sync_model::normalize_home_relative_path,
    sync_server::{DEFAULT_OBJECT_CHUNK_BYTES, SyncObjectManifest},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncObjectStoreConfig {
    pub chunk_size_bytes: u64,
}

impl SyncObjectStoreConfig {
    pub fn new() -> Self {
        Self {
            chunk_size_bytes: DEFAULT_OBJECT_CHUNK_BYTES,
        }
    }

    pub fn with_chunk_size_bytes(mut self, chunk_size_bytes: u64) -> Self {
        self.chunk_size_bytes = chunk_size_bytes.max(1);
        self
    }
}

impl Default for SyncObjectStoreConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncFileSystemObjectStoreConfig {
    pub root_dir: PathBuf,
    pub chunk_size_bytes: u64,
}

impl SyncFileSystemObjectStoreConfig {
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
            chunk_size_bytes: DEFAULT_OBJECT_CHUNK_BYTES,
        }
    }

    pub fn with_chunk_size_bytes(mut self, chunk_size_bytes: u64) -> Self {
        self.chunk_size_bytes = chunk_size_bytes.max(1);
        self
    }

    fn manifest_config(&self) -> SyncObjectStoreConfig {
        SyncObjectStoreConfig {
            chunk_size_bytes: self.chunk_size_bytes,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileSystemSyncObjectStore {
    config: SyncFileSystemObjectStoreConfig,
}

impl FileSystemSyncObjectStore {
    pub fn new(config: SyncFileSystemObjectStoreConfig) -> Self {
        Self { config }
    }

    pub fn put_file(
        &self,
        space_id: impl Into<String>,
        relative_path: &str,
        source_path: impl AsRef<Path>,
    ) -> SyncResult<SyncObjectManifest> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        let source_path = source_path.as_ref();
        let manifest = manifest_for_file(
            space_id,
            &relative_path,
            source_path,
            &self.config.manifest_config(),
        )?;
        let mut source = fs::File::open(source_path).map_err(|source| SyncError::Io {
            path: source_path.to_path_buf(),
            source,
        })?;
        for chunk in &manifest.chunks {
            let chunk_path = self.chunk_path(&chunk.object_key)?;
            if let Some(parent) = chunk_path.parent() {
                fs::create_dir_all(parent).map_err(|source| SyncError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            let mut target = fs::File::create(&chunk_path).map_err(|source| SyncError::Io {
                path: chunk_path.clone(),
                source,
            })?;
            copy_limited(&mut source, &mut target, chunk.size_bytes, &chunk_path)?;
        }
        Ok(manifest)
    }

    pub fn materialize_file(
        &self,
        manifest: &SyncObjectManifest,
        target_path: impl AsRef<Path>,
    ) -> SyncResult<()> {
        let target_path = target_path.as_ref();
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|source| SyncError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let mut target = fs::File::create(target_path).map_err(|source| SyncError::Io {
            path: target_path.to_path_buf(),
            source,
        })?;
        for chunk in &manifest.chunks {
            let chunk_path = self.chunk_path(&chunk.object_key)?;
            let mut source = fs::File::open(&chunk_path).map_err(|source| SyncError::Io {
                path: chunk_path.clone(),
                source,
            })?;
            std::io::copy(&mut source, &mut target).map_err(|source| SyncError::Io {
                path: target_path.to_path_buf(),
                source,
            })?;
        }
        let restored_hash = sha256_file(target_path)?;
        if restored_hash != manifest.content_hash {
            return Err(SyncError::ObjectHashMismatch {
                relative_path: manifest.relative_path.clone(),
                expected: manifest.content_hash.clone(),
                actual: restored_hash,
            });
        }
        Ok(())
    }

    pub fn chunk_path(&self, object_key: &str) -> SyncResult<PathBuf> {
        let object_key = normalize_home_relative_path(object_key)?;
        Ok(self.config.root_dir.join(object_key))
    }
}

#[derive(Clone)]
pub struct RustfsSyncObjectStore {
    client: Arc<dyn S3StorageClient>,
    bucket_name: String,
    config: SyncObjectStoreConfig,
}

impl RustfsSyncObjectStore {
    pub fn new(
        client: Arc<dyn S3StorageClient>,
        bucket_name: impl Into<String>,
        config: SyncObjectStoreConfig,
    ) -> Self {
        Self {
            client,
            bucket_name: bucket_name.into(),
            config,
        }
    }

    pub fn ensure_bucket(&self) -> SyncResult<()> {
        if self.client.bucket_exists(&self.bucket_name)? {
            Ok(())
        } else {
            self.client.create_bucket(&self.bucket_name)?;
            Ok(())
        }
    }

    pub fn put_file(
        &self,
        space_id: impl Into<String>,
        relative_path: &str,
        source_path: impl AsRef<Path>,
    ) -> SyncResult<SyncObjectManifest> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        let source_path = source_path.as_ref();
        let manifest = manifest_for_file(space_id, &relative_path, source_path, &self.config)?;
        self.ensure_bucket()?;
        let mut source = fs::File::open(source_path).map_err(|source| SyncError::Io {
            path: source_path.to_path_buf(),
            source,
        })?;
        for chunk in &manifest.chunks {
            let bytes = read_limited(&mut source, chunk.size_bytes, source_path)?;
            self.client.put_object_bytes(
                &self.bucket_name,
                &chunk.object_key,
                &bytes,
                Some("application/octet-stream"),
                &BTreeMap::from([
                    ("space-id".to_string(), manifest.space_id.clone()),
                    ("relative-path".to_string(), manifest.relative_path.clone()),
                    ("content-hash".to_string(), manifest.content_hash.clone()),
                ]),
            )?;
        }
        Ok(manifest)
    }

    pub fn materialize_file(
        &self,
        manifest: &SyncObjectManifest,
        target_path: impl AsRef<Path>,
    ) -> SyncResult<()> {
        let target_path = target_path.as_ref();
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|source| SyncError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let mut target = fs::File::create(target_path).map_err(|source| SyncError::Io {
            path: target_path.to_path_buf(),
            source,
        })?;
        for chunk in &manifest.chunks {
            let bytes = self
                .client
                .get_object(&self.bucket_name, &chunk.object_key)?;
            target.write_all(&bytes).map_err(|source| SyncError::Io {
                path: target_path.to_path_buf(),
                source,
            })?;
        }
        let restored_hash = sha256_file(target_path)?;
        if restored_hash != manifest.content_hash {
            return Err(SyncError::ObjectHashMismatch {
                relative_path: manifest.relative_path.clone(),
                expected: manifest.content_hash.clone(),
                actual: restored_hash,
            });
        }
        Ok(())
    }

    pub fn object_exists(&self, object_key: &str) -> SyncResult<bool> {
        let object_key = normalize_home_relative_path(object_key)?;
        self.client
            .object_exists(&self.bucket_name, &object_key)
            .map_err(Into::into)
    }
}

pub fn sha256_file(path: impl AsRef<Path>) -> SyncResult<String> {
    let path = path.as_ref();
    let mut file = fs::File::open(path).map_err(|source| SyncError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(|source| SyncError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn manifest_for_file(
    space_id: impl Into<String>,
    relative_path: &str,
    source_path: &Path,
    config: &SyncObjectStoreConfig,
) -> SyncResult<SyncObjectManifest> {
    let metadata = fs::metadata(source_path).map_err(|source| SyncError::Io {
        path: source_path.to_path_buf(),
        source,
    })?;
    let content_hash = sha256_file(source_path)?;
    SyncObjectManifest::plan(
        space_id,
        relative_path,
        content_hash,
        metadata.len(),
        config.chunk_size_bytes,
    )
}

fn read_limited(source: &mut fs::File, size_bytes: u64, path: &Path) -> SyncResult<Vec<u8>> {
    let capacity = usize::try_from(size_bytes).unwrap_or(usize::MAX);
    let mut bytes = Vec::with_capacity(capacity);
    let mut remaining = size_bytes;
    let mut buffer = [0_u8; 8192];
    while remaining > 0 {
        let read_limit = buffer
            .len()
            .min(usize::try_from(remaining).unwrap_or(usize::MAX));
        let read = source
            .read(&mut buffer[..read_limit])
            .map_err(|source| SyncError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);
        remaining = remaining.saturating_sub(u64::try_from(read).unwrap_or(0));
    }
    Ok(bytes)
}

fn copy_limited(
    source: &mut fs::File,
    target: &mut fs::File,
    size_bytes: u64,
    path: &Path,
) -> SyncResult<()> {
    let mut remaining = size_bytes;
    let mut buffer = [0_u8; 8192];
    while remaining > 0 {
        let read_limit = buffer
            .len()
            .min(usize::try_from(remaining).unwrap_or(usize::MAX));
        let read = source
            .read(&mut buffer[..read_limit])
            .map_err(|source| SyncError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        if read == 0 {
            break;
        }
        target
            .write_all(&buffer[..read])
            .map_err(|source| SyncError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        remaining = remaining.saturating_sub(u64::try_from(read).unwrap_or(0));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::Arc};

    use az_rustfs::InMemoryS3StorageClient;

    use super::{
        FileSystemSyncObjectStore, RustfsSyncObjectStore, SyncFileSystemObjectStoreConfig,
        SyncObjectStoreConfig, sha256_file,
    };

    #[test]
    fn filesystem_object_store_chunks_and_restores_binary_file()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let source_path = temp_dir.path().join("source.bin");
        let restored_path = temp_dir.path().join("restored.bin");
        let object_root = temp_dir.path().join("objects");
        let bytes = (0_u16..300)
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>();
        fs::write(&source_path, &bytes)?;
        let store = FileSystemSyncObjectStore::new(
            SyncFileSystemObjectStoreConfig::new(&object_root).with_chunk_size_bytes(128),
        );

        let manifest = store.put_file("main", "az-sync/blob.bin", &source_path)?;
        assert_eq!(manifest.chunks.len(), 5);
        for chunk in &manifest.chunks {
            assert!(store.chunk_path(&chunk.object_key)?.exists());
        }

        store.materialize_file(&manifest, &restored_path)?;
        assert_eq!(fs::read(restored_path)?, bytes);
        assert_eq!(manifest.content_hash, sha256_file(&source_path)?);
        Ok(())
    }

    #[test]
    fn rustfs_object_store_chunks_and_restores_binary_file()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let source_path = temp_dir.path().join("source.bin");
        let restored_path = temp_dir.path().join("restored.bin");
        let bytes = (0_u16..300)
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>();
        fs::write(&source_path, &bytes)?;
        let store = RustfsSyncObjectStore::new(
            Arc::new(InMemoryS3StorageClient::default()),
            "az-sync-objects",
            SyncObjectStoreConfig::new().with_chunk_size_bytes(128),
        );

        let manifest = store.put_file("main", "az-sync/blob.bin", &source_path)?;
        assert_eq!(manifest.chunks.len(), 5);
        for chunk in &manifest.chunks {
            assert!(store.object_exists(&chunk.object_key)?);
        }

        store.materialize_file(&manifest, &restored_path)?;
        assert_eq!(fs::read(restored_path)?, bytes);
        assert_eq!(manifest.content_hash, sha256_file(&source_path)?);
        Ok(())
    }
}
