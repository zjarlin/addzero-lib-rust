#![forbid(unsafe_code)]

//! 独立网盘的元数据与对象存储抽象。
//!
//! PostgreSQL 是正式的元数据存储，而内存存储仅用于测试和本地冒烟运行。
//! 对象字节按内容哈希进行存储。

use async_trait::async_trait;
use az_derive_aliases::{
    apply, error, plain_default_clone, plain_default_copy_eq, serde_code, serde_eq, serde_eq_copy,
};
use az_drive_core::{EntryKey, RelativePath, RootAlias};
use az_rustfs::StorageError;
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;

pub mod git_pool;
pub mod gitdb_object_store;

pub use git_pool::{
    DEFAULT_AUTO_GIT_POOL_PREFIX, DEFAULT_GIT_POOL_LIMIT_BYTES, GitPoolConfig, GitPoolDriveStore,
    GitPoolMountConfig, GitPoolRepoConfig,
};
pub use gitdb_object_store::{
    DEFAULT_BLOB_SHARD_PREFIX, DEFAULT_MAX_BLOB_SHARD_SIZE_BYTES, GitDbObjectStore,
    GitDbObjectStoreConfig,
};

/// Result alias for drive store operations.
pub type DriveStoreResult<T> = Result<T, DriveStoreError>;

/// Storage-layer errors with preserved root cause context.
#[apply(error)]
pub enum DriveStoreError {
    /// A requested entry does not exist.
    #[error("drive entry was not found: {0}")]
    EntryNotFound(String),
    /// A requested object does not exist.
    #[error("drive object was not found: {0}")]
    ObjectNotFound(String),
    /// The requested operation would violate an active lock.
    #[error("drive entry is locked by `{owner_device_id}`")]
    LockedByOther {
        /// Lock owner device id.
        owner_device_id: String,
    },
    /// PostgreSQL operation failed.
    #[error("postgres drive store error: {0}")]
    Sqlx(#[from] sqlx::Error),
    /// RustFS/S3-compatible object storage operation failed.
    #[error("object storage error: {0}")]
    ObjectStorage(#[from] StorageError),
    /// GitDB object storage operation failed.
    #[error("gitdb object storage error: {0}")]
    GitDbObjectStorage(String),
    /// Git pool operation failed with a concrete failure phase.
    #[error("git pool {phase}: {message}")]
    GitPool {
        /// Stable failure phase such as `git_missing` or `push_failed`.
        phase: &'static str,
        /// Concrete cause.
        message: String,
    },
    /// Internal in-memory lock was poisoned and could not be recovered.
    #[error("in-memory drive store lock failed")]
    LockPoisoned,
    /// Version values exceeded the supported PostgreSQL range.
    #[error("version value is outside supported range: {0}")]
    VersionOutOfRange(u64),
}

/// File-system entry kind tracked by drive metadata.
#[apply(serde_eq_copy)]
pub enum DriveEntryKind {
    /// Regular file.
    File,
    /// Directory marker.
    Directory,
}

/// Metadata record for a remote drive entry.
#[apply(serde_eq)]
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
#[apply(serde_eq)]
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
#[apply(serde_eq)]
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
#[apply(serde_eq)]
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
#[apply(serde_code)]
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

/// Durable sync task status.
#[apply(serde_code)]
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

/// Queue item persisted for sync diagnostics and retry intent.
#[apply(serde_eq)]
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
#[apply(serde_eq)]
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
#[apply(serde_eq)]
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
    ) -> DriveStoreResult<DriveEntry>;

    /// Looks up an entry by key.
    async fn get_entry(&self, key: &EntryKey) -> DriveStoreResult<Option<DriveEntry>>;

    /// Looks up an entry by id.
    async fn get_entry_by_id(&self, id: Uuid) -> DriveStoreResult<Option<DriveEntry>>;

    /// Lists entries under a prefix.
    async fn list_entries(
        &self,
        space_id: &str,
        root_alias: &RootAlias,
        prefix: &RelativePath,
    ) -> DriveStoreResult<Vec<DriveEntry>>;

    /// Lists all non-deleted entries in an owner Drive namespace.
    async fn list_entries_by_space(&self, space_id: &str) -> DriveStoreResult<Vec<DriveEntry>>;

    /// Migrates legacy namespace rows to the owner Drive namespace.
    async fn migrate_owner_drive_namespace(
        &self,
        from_owner_drive_id: &str,
        to_owner_drive_id: &str,
    ) -> DriveStoreResult<u64>;

