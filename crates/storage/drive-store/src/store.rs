//! 独立网盘的元数据与对象存储抽象。
//!
//! PostgreSQL 是正式的元数据存储，而内存存储仅用于测试和本地冒烟运行。
//! 对象字节按内容哈希进行存储。

use anyhow::{anyhow, bail};
use async_trait::async_trait;
use az_drive_core::model::{EntryKey, RelativePath, RootAlias};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;

pub use crate::git_pool::{
    DEFAULT_AUTO_GIT_POOL_PREFIX, DEFAULT_GIT_POOL_LIMIT_BYTES, GitPoolConfig, GitPoolDriveStore,
    GitPoolMountConfig, GitPoolRepoConfig,
};
pub use crate::gitdb_object_store::{
    DEFAULT_BLOB_SHARD_PREFIX, DEFAULT_MAX_BLOB_SHARD_SIZE_BYTES, GitDbObjectStore,
    GitDbObjectStoreConfig,
};

/// File-system entry kind tracked by drive metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DriveEntryKind {
    /// Regular file.
    File,
    /// Directory marker.
    Directory,
}

/// Metadata record for a remote drive entry.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DriveEntry {
    /// Stable metadata id.
    pub id: Uuid,
    /// Remote identity.
    pub key: EntryKey,
    /// Entry kind.
    pub kind: DriveEntryKind,
    /// Latest version number.
    pub latest_version: u64,
    /// Latest content hash for files.
    pub latest_hash: Option<String>,
    /// Tombstone flag.
    pub deleted: bool,
    /// Last metadata update time.
    pub updated_at: DateTime<Utc>,
}

/// Version record for content-addressed bytes.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DriveVersion {
    /// Version row id.
    pub id: Uuid,
    /// Entry id.
    pub entry_id: Uuid,
    /// Monotonic version.
    pub version: u64,
    /// Content hash.
    pub content_hash: String,
    /// Object-store key.
    pub object_key: String,
    /// Object size.
    pub size_bytes: u64,
    /// Device that produced the version.
    pub device_id: String,
    /// Source modification time.
    pub modified_at: DateTime<Utc>,
}

/// Active lock record.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DriveLock {
    /// Entry id.
    pub entry_id: Uuid,
    /// Lock owner device id.
    pub owner_device_id: String,
    /// Human-readable owner name.
    pub owner_name: String,
    /// Opaque lock token.
    pub token: String,
    /// Expiry timestamp.
    pub expires_at: DateTime<Utc>,
}

/// Conflict record persisted for status and diagnostics.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DriveConflict {
    /// Conflict id.
    pub id: Uuid,
    /// Entry id.
    pub entry_id: Uuid,
    /// Local base version when the conflict was detected.
    pub base_version: Option<u64>,
    /// Local content hash.
    pub local_hash: String,
    /// Remote content hash.
    pub remote_hash: String,
    /// Device that detected the conflict.
    pub device_id: String,
    /// Local conflict copy path.
    pub conflict_path: String,
    /// Whether the conflict has been marked resolved.
    pub resolved: bool,
    /// Conflict creation time.
    pub created_at: DateTime<Utc>,
}

/// Durable sync task kind used for queue diagnostics and retry.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DriveSyncTaskKind {
    /// Upload local bytes as a new remote version.
    Upload,
    /// Download remote bytes to a tracked local path.
    Download,
    /// Materialize a remote-only entry on this device.
    Materialize,
    /// Conflict processing task.
    Conflict,
}

impl DriveSyncTaskKind {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// Durable sync task status.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DriveSyncTaskStatus {
    /// Task has been discovered but not completed.
    Pending,
    /// Task is currently being processed by a sync cycle.
    Running,
    /// Task completed successfully.
    Done,
    /// Task failed and can be retried.
    Failed,
}

