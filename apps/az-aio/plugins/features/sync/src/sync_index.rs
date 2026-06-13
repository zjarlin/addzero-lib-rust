#![cfg(not(target_arch = "wasm32"))]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{SyncError, SyncResult},
    sync_model::{
        SyncDeviceInfo, SyncDocumentRecord, SyncFileStatus, SyncRoot, normalize_home_relative_path,
    },
};

pub const SYNC_INDEX_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncLocalIndex {
    pub schema_version: u32,
    pub device: SyncDeviceInfo,
    pub roots: Vec<SyncRoot>,
    pub files: BTreeMap<String, SyncIndexRecord>,
}

impl SyncLocalIndex {
    pub fn new(device: SyncDeviceInfo, roots: Vec<SyncRoot>) -> Self {
        Self {
            schema_version: SYNC_INDEX_SCHEMA_VERSION,
            device,
            roots,
            files: BTreeMap::new(),
        }
    }

    pub fn read_from_path(path: impl AsRef<Path>) -> SyncResult<Self> {
        let path = path.as_ref();
        let json = fs::read_to_string(path).map_err(|source| SyncError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        serde_json::from_str(&json).map_err(|source| SyncError::Json {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn write_to_path(&self, path: impl AsRef<Path>) -> SyncResult<()> {
        let path = path.as_ref();
        validate_index_path_outside_roots(path, &self.roots)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| SyncError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|source| SyncError::Json {
            path: path.to_path_buf(),
            source,
        })?;
        fs::write(path, json).map_err(|source| SyncError::Io {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn upsert_document(&mut self, document: &SyncDocumentRecord) -> SyncResult<()> {
        let metadata = SyncFileMetadata::from_path(&document.local_path)?;
        self.upsert_document_with_metadata(document, metadata)
    }

    pub fn upsert_document_with_metadata(
        &mut self,
        document: &SyncDocumentRecord,
        metadata: SyncFileMetadata,
    ) -> SyncResult<()> {
        let relative_path = normalize_home_relative_path(&document.relative_path)?;
        self.files.insert(
            relative_path.clone(),
            SyncIndexRecord {
                relative_path,
                local_path: document.local_path.clone(),
                device_name: document.device_name.clone(),
                home_dir: document.home_dir.clone(),
                file_kind: metadata.file_kind,
                local_mtime_unix_ms: metadata.local_mtime_unix_ms,
                local_size_bytes: metadata.local_size_bytes,
                content_hash: document.content_hash.clone(),
                crdt_snapshot: document.crdt_snapshot.clone(),
                crdt_version: document.crdt_version.clone(),
                sent_version: None,
                acked_version: None,
                status: document.status,
            },
        );
        Ok(())
    }

    pub fn mark_sent(&mut self, relative_path: &str, version: Vec<u8>) -> SyncResult<()> {
        let record = self.record_mut(relative_path)?;
        record.sent_version = Some(version);
        Ok(())
    }

    pub fn mark_acked(&mut self, relative_path: &str, version: Vec<u8>) -> SyncResult<()> {
        let record = self.record_mut(relative_path)?;
        record.acked_version = Some(version);
        record.status = SyncFileStatus::Synced;
        Ok(())
    }

    pub fn summary(&self) -> SyncIndexSummary {
        let path = default_local_index_path(&self.device.home_dir);
        let mut summary = SyncIndexSummary {
            path,
            schema_version: self.schema_version,
            root_count: self.roots.len(),
            file_count: self.files.len(),
            synced_count: 0,
            syncing_count: 0,
            error_count: 0,
            deleted_count: 0,
            sent_count: 0,
            acked_count: 0,
            stored_outside_sync_roots: true,
        };

        for file in self.files.values() {
            match file.status {
                SyncFileStatus::Synced | SyncFileStatus::Shared => summary.synced_count += 1,
                SyncFileStatus::Syncing => summary.syncing_count += 1,
                SyncFileStatus::Error => summary.error_count += 1,
                SyncFileStatus::Deleted => summary.deleted_count += 1,
            }
            if file.sent_version.is_some() {
                summary.sent_count += 1;
            }
            if file.acked_version.is_some() {
                summary.acked_count += 1;
            }
        }

        summary.stored_outside_sync_roots =
            validate_index_path_outside_roots(&summary.path, &self.roots).is_ok();
        summary
    }

    fn record_mut(&mut self, relative_path: &str) -> SyncResult<&mut SyncIndexRecord> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        self.files
            .get_mut(&relative_path)
            .ok_or(SyncError::MissingDocument { relative_path })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncIndexSummary {
    pub path: PathBuf,
    pub schema_version: u32,
    pub root_count: usize,
    pub file_count: usize,
    pub synced_count: usize,
    pub syncing_count: usize,
    pub error_count: usize,
    pub deleted_count: usize,
    pub sent_count: usize,
    pub acked_count: usize,
    pub stored_outside_sync_roots: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncIndexRecord {
    pub relative_path: String,
    pub local_path: PathBuf,
    pub device_name: String,
    pub home_dir: PathBuf,
    pub file_kind: SyncIndexedFileKind,
    pub local_mtime_unix_ms: Option<u64>,
    pub local_size_bytes: Option<u64>,
    pub content_hash: String,
    pub crdt_snapshot: Vec<u8>,
    pub crdt_version: Vec<u8>,
    pub sent_version: Option<Vec<u8>>,
    pub acked_version: Option<Vec<u8>>,
    pub status: SyncFileStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncIndexedFileKind {
    Text,
    Binary,
    Directory,
    Missing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyncFileMetadata {
    pub file_kind: SyncIndexedFileKind,
    pub local_mtime_unix_ms: Option<u64>,
    pub local_size_bytes: Option<u64>,
}

impl SyncFileMetadata {
    pub fn text(size_bytes: u64, local_mtime_unix_ms: u64) -> Self {
        Self {
            file_kind: SyncIndexedFileKind::Text,
            local_mtime_unix_ms: Some(local_mtime_unix_ms),
            local_size_bytes: Some(size_bytes),
        }
    }

    pub fn missing() -> Self {
        Self {
            file_kind: SyncIndexedFileKind::Missing,
            local_mtime_unix_ms: None,
            local_size_bytes: None,
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> SyncResult<Self> {
        let path = path.as_ref();
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::missing());
            }
            Err(source) => {
                return Err(SyncError::Io {
                    path: path.to_path_buf(),
                    source,
                });
            }
        };
        let file_kind = if metadata.is_dir() {
            SyncIndexedFileKind::Directory
        } else {
            SyncIndexedFileKind::Text
        };
        Ok(Self {
            file_kind,
            local_mtime_unix_ms: metadata.modified().ok().and_then(system_time_to_unix_ms),
            local_size_bytes: if metadata.is_file() {
                Some(metadata.len())
            } else {
                None
            },
        })
    }
}

pub fn default_local_index_path(home_dir: impl AsRef<Path>) -> PathBuf {
    home_dir
        .as_ref()
        .join(".config")
        .join("addzero")
        .join("sync")
        .join("index.db")
}

pub fn validate_index_path_outside_roots(path: &Path, roots: &[SyncRoot]) -> SyncResult<()> {
    for root in roots {
        if path.starts_with(&root.local_path) {
            return Err(SyncError::IndexInsideSyncRoot {
                path: path.to_path_buf(),
                root: root.local_path.clone(),
            });
        }
    }
    Ok(())
}

fn system_time_to_unix_ms(value: SystemTime) -> Option<u64> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        SyncEngine,
        sync_model::{SyncDeviceInfo, SyncRoot},
    };

    use super::{
        SyncFileMetadata, SyncIndexedFileKind, SyncLocalIndex, default_local_index_path,
        validate_index_path_outside_roots,
    };

    #[test]
    fn default_index_path_is_outside_default_sync_root() {
        let device = SyncDeviceInfo::new("mac-a", PathBuf::from("/Users/a"));
        let root = SyncRoot::default_for_device(&device);
        let index_path = default_local_index_path(&device.home_dir);

        assert_eq!(
            index_path,
            PathBuf::from("/Users/a/.config/addzero/sync/index.db")
        );
        assert!(validate_index_path_outside_roots(&index_path, &[root]).is_ok());
    }

    #[test]
    fn index_rejects_storage_inside_sync_root() {
        let device = SyncDeviceInfo::new("mac-a", PathBuf::from("/Users/a"));
        let root = SyncRoot::default_for_device(&device);
        let index_path = PathBuf::from("/Users/a/az-sync/index.db");

        let error = validate_index_path_outside_roots(&index_path, &[root]).unwrap_err();
        assert!(error.to_string().contains("inside sync root"));
    }

    #[test]
    fn index_round_trips_crdt_state_and_watermarks() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let home_dir = temp_dir.path().join("home-a");
        let mut engine = SyncEngine::with_device(SyncDeviceInfo::new("mac-a", home_dir.clone()));
        let record = engine.apply_local_text(home_dir.join("az-sync/a.txt"), "one\ntwo")?;
        let mut index = SyncLocalIndex::new(engine.device().clone(), engine.roots());

        index.upsert_document_with_metadata(&record, SyncFileMetadata::text(7, 42))?;
        index.mark_sent("az-sync/a.txt", record.crdt_version.clone())?;
        index.mark_acked("az-sync/a.txt", record.crdt_version.clone())?;
        let index_path = default_local_index_path(&home_dir);
        index.write_to_path(&index_path)?;
        let restored = SyncLocalIndex::read_from_path(&index_path)?;
        let restored_file = restored.files.get("az-sync/a.txt").expect("indexed file");

        assert_eq!(restored_file.file_kind, SyncIndexedFileKind::Text);
        assert_eq!(restored_file.local_size_bytes, Some(7));
        assert_eq!(restored_file.acked_version, Some(record.crdt_version));
        assert!(restored.summary().stored_outside_sync_roots);
        Ok(())
    }
}