    /// Creates or refreshes an ignore rule for a remote path.
    async fn upsert_ignored_path(
        &self,
        key: &EntryKey,
        source_device_id: &str,
    ) -> DriveStoreResult<DriveIgnoredPath>;

    /// Deletes an exact ignore rule for a remote path.
    async fn delete_ignored_path(&self, key: &EntryKey) -> DriveStoreResult<()>;

    /// Lists ignore rules, optionally scoped to a root and prefix.
    async fn list_ignored_paths(
        &self,
        space_id: &str,
        root_alias: Option<&RootAlias>,
        prefix: Option<&RelativePath>,
    ) -> DriveStoreResult<Vec<DriveIgnoredPath>>;

    /// Deletes an entry tombstone.
    async fn delete_entry(&self, key: &EntryKey) -> DriveStoreResult<()>;

    /// Inserts a new version and updates the entry latest pointer.
    async fn insert_version(&self, version: DriveVersion) -> DriveStoreResult<DriveVersion>;

    /// Returns the latest version for an entry.
    async fn latest_version(&self, entry_id: Uuid) -> DriveStoreResult<Option<DriveVersion>>;

    /// Records a conflict.
    async fn record_conflict(&self, conflict: DriveConflict) -> DriveStoreResult<DriveConflict>;

    /// Lists unresolved conflicts.
    async fn list_conflicts(&self, resolved: Option<bool>) -> DriveStoreResult<Vec<DriveConflict>>;

    /// Marks a conflict resolved.
    async fn resolve_conflict(
        &self,
        _conflict_id: Uuid,
    ) -> DriveStoreResult<Option<DriveConflict>> {
        Ok(None)
    }

    /// Enqueues a sync task.
    async fn enqueue_sync_task(
        &self,
        item: DriveSyncQueueItem,
    ) -> DriveStoreResult<DriveSyncQueueItem> {
        Ok(item)
    }

    /// Updates a sync task status.
    async fn update_sync_task(
        &self,
        _id: Uuid,
        _status: DriveSyncTaskStatus,
        _last_error: Option<&str>,
    ) -> DriveStoreResult<Option<DriveSyncQueueItem>> {
        Ok(None)
    }

    /// Lists queued sync tasks.
    async fn list_sync_queue(
        &self,
        _status: Option<DriveSyncTaskStatus>,
    ) -> DriveStoreResult<Vec<DriveSyncQueueItem>> {
        Ok(Vec::new())
    }

    /// Moves failed queue items back to pending.
    async fn retry_failed_sync_tasks(&self) -> DriveStoreResult<u64> {
        Ok(0)
    }

    /// Creates or refreshes a suspended path.
    async fn upsert_suspended_path(
        &self,
        suspension: DriveSuspendedPath,
    ) -> DriveStoreResult<DriveSuspendedPath> {
        Ok(suspension)
    }

    /// Returns a suspension by entry id.
    async fn get_suspended_path(
        &self,
        _entry_id: Uuid,
    ) -> DriveStoreResult<Option<DriveSuspendedPath>> {
        Ok(None)
    }

    /// Lists suspended paths.
    async fn list_suspended_paths(&self) -> DriveStoreResult<Vec<DriveSuspendedPath>> {
        Ok(Vec::new())
    }

    /// Removes a suspended path by entry id.
    async fn delete_suspended_path(&self, _entry_id: Uuid) -> DriveStoreResult<bool> {
        Ok(false)
    }

    /// Acquires or replaces an expired lock.
    async fn acquire_lock(&self, lock: DriveLock) -> DriveStoreResult<DriveLock>;

    /// Releases a lock by token.
    async fn release_lock(&self, entry_id: Uuid, token: &str) -> DriveStoreResult<bool>;

    /// Returns the active lock if present.
    async fn get_lock(&self, entry_id: Uuid) -> DriveStoreResult<Option<DriveLock>>;
}

/// Object byte store contract.
#[async_trait]
pub trait DriveObjectStore: Send + Sync {
    /// Stores object bytes under a content-addressed key.
    async fn put_object(&self, object_key: &str, bytes: &[u8]) -> DriveStoreResult<()>;

    /// Loads object bytes.
    async fn get_object(&self, object_key: &str) -> DriveStoreResult<Vec<u8>>;