impl DriveSyncTaskStatus {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// Queue item persisted for sync diagnostics and retry intent.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DriveSyncQueueItem {
    /// Queue item id.
    pub id: Uuid,
    /// Task kind.
    pub kind: DriveSyncTaskKind,
    /// Current task status.
    pub status: DriveSyncTaskStatus,
    /// Owner Drive namespace.
    pub space_id: String,
    /// Cross-device root alias.
    pub root_alias: RootAlias,
    /// Path relative to the logical root.
    pub relative_path: RelativePath,
    /// Device-local path associated with the task when known.
    pub local_path: Option<String>,
    /// Remote path for display and diagnostics.
    pub remote_path: String,
    /// Number of attempts.
    pub attempts: u32,
    /// Last error message for failed tasks.
    pub last_error: Option<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Remote path suspended from automatic sync until its conflict is resolved.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DriveSuspendedPath {
    /// Suspension id.
    pub id: Uuid,
    /// Suspended entry id.
    pub entry_id: Uuid,
    /// Owner Drive namespace.
    pub space_id: String,
    /// Cross-device root alias.
    pub root_alias: RootAlias,
    /// Path relative to the logical root.
    pub relative_path: RelativePath,
    /// Stable reason such as `conflict`.
    pub reason: String,
    /// Conflict id that caused the suspension when available.
    pub conflict_id: Option<Uuid>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Shared metadata rule that excludes a remote path from automatic hosting.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DriveIgnoredPath {
    /// Stable ignore rule id.
    pub id: Uuid,
    /// Owner Drive namespace.
    pub space_id: String,
    /// Cross-device root alias.
    pub root_alias: RootAlias,
    /// Path relative to the logical root.
    pub relative_path: RelativePath,
    /// Device that created or last updated the ignore rule.
    pub source_device_id: String,
    /// Rule creation time.
    pub created_at: DateTime<Utc>,
    /// Rule update time.
    pub updated_at: DateTime<Utc>,
}

/// Metadata store contract shared by the daemon, server, and future AIO integration.
#[async_trait]
pub trait DriveMetadataStore: Send + Sync {
    /// Creates or returns an entry for a remote key.
    async fn upsert_entry(
        &self,
        key: &EntryKey,
        kind: DriveEntryKind,
    ) -> anyhow::Result<DriveEntry>;

    /// Looks up an entry by key.
    async fn get_entry(&self, key: &EntryKey) -> anyhow::Result<Option<DriveEntry>>;

    /// Looks up an entry by id.
    async fn get_entry_by_id(&self, id: Uuid) -> anyhow::Result<Option<DriveEntry>>;

    /// Lists entries under a prefix.
    async fn list_entries(
        &self,
        space_id: &str,
        root_alias: &RootAlias,
        prefix: &RelativePath,
    ) -> anyhow::Result<Vec<DriveEntry>>;

    /// Lists all non-deleted entries in an owner Drive namespace.
    async fn list_entries_by_space(&self, space_id: &str) -> anyhow::Result<Vec<DriveEntry>>;

    /// Migrates legacy namespace rows to the owner Drive namespace.
    async fn migrate_owner_drive_namespace(
        &self,
        from_owner_drive_id: &str,
        to_owner_drive_id: &str,
    ) -> anyhow::Result<u64>;

    /// Creates or refreshes an ignore rule for a remote path.
    async fn upsert_ignored_path(
        &self,
        key: &EntryKey,
        source_device_id: &str,
    ) -> anyhow::Result<DriveIgnoredPath>;

    /// Deletes an exact ignore rule for a remote path.
    async fn delete_ignored_path(&self, key: &EntryKey) -> anyhow::Result<()>;

    /// Lists ignore rules, optionally scoped to a root and prefix.
    async fn list_ignored_paths(
        &self,
        space_id: &str,
        root_alias: Option<&RootAlias>,
        prefix: Option<&RelativePath>,
    ) -> anyhow::Result<Vec<DriveIgnoredPath>>;

    /// Deletes an entry tombstone.
    async fn delete_entry(&self, key: &EntryKey) -> anyhow::Result<()>;

    /// Inserts a new version and updates the entry latest pointer.
    async fn insert_version(&self, version: DriveVersion) -> anyhow::Result<DriveVersion>;

    /// Returns the latest version for an entry.
    async fn latest_version(&self, entry_id: Uuid) -> anyhow::Result<Option<DriveVersion>>;

    /// Records a conflict.
    async fn record_conflict(&self, conflict: DriveConflict) -> anyhow::Result<DriveConflict>;

    /// Lists unresolved conflicts.
    async fn list_conflicts(&self, resolved: Option<bool>) -> anyhow::Result<Vec<DriveConflict>>;

    /// Marks a conflict resolved.
    async fn resolve_conflict(&self, _conflict_id: Uuid) -> anyhow::Result<Option<DriveConflict>> {
        Ok(None)
    }

    /// Enqueues a sync task.
    async fn enqueue_sync_task(
        &self,
        item: DriveSyncQueueItem,
    ) -> anyhow::Result<DriveSyncQueueItem> {
        Ok(item)
    }

    /// Updates a sync task status.
    async fn update_sync_task(
        &self,
        _id: Uuid,
        _status: DriveSyncTaskStatus,
        _last_error: Option<&str>,
    ) -> anyhow::Result<Option<DriveSyncQueueItem>> {
        Ok(None)
    }

    /// Lists queued sync tasks.
    async fn list_sync_queue(
        &self,
        _status: Option<DriveSyncTaskStatus>,
    ) -> anyhow::Result<Vec<DriveSyncQueueItem>> {
        Ok(Vec::new())
    }

    /// Moves failed queue items back to pending.
    async fn retry_failed_sync_tasks(&self) -> anyhow::Result<u64> {
        Ok(0)
    }

    /// Creates or refreshes a suspended path.
    async fn upsert_suspended_path(
        &self,
        suspension: DriveSuspendedPath,
    ) -> anyhow::Result<DriveSuspendedPath> {
        Ok(suspension)
    }

    /// Returns a suspension by entry id.
    async fn get_suspended_path(
        &self,
        _entry_id: Uuid,
    ) -> anyhow::Result<Option<DriveSuspendedPath>> {
        Ok(None)
    }

    /// Lists suspended paths.
    async fn list_suspended_paths(&self) -> anyhow::Result<Vec<DriveSuspendedPath>> {
        Ok(Vec::new())
    }

    /// Removes a suspended path by entry id.
    async fn delete_suspended_path(&self, _entry_id: Uuid) -> anyhow::Result<bool> {
        Ok(false)
    }

    /// Acquires or replaces an expired lock.
    async fn acquire_lock(&self, lock: DriveLock) -> anyhow::Result<DriveLock>;

    /// Releases a lock by token.
    async fn release_lock(&self, entry_id: Uuid, token: &str) -> anyhow::Result<bool>;

    /// Returns the active lock if present.
    async fn get_lock(&self, entry_id: Uuid) -> anyhow::Result<Option<DriveLock>>;
}

/// Object byte store contract.
#[async_trait]
pub trait DriveObjectStore: Send + Sync {
    /// Stores object bytes under a content-addressed key.
    async fn put_object(&self, object_key: &str, bytes: &[u8]) -> anyhow::Result<()>;

    /// Loads object bytes.
    async fn get_object(&self, object_key: &str) -> anyhow::Result<Vec<u8>>;

    /// Deletes an object by key.
    async fn delete_object(&self, object_key: &str) -> anyhow::Result<()>;

    /// Returns true when the object exists.
    async fn object_exists(&self, object_key: &str) -> anyhow::Result<bool>;
}

/// Optional synchronization coordinator for stores that need remote VCS pulls
/// and pushes around a logical drive operation.
#[async_trait]
pub trait DriveSyncCoordinator: Send + Sync {
    /// Pulls remote state before local reads/writes.
    async fn prepare_sync(&self) -> anyhow::Result<()>;

    /// Commits and pushes local state after successful writes.
    async fn flush_sync(&self) -> anyhow::Result<()>;
}

/// No-op coordinator used by database/object-store backends.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoopDriveSyncCoordinator;

#[async_trait]
impl DriveSyncCoordinator for NoopDriveSyncCoordinator {
    async fn prepare_sync(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn flush_sync(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Recoverable in-memory implementation for tests and local-only smoke runs.
#[derive(Clone, Default)]
pub struct InMemoryDriveMetadataStore {
    state: Arc<Mutex<InMemoryState>>,
}

#[derive(Default)]
struct InMemoryState {
    entries_by_key: BTreeMap<String, DriveEntry>,
    entries_by_id: HashMap<Uuid, String>,
    ignored_by_key: BTreeMap<String, DriveIgnoredPath>,
    versions: BTreeMap<Uuid, Vec<DriveVersion>>,
    locks: HashMap<Uuid, DriveLock>,
    conflicts: Vec<DriveConflict>,
    sync_queue: BTreeMap<Uuid, DriveSyncQueueItem>,
    suspended_by_entry: BTreeMap<Uuid, DriveSuspendedPath>,
}

impl InMemoryDriveMetadataStore {
    /// Creates an empty in-memory metadata store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn state(&self) -> anyhow::Result<MutexGuard<'_, InMemoryState>> {
        self.state
            .lock()
            .map_err(|_| anyhow!("in-memory drive store lock failed"))
    }
}

#[async_trait]
impl DriveMetadataStore for InMemoryDriveMetadataStore {
    async fn upsert_entry(
        &self,
        key: &EntryKey,
        kind: DriveEntryKind,
    ) -> anyhow::Result<DriveEntry> {
        let mut state = self.state()?;
        let key_text = key.remote_path();
        if let Some(entry) = state.entries_by_key.get(&key_text) {
            return Ok(entry.clone());
        }

        let entry = DriveEntry {
            id: Uuid::new_v4(),
            key: key.clone(),
            kind,
            latest_version: 0,
            latest_hash: None,
            deleted: false,
            updated_at: Utc::now(),
        };
        state.entries_by_id.insert(entry.id, key_text.clone());
        state.entries_by_key.insert(key_text, entry.clone());
        Ok(entry)
    }

    async fn get_entry(&self, key: &EntryKey) -> anyhow::Result<Option<DriveEntry>> {
        Ok(self
            .state()?
            .entries_by_key
            .get(&key.remote_path())
            .cloned())
    }

    async fn get_entry_by_id(&self, id: Uuid) -> anyhow::Result<Option<DriveEntry>> {
        let state = self.state()?;
        let Some(key) = state.entries_by_id.get(&id) else {
            return Ok(None);
        };
        Ok(state.entries_by_key.get(key).cloned())
    }

    async fn list_entries(
        &self,
        space_id: &str,
        root_alias: &RootAlias,
        prefix: &RelativePath,
    ) -> anyhow::Result<Vec<DriveEntry>> {
        let prefix_text = prefix.as_str();
        let mut entries = self
            .state()?
            .entries_by_key
            .values()
            .filter(|entry| {
                entry.key.space_id == space_id
                    && &entry.key.root_alias == root_alias
                    && !entry.deleted
                    && (prefix_text.is_empty()
                        || entry.key.relative_path.as_str() == prefix_text
                        || entry
                            .key
                            .relative_path
                            .as_str()
                            .starts_with(&format!("{prefix_text}/")))
            })
            .cloned()
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.key.relative_path.cmp(&right.key.relative_path));
        Ok(entries)
    }

    async fn list_entries_by_space(&self, space_id: &str) -> anyhow::Result<Vec<DriveEntry>> {
        let mut entries = self
            .state()?
            .entries_by_key
            .values()
            .filter(|entry| entry.key.space_id == space_id && !entry.deleted)
            .cloned()
            .collect::<Vec<_>>();
        entries.sort_by(entry_order);
        Ok(entries)
    }

    async fn migrate_owner_drive_namespace(
        &self,
        from_owner_drive_id: &str,
        to_owner_drive_id: &str,
    ) -> anyhow::Result<u64> {
        if from_owner_drive_id == to_owner_drive_id {
            return Ok(0);
        }
        let mut state = self.state()?;
        let old_keys = state
            .entries_by_key
            .iter()
            .filter_map(|(key_text, entry)| {
                (entry.key.space_id == from_owner_drive_id).then_some(key_text.clone())
            })
            .collect::<Vec<_>>();
        let mut migrated = 0;
        for old_key in old_keys {
            let Some(mut entry) = state.entries_by_key.remove(&old_key) else {
                continue;
            };
            let new_key = EntryKey::new(
                to_owner_drive_id,
                entry.key.root_alias.clone(),
                entry.key.relative_path.clone(),
            );
            let new_key_text = new_key.remote_path();
            if state.entries_by_key.contains_key(&new_key_text) {
                entry.deleted = true;
                entry.updated_at = Utc::now();
                state.entries_by_key.insert(old_key, entry);
                continue;
            }
            entry.key = new_key;
            entry.updated_at = Utc::now();
            state.entries_by_id.insert(entry.id, new_key_text.clone());
            state.entries_by_key.insert(new_key_text, entry);
            migrated += 1;
        }

        let old_ignored_keys = state
            .ignored_by_key
            .iter()
            .filter_map(|(key_text, ignored)| {
                (ignored.space_id == from_owner_drive_id).then_some(key_text.clone())
            })
            .collect::<Vec<_>>();
        for old_key in old_ignored_keys {
            let Some(mut ignored) = state.ignored_by_key.remove(&old_key) else {
                continue;
            };
            let new_key = EntryKey::new(
                to_owner_drive_id,
                ignored.root_alias.clone(),
                ignored.relative_path.clone(),
            );
            let new_key_text = new_key.remote_path();
            if state.ignored_by_key.contains_key(&new_key_text) {
                continue;
            }
            ignored.space_id = to_owner_drive_id.to_owned();
            ignored.updated_at = Utc::now();
            state.ignored_by_key.insert(new_key_text, ignored);
            migrated += 1;
        }
        Ok(migrated)
    }

    async fn upsert_ignored_path(
        &self,
        key: &EntryKey,
        source_device_id: &str,
    ) -> anyhow::Result<DriveIgnoredPath> {
        let mut state = self.state()?;
        let key_text = key.remote_path();
        if let Some(existing) = state.ignored_by_key.get_mut(&key_text) {
            existing.source_device_id = source_device_id.to_owned();
            existing.updated_at = Utc::now();
            return Ok(existing.clone());
        }
        let now = Utc::now();
        let ignored = DriveIgnoredPath {
            id: Uuid::new_v4(),
            space_id: key.space_id.clone(),
            root_alias: key.root_alias.clone(),
            relative_path: key.relative_path.clone(),
            source_device_id: source_device_id.to_owned(),
            created_at: now,
            updated_at: now,
        };
        state.ignored_by_key.insert(key_text, ignored.clone());
        Ok(ignored)
    }

    async fn delete_ignored_path(&self, key: &EntryKey) -> anyhow::Result<()> {
        self.state()?.ignored_by_key.remove(&key.remote_path());
        Ok(())
    }

    async fn list_ignored_paths(
        &self,
        space_id: &str,
        root_alias: Option<&RootAlias>,
        prefix: Option<&RelativePath>,
    ) -> anyhow::Result<Vec<DriveIgnoredPath>> {
        let mut ignored = self
            .state()?
            .ignored_by_key
            .values()
            .filter(|ignored| {
                ignored.space_id == space_id
                    && root_alias.is_none_or(|alias| &ignored.root_alias == alias)
                    && prefix.is_none_or(|prefix| {
                        prefix.is_root()
                            || ignored.relative_path == *prefix
                            || ignored
                                .relative_path
                                .as_str()
                                .starts_with(&format!("{}/", prefix.as_str()))
                    })
            })
            .cloned()
            .collect::<Vec<_>>();
        ignored.sort_by(ignored_order);
        Ok(ignored)
    }

    async fn delete_entry(&self, key: &EntryKey) -> anyhow::Result<()> {
        let mut state = self.state()?;
        if let Some(entry) = state.entries_by_key.get_mut(&key.remote_path()) {
            entry.deleted = true;
            entry.updated_at = Utc::now();
        }
        Ok(())
    }

    async fn insert_version(&self, version: DriveVersion) -> anyhow::Result<DriveVersion> {
        let mut state = self.state()?;
        let key = state
            .entries_by_id
            .get(&version.entry_id)
            .cloned()
            .ok_or_else(|| anyhow!("drive entry was not found: {}", version.entry_id))?;
        let entry = state
            .entries_by_key
            .get_mut(&key)
            .ok_or_else(|| anyhow!("drive entry was not found: {key}"))?;
        entry.latest_version = version.version;
        entry.latest_hash = Some(version.content_hash.clone());
        entry.deleted = false;
        entry.updated_at = Utc::now();
        state
            .versions
            .entry(version.entry_id)
            .or_default()
            .push(version.clone());
        Ok(version)
    }

    async fn latest_version(&self, entry_id: Uuid) -> anyhow::Result<Option<DriveVersion>> {
        Ok(self
            .state()?
            .versions
            .get(&entry_id)
            .and_then(|versions| versions.iter().max_by_key(|version| version.version))
            .cloned())
    }

    async fn record_conflict(&self, conflict: DriveConflict) -> anyhow::Result<DriveConflict> {
        self.state()?.conflicts.push(conflict.clone());
        Ok(conflict)
    }

    async fn list_conflicts(&self, resolved: Option<bool>) -> anyhow::Result<Vec<DriveConflict>> {
        Ok(self
            .state()?
            .conflicts
            .iter()
            .filter(|conflict| resolved.is_none_or(|value| conflict.resolved == value))
            .cloned()
            .collect())
    }

    async fn resolve_conflict(&self, conflict_id: Uuid) -> anyhow::Result<Option<DriveConflict>> {
        let mut state = self.state()?;
        let Some(conflict) = state
            .conflicts
            .iter_mut()
            .find(|conflict| conflict.id == conflict_id)
        else {
            return Ok(None);
        };
        conflict.resolved = true;
        Ok(Some(conflict.clone()))
    }

    async fn enqueue_sync_task(
        &self,
        item: DriveSyncQueueItem,
    ) -> anyhow::Result<DriveSyncQueueItem> {
        self.state()?.sync_queue.insert(item.id, item.clone());
        Ok(item)
    }

    async fn update_sync_task(
        &self,
        id: Uuid,
        status: DriveSyncTaskStatus,
        last_error: Option<&str>,
    ) -> anyhow::Result<Option<DriveSyncQueueItem>> {
        let mut state = self.state()?;
        let Some(item) = state.sync_queue.get_mut(&id) else {
            return Ok(None);
        };
        item.status = status;
        item.updated_at = Utc::now();
        item.last_error = last_error.map(str::to_owned);
        if matches!(status, DriveSyncTaskStatus::Running) {
            item.attempts = item.attempts.saturating_add(1);
        }
        Ok(Some(item.clone()))
    }

    async fn list_sync_queue(
        &self,
        status: Option<DriveSyncTaskStatus>,
    ) -> anyhow::Result<Vec<DriveSyncQueueItem>> {
        let mut items = self
            .state()?
            .sync_queue
            .values()
            .filter(|item| status.is_none_or(|status| item.status == status))
            .cloned()
            .collect::<Vec<_>>();
        items.sort_by_key(|item| item.created_at);
        Ok(items)
    }

    async fn retry_failed_sync_tasks(&self) -> anyhow::Result<u64> {
        let mut state = self.state()?;
        let mut count = 0;
        for item in state.sync_queue.values_mut() {
            if item.status == DriveSyncTaskStatus::Failed {
                item.status = DriveSyncTaskStatus::Pending;
                item.last_error = None;
                item.updated_at = Utc::now();
                count += 1;
            }
        }
        Ok(count)
    }

    async fn upsert_suspended_path(
        &self,
        suspension: DriveSuspendedPath,
    ) -> anyhow::Result<DriveSuspendedPath> {
        self.state()?
            .suspended_by_entry
            .insert(suspension.entry_id, suspension.clone());
        Ok(suspension)
    }

    async fn get_suspended_path(
        &self,
        entry_id: Uuid,
    ) -> anyhow::Result<Option<DriveSuspendedPath>> {
        Ok(self.state()?.suspended_by_entry.get(&entry_id).cloned())
    }

    async fn list_suspended_paths(&self) -> anyhow::Result<Vec<DriveSuspendedPath>> {
        let mut items = self
            .state()?
            .suspended_by_entry
            .values()
            .cloned()
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            left.space_id
                .cmp(&right.space_id)
                .then_with(|| left.root_alias.cmp(&right.root_alias))
                .then_with(|| left.relative_path.cmp(&right.relative_path))
        });
        Ok(items)
    }

    async fn delete_suspended_path(&self, entry_id: Uuid) -> anyhow::Result<bool> {
        Ok(self.state()?.suspended_by_entry.remove(&entry_id).is_some())
    }

    async fn acquire_lock(&self, lock: DriveLock) -> anyhow::Result<DriveLock> {
        let mut state = self.state()?;
        if let Some(existing) = state.locks.get(&lock.entry_id)
            && existing.expires_at > Utc::now()
            && existing.owner_device_id != lock.owner_device_id
        {
            bail!("drive entry is locked by `{}`", existing.owner_device_id);
        }
        state.locks.insert(lock.entry_id, lock.clone());
        Ok(lock)
    }

    async fn release_lock(&self, entry_id: Uuid, token: &str) -> anyhow::Result<bool> {
        let mut state = self.state()?;
        let should_remove = state
            .locks
            .get(&entry_id)
            .is_some_and(|lock| lock.token == token);
        if should_remove {
            state.locks.remove(&entry_id);
        }
        Ok(should_remove)
    }

    async fn get_lock(&self, entry_id: Uuid) -> anyhow::Result<Option<DriveLock>> {
        Ok(self.state()?.locks.get(&entry_id).cloned())
    }
}

/// In-memory object byte store.
#[derive(Clone, Default)]
pub struct InMemoryDriveObjectStore {
    objects: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl InMemoryDriveObjectStore {
    /// Creates an empty in-memory object store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn objects(&self) -> anyhow::Result<MutexGuard<'_, HashMap<String, Vec<u8>>>> {
        self.objects
            .lock()
            .map_err(|_| anyhow!("in-memory drive store lock failed"))
    }
}

#[async_trait]
impl DriveObjectStore for InMemoryDriveObjectStore {
    async fn put_object(&self, object_key: &str, bytes: &[u8]) -> anyhow::Result<()> {
        self.objects()?
            .insert(object_key.to_owned(), bytes.to_vec());
        Ok(())
    }

    async fn get_object(&self, object_key: &str) -> anyhow::Result<Vec<u8>> {
        self.objects()?
            .get(object_key)
            .cloned()
            .ok_or_else(|| anyhow!("drive object was not found: {object_key}"))
    }

    async fn delete_object(&self, object_key: &str) -> anyhow::Result<()> {
        self.objects()?.remove(object_key);
        Ok(())
    }

    async fn object_exists(&self, object_key: &str) -> anyhow::Result<bool> {
        Ok(self.objects()?.contains_key(object_key))
    }
}

fn entry_order(left: &DriveEntry, right: &DriveEntry) -> std::cmp::Ordering {
    left.key
        .root_alias
        .cmp(&right.key.root_alias)
        .then_with(|| left.key.relative_path.cmp(&right.key.relative_path))
}

fn ignored_order(left: &DriveIgnoredPath, right: &DriveIgnoredPath) -> std::cmp::Ordering {
    left.root_alias
        .cmp(&right.root_alias)
        .then_with(|| left.relative_path.cmp(&right.relative_path))
}