    /// Deletes an object by key.
    async fn delete_object(&self, object_key: &str) -> DriveStoreResult<()>;

    /// Returns true when the object exists.
    async fn object_exists(&self, object_key: &str) -> DriveStoreResult<bool>;
}

/// Optional synchronization coordinator for stores that need remote VCS pulls
/// and pushes around a logical drive operation.
#[async_trait]
pub trait DriveSyncCoordinator: Send + Sync {
    /// Pulls remote state before local reads/writes.
    async fn prepare_sync(&self) -> DriveStoreResult<()>;

    /// Commits and pushes local state after successful writes.
    async fn flush_sync(&self) -> DriveStoreResult<()>;
}

/// No-op coordinator used by database/object-store backends.
#[apply(plain_default_copy_eq)]
pub struct NoopDriveSyncCoordinator;

#[async_trait]
impl DriveSyncCoordinator for NoopDriveSyncCoordinator {
    async fn prepare_sync(&self) -> DriveStoreResult<()> {
        Ok(())
    }

    async fn flush_sync(&self) -> DriveStoreResult<()> {
        Ok(())
    }
}

/// Recoverable in-memory implementation for tests and local-only smoke runs.
#[apply(plain_default_clone)]
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

    fn state(&self) -> DriveStoreResult<MutexGuard<'_, InMemoryState>> {
        self.state.lock().map_err(|_| DriveStoreError::LockPoisoned)
    }
}

#[async_trait]
impl DriveMetadataStore for InMemoryDriveMetadataStore {
    async fn upsert_entry(
        &self,
        key: &EntryKey,
        kind: DriveEntryKind,
    ) -> DriveStoreResult<DriveEntry> {
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

    async fn get_entry(&self, key: &EntryKey) -> DriveStoreResult<Option<DriveEntry>> {
        Ok(self
            .state()?
            .entries_by_key
            .get(&key.remote_path())
            .cloned())
    }

    async fn get_entry_by_id(&self, id: Uuid) -> DriveStoreResult<Option<DriveEntry>> {
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
    ) -> DriveStoreResult<Vec<DriveEntry>> {
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

    async fn list_entries_by_space(&self, space_id: &str) -> DriveStoreResult<Vec<DriveEntry>> {
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
    ) -> DriveStoreResult<u64> {
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
    ) -> DriveStoreResult<DriveIgnoredPath> {
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

    async fn delete_ignored_path(&self, key: &EntryKey) -> DriveStoreResult<()> {
        self.state()?.ignored_by_key.remove(&key.remote_path());
        Ok(())
    }

    async fn list_ignored_paths(
        &self,
        space_id: &str,
        root_alias: Option<&RootAlias>,
        prefix: Option<&RelativePath>,
    ) -> DriveStoreResult<Vec<DriveIgnoredPath>> {
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

    async fn delete_entry(&self, key: &EntryKey) -> DriveStoreResult<()> {
        let mut state = self.state()?;
        if let Some(entry) = state.entries_by_key.get_mut(&key.remote_path()) {
            entry.deleted = true;
            entry.updated_at = Utc::now();
        }
        Ok(())
    }

    async fn insert_version(&self, version: DriveVersion) -> DriveStoreResult<DriveVersion> {
        let mut state = self.state()?;
        let key = state
            .entries_by_id
            .get(&version.entry_id)
            .cloned()
            .ok_or_else(|| DriveStoreError::EntryNotFound(version.entry_id.to_string()))?;
        let entry = state
            .entries_by_key
            .get_mut(&key)
            .ok_or_else(|| DriveStoreError::EntryNotFound(key.clone()))?;
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

    async fn latest_version(&self, entry_id: Uuid) -> DriveStoreResult<Option<DriveVersion>> {
        Ok(self
            .state()?
            .versions
            .get(&entry_id)
            .and_then(|versions| versions.iter().max_by_key(|version| version.version))
            .cloned())
    }

    async fn record_conflict(&self, conflict: DriveConflict) -> DriveStoreResult<DriveConflict> {
        self.state()?.conflicts.push(conflict.clone());
        Ok(conflict)
    }

    async fn list_conflicts(&self, resolved: Option<bool>) -> DriveStoreResult<Vec<DriveConflict>> {
        Ok(self
            .state()?
            .conflicts
            .iter()
            .filter(|conflict| resolved.is_none_or(|value| conflict.resolved == value))
            .cloned()
            .collect())
    }

    async fn resolve_conflict(&self, conflict_id: Uuid) -> DriveStoreResult<Option<DriveConflict>> {
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
    ) -> DriveStoreResult<DriveSyncQueueItem> {
        self.state()?.sync_queue.insert(item.id, item.clone());
        Ok(item)
    }

    async fn update_sync_task(
        &self,
        id: Uuid,
        status: DriveSyncTaskStatus,
        last_error: Option<&str>,
    ) -> DriveStoreResult<Option<DriveSyncQueueItem>> {
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
    ) -> DriveStoreResult<Vec<DriveSyncQueueItem>> {
        let mut items = self
            .state()?
            .sync_queue
            .values()
            .filter(|item| status.is_none_or(|status| item.status == status))
            .cloned()
            .collect::<Vec<_>>();
        items.sort_by(|left, right| left.created_at.cmp(&right.created_at));
        Ok(items)
    }

    async fn retry_failed_sync_tasks(&self) -> DriveStoreResult<u64> {
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
    ) -> DriveStoreResult<DriveSuspendedPath> {
        self.state()?
            .suspended_by_entry
            .insert(suspension.entry_id, suspension.clone());
        Ok(suspension)
    }

    async fn get_suspended_path(
        &self,
        entry_id: Uuid,
    ) -> DriveStoreResult<Option<DriveSuspendedPath>> {
        Ok(self.state()?.suspended_by_entry.get(&entry_id).cloned())
    }

    async fn list_suspended_paths(&self) -> DriveStoreResult<Vec<DriveSuspendedPath>> {
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

    async fn delete_suspended_path(&self, entry_id: Uuid) -> DriveStoreResult<bool> {
        Ok(self.state()?.suspended_by_entry.remove(&entry_id).is_some())
    }

    async fn acquire_lock(&self, lock: DriveLock) -> DriveStoreResult<DriveLock> {
        let mut state = self.state()?;
        if let Some(existing) = state.locks.get(&lock.entry_id)
            && existing.expires_at > Utc::now()
            && existing.owner_device_id != lock.owner_device_id
        {
            return Err(DriveStoreError::LockedByOther {
                owner_device_id: existing.owner_device_id.clone(),
            });
        }
        state.locks.insert(lock.entry_id, lock.clone());
        Ok(lock)
    }

    async fn release_lock(&self, entry_id: Uuid, token: &str) -> DriveStoreResult<bool> {
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

    async fn get_lock(&self, entry_id: Uuid) -> DriveStoreResult<Option<DriveLock>> {
        Ok(self.state()?.locks.get(&entry_id).cloned())
    }
}

/// In-memory object byte store.
#[apply(plain_default_clone)]
pub struct InMemoryDriveObjectStore {
    objects: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl InMemoryDriveObjectStore {
    /// Creates an empty in-memory object store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn objects(&self) -> DriveStoreResult<MutexGuard<'_, HashMap<String, Vec<u8>>>> {
        self.objects
            .lock()
            .map_err(|_| DriveStoreError::LockPoisoned)
    }
}

#[async_trait]
impl DriveObjectStore for InMemoryDriveObjectStore {
    async fn put_object(&self, object_key: &str, bytes: &[u8]) -> DriveStoreResult<()> {
        self.objects()?
            .insert(object_key.to_owned(), bytes.to_vec());
        Ok(())
    }

    async fn get_object(&self, object_key: &str) -> DriveStoreResult<Vec<u8>> {
        self.objects()?
            .get(object_key)
            .cloned()
            .ok_or_else(|| DriveStoreError::ObjectNotFound(object_key.to_owned()))
    }

    async fn delete_object(&self, object_key: &str) -> DriveStoreResult<()> {
        self.objects()?.remove(object_key);
        Ok(())
    }

    async fn object_exists(&self, object_key: &str) -> DriveStoreResult<bool> {
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

#[cfg(test)]
mod tests {
    use super::{
        DriveEntryKind, DriveMetadataStore, DriveObjectStore, DriveStoreError, DriveVersion,
        InMemoryDriveMetadataStore, InMemoryDriveObjectStore,
    };
    use az_drive_core::{EntryKey, RelativePath, RootAlias, content_hash, object_key_for_hash};
    use chrono::Utc;
    use std::error::Error as _;
    use uuid::Uuid;

    fn key() -> EntryKey {
        EntryKey::new(
            "main",
            RootAlias::parse("workspace").expect("alias should parse"),
            RelativePath::parse("docs/a.md").expect("path should parse"),
        )
    }

    #[test]
    fn object_storage_error_preserves_source_chain() {
        let err = DriveStoreError::from(az_rustfs::StorageError::Backend("offline".to_owned()));
        let source = err
            .source()
            .expect("object storage error should preserve source");

        assert_eq!(source.to_string(), "storage backend error: offline");
    }

    #[tokio::test]
    async fn in_memory_store_tracks_latest_version() {
        let store = InMemoryDriveMetadataStore::new();
        let entry = store
            .upsert_entry(&key(), DriveEntryKind::File)
            .await
            .expect("entry should upsert");
        let hash = content_hash(b"hello");
        let version = DriveVersion {
            id: Uuid::new_v4(),
            entry_id: entry.id,
            version: 1,
            content_hash: hash.clone(),
            object_key: object_key_for_hash(&hash),
            size_bytes: 5,
            device_id: "device-a".to_owned(),
            modified_at: Utc::now(),
        };

        store
            .insert_version(version)
            .await
            .expect("version should insert");
        let latest = store
            .latest_version(entry.id)
            .await
            .expect("latest version query should work")
            .expect("latest version should exist");

        assert_eq!(latest.content_hash, hash);
    }

    #[tokio::test]
    async fn in_memory_store_lists_entries_by_space() {
        let store = InMemoryDriveMetadataStore::new();
        store
            .upsert_entry(&key(), DriveEntryKind::File)
            .await
            .expect("main entry should upsert");
        store
            .upsert_entry(
                &EntryKey::new(
                    "other",
                    RootAlias::parse("workspace").expect("alias should parse"),
                    RelativePath::parse("docs/b.md").expect("path should parse"),
                ),
                DriveEntryKind::File,
            )
            .await
            .expect("other entry should upsert");

        let entries = store
            .list_entries_by_space("main")
            .await
            .expect("space entries should list");

        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn in_memory_store_migrates_legacy_main_namespace_to_owner_drive() {
        let store = InMemoryDriveMetadataStore::new();
        let entry = store
            .upsert_entry(&key(), DriveEntryKind::File)
            .await
            .expect("entry should upsert");
        let hash = content_hash(b"hello");
        store
            .insert_version(DriveVersion {
                id: Uuid::new_v4(),
                entry_id: entry.id,
                version: 1,
                content_hash: hash.clone(),
                object_key: object_key_for_hash(&hash),
                size_bytes: 5,
                device_id: "device-a".to_owned(),
                modified_at: Utc::now(),
            })
            .await
            .expect("version should insert");
        store
            .upsert_ignored_path(&key(), "device-a")
            .await
            .expect("ignore should upsert");

        let migrated = store
            .migrate_owner_drive_namespace("main", "user-zjarlin")
            .await
            .expect("namespace should migrate");

        assert_eq!(migrated, 2);
        assert!(
            store
                .list_entries_by_space("main")
                .await
                .expect("main entries should list")
                .is_empty()
        );
        assert_eq!(
            store
                .list_entries_by_space("user-zjarlin")
                .await
                .expect("owner entries should list")
                .len(),
            1
        );
        assert_eq!(
            store
                .list_ignored_paths("user-zjarlin", None, None)
                .await
                .expect("owner ignored paths should list")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn in_memory_store_lists_ignored_paths_by_prefix() {
        let store = InMemoryDriveMetadataStore::new();
        store
            .upsert_ignored_path(&key(), "device-a")
            .await
            .expect("ignore should upsert");

        let ignored = store
            .list_ignored_paths(
                "main",
                Some(&RootAlias::parse("workspace").expect("alias should parse")),
                Some(&RelativePath::parse("docs").expect("prefix should parse")),
            )
            .await
            .expect("ignored paths should list");

        assert_eq!(ignored[0].relative_path.as_str(), "docs/a.md");
    }

    #[tokio::test]
    async fn in_memory_object_store_round_trips_bytes() {
        let store = InMemoryDriveObjectStore::new();

        store
            .put_object("objects/demo", b"hello")
            .await
            .expect("object should store");
        let bytes = store
            .get_object("objects/demo")
            .await
            .expect("object should load");

        assert_eq!(bytes, b"hello");
    }
}
