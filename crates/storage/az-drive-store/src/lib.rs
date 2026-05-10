#![forbid(unsafe_code)]

//! Metadata and object storage abstractions for the standalone drive.
//!
//! PostgreSQL is the formal metadata store, while the in-memory store exists
//! for tests and local smoke runs. Object bytes are stored by content hash.

use async_trait::async_trait;
use az_drive_core::{EntryKey, RelativePath, RootAlias};
use az_rustfs::{S3StorageClient, StorageError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, MutexGuard};
use thiserror::Error;
use uuid::Uuid;

/// Built-in PostgreSQL migration for the drive metadata schema.
pub const DRIVE_MIGRATION_SQL: &str = include_str!("../migrations/0001_drive.sql");

/// Result alias for drive store operations.
pub type DriveStoreResult<T> = Result<T, DriveStoreError>;

/// Storage-layer errors with preserved root cause context.
#[derive(Debug, Error)]
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
    /// Object storage operation failed.
    #[error("object storage error: {0}")]
    ObjectStorage(String),
    /// Internal in-memory lock was poisoned and could not be recovered.
    #[error("in-memory drive store lock failed")]
    LockPoisoned,
    /// Version values exceeded the supported PostgreSQL range.
    #[error("version value is outside supported range: {0}")]
    VersionOutOfRange(u64),
}

impl From<StorageError> for DriveStoreError {
    fn from(value: StorageError) -> Self {
        Self::ObjectStorage(value.to_string())
    }
}

/// File-system entry kind tracked by drive metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DriveEntryKind {
    /// Regular file.
    File,
    /// Directory marker.
    Directory,
}

impl DriveEntryKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "directory" => Self::Directory,
            _ => Self::File,
        }
    }
}

/// Metadata record for a remote drive entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

/// Recoverable in-memory implementation for tests and local-only smoke runs.
#[derive(Clone, Default)]
pub struct InMemoryDriveMetadataStore {
    state: Arc<Mutex<InMemoryState>>,
}

#[derive(Default)]
struct InMemoryState {
    entries_by_key: BTreeMap<String, DriveEntry>,
    entries_by_id: HashMap<Uuid, String>,
    versions: BTreeMap<Uuid, Vec<DriveVersion>>,
    locks: HashMap<Uuid, DriveLock>,
    conflicts: Vec<DriveConflict>,
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

/// PostgreSQL metadata store.
#[derive(Clone)]
pub struct PgDriveMetadataStore {
    pool: sqlx::PgPool,
}

impl PgDriveMetadataStore {
    /// Connects to PostgreSQL.
    ///
    /// # Errors
    /// Returns [`DriveStoreError`] when the database connection fails.
    pub async fn connect(database_url: &str) -> DriveStoreResult<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(8)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    /// Creates a store from an existing pool.
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Runs the built-in migration statements.
    ///
    /// # Errors
    /// Returns [`DriveStoreError`] when any statement fails.
    pub async fn run_migrations(&self) -> DriveStoreResult<()> {
        for statement in split_sql_statements(DRIVE_MIGRATION_SQL) {
            sqlx::query(&statement).execute(&self.pool).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl DriveMetadataStore for PgDriveMetadataStore {
    async fn upsert_entry(
        &self,
        key: &EntryKey,
        kind: DriveEntryKind,
    ) -> DriveStoreResult<DriveEntry> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO drive_entries (
                id, space_id, root_alias, relative_path, kind, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (space_id, root_alias, relative_path)
            DO UPDATE SET kind = EXCLUDED.kind, deleted = FALSE, updated_at = NOW()
            "#,
        )
        .bind(id)
        .bind(&key.space_id)
        .bind(key.root_alias.as_str())
        .bind(key.relative_path.as_str())
        .bind(kind.as_str())
        .execute(&self.pool)
        .await?;

        self.get_entry(key)
            .await?
            .ok_or_else(|| DriveStoreError::EntryNotFound(key.remote_path()))
    }

    async fn get_entry(&self, key: &EntryKey) -> DriveStoreResult<Option<DriveEntry>> {
        let row = sqlx::query_as::<
            _,
            (
                Uuid,
                String,
                String,
                String,
                String,
                i64,
                Option<String>,
                bool,
                DateTime<Utc>,
            ),
        >(
            r#"
            SELECT id, space_id, root_alias, relative_path, kind, latest_version,
                   latest_hash, deleted, updated_at
            FROM drive_entries
            WHERE space_id = $1 AND root_alias = $2 AND relative_path = $3
            "#,
        )
        .bind(&key.space_id)
        .bind(key.root_alias.as_str())
        .bind(key.relative_path.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_entry).transpose()
    }

    async fn get_entry_by_id(&self, id: Uuid) -> DriveStoreResult<Option<DriveEntry>> {
        let row = sqlx::query_as::<
            _,
            (
                Uuid,
                String,
                String,
                String,
                String,
                i64,
                Option<String>,
                bool,
                DateTime<Utc>,
            ),
        >(
            r#"
            SELECT id, space_id, root_alias, relative_path, kind, latest_version,
                   latest_hash, deleted, updated_at
            FROM drive_entries
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_entry).transpose()
    }

    async fn list_entries(
        &self,
        space_id: &str,
        root_alias: &RootAlias,
        prefix: &RelativePath,
    ) -> DriveStoreResult<Vec<DriveEntry>> {
        let like_prefix = if prefix.is_root() {
            "%".to_owned()
        } else {
            format!("{}/%", prefix.as_str())
        };
        let rows = sqlx::query_as::<
            _,
            (
                Uuid,
                String,
                String,
                String,
                String,
                i64,
                Option<String>,
                bool,
                DateTime<Utc>,
            ),
        >(
            r#"
            SELECT id, space_id, root_alias, relative_path, kind, latest_version,
                   latest_hash, deleted, updated_at
            FROM drive_entries
            WHERE space_id = $1
              AND root_alias = $2
              AND deleted = FALSE
              AND ($3 = '' OR relative_path = $3 OR relative_path LIKE $4)
            ORDER BY relative_path
            "#,
        )
        .bind(space_id)
        .bind(root_alias.as_str())
        .bind(prefix.as_str())
        .bind(like_prefix)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_entry).collect()
    }

    async fn delete_entry(&self, key: &EntryKey) -> DriveStoreResult<()> {
        sqlx::query(
            r#"
            UPDATE drive_entries
            SET deleted = TRUE, updated_at = NOW()
            WHERE space_id = $1 AND root_alias = $2 AND relative_path = $3
            "#,
        )
        .bind(&key.space_id)
        .bind(key.root_alias.as_str())
        .bind(key.relative_path.as_str())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn insert_version(&self, version: DriveVersion) -> DriveStoreResult<DriveVersion> {
        let version_i64 = to_i64_version(version.version)?;
        let size_i64 = i64::try_from(version.size_bytes)
            .map_err(|_| DriveStoreError::VersionOutOfRange(version.size_bytes))?;
        sqlx::query(
            r#"
            INSERT INTO drive_versions (
                id, entry_id, version, content_hash, object_key, size_bytes,
                device_id, modified_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (entry_id, version) DO NOTHING
            "#,
        )
        .bind(version.id)
        .bind(version.entry_id)
        .bind(version_i64)
        .bind(&version.content_hash)
        .bind(&version.object_key)
        .bind(size_i64)
        .bind(&version.device_id)
        .bind(version.modified_at)
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
            UPDATE drive_entries
            SET latest_version = GREATEST(latest_version, $2),
                latest_hash = $3,
                deleted = FALSE,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(version.entry_id)
        .bind(version_i64)
        .bind(&version.content_hash)
        .execute(&self.pool)
        .await?;
        Ok(version)
    }

    async fn latest_version(&self, entry_id: Uuid) -> DriveStoreResult<Option<DriveVersion>> {
        let row =
            sqlx::query_as::<_, (Uuid, Uuid, i64, String, String, i64, String, DateTime<Utc>)>(
                r#"
            SELECT id, entry_id, version, content_hash, object_key, size_bytes,
                   device_id, modified_at
            FROM drive_versions
            WHERE entry_id = $1
            ORDER BY version DESC
            LIMIT 1
            "#,
            )
            .bind(entry_id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(row_to_version).transpose()
    }

    async fn record_conflict(&self, conflict: DriveConflict) -> DriveStoreResult<DriveConflict> {
        sqlx::query(
            r#"
            INSERT INTO drive_conflicts (
                id, entry_id, base_version, local_hash, remote_hash,
                device_id, conflict_path, resolved, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(conflict.id)
        .bind(conflict.entry_id)
        .bind(conflict.base_version.map(to_i64_version).transpose()?)
        .bind(&conflict.local_hash)
        .bind(&conflict.remote_hash)
        .bind(&conflict.device_id)
        .bind(&conflict.conflict_path)
        .bind(conflict.resolved)
        .bind(conflict.created_at)
        .execute(&self.pool)
        .await?;
        Ok(conflict)
    }

    async fn list_conflicts(&self, resolved: Option<bool>) -> DriveStoreResult<Vec<DriveConflict>> {
        let rows = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Option<i64>,
                String,
                String,
                String,
                String,
                bool,
                DateTime<Utc>,
            ),
        >(
            r#"
            SELECT id, entry_id, base_version, local_hash, remote_hash, device_id,
                   conflict_path, resolved, created_at
            FROM drive_conflicts
            WHERE ($1::BOOLEAN IS NULL OR resolved = $1)
            ORDER BY created_at DESC
            "#,
        )
        .bind(resolved)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_conflict).collect()
    }

    async fn acquire_lock(&self, lock: DriveLock) -> DriveStoreResult<DriveLock> {
        if let Some(existing) = self.get_lock(lock.entry_id).await?
            && existing.expires_at > Utc::now()
            && existing.owner_device_id != lock.owner_device_id
        {
            return Err(DriveStoreError::LockedByOther {
                owner_device_id: existing.owner_device_id,
            });
        }
        sqlx::query(
            r#"
            INSERT INTO drive_locks (
                entry_id, owner_device_id, owner_name, token, expires_at
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (entry_id)
            DO UPDATE SET owner_device_id = EXCLUDED.owner_device_id,
                          owner_name = EXCLUDED.owner_name,
                          token = EXCLUDED.token,
                          expires_at = EXCLUDED.expires_at,
                          created_at = NOW()
            "#,
        )
        .bind(lock.entry_id)
        .bind(&lock.owner_device_id)
        .bind(&lock.owner_name)
        .bind(&lock.token)
        .bind(lock.expires_at)
        .execute(&self.pool)
        .await?;
        Ok(lock)
    }

    async fn release_lock(&self, entry_id: Uuid, token: &str) -> DriveStoreResult<bool> {
        let result = sqlx::query("DELETE FROM drive_locks WHERE entry_id = $1 AND token = $2")
            .bind(entry_id)
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn get_lock(&self, entry_id: Uuid) -> DriveStoreResult<Option<DriveLock>> {
        let row = sqlx::query_as::<_, (Uuid, String, String, String, DateTime<Utc>)>(
            r#"
            SELECT entry_id, owner_device_id, owner_name, token, expires_at
            FROM drive_locks
            WHERE entry_id = $1
            "#,
        )
        .bind(entry_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(entry_id, owner_device_id, owner_name, token, expires_at)| DriveLock {
                entry_id,
                owner_device_id,
                owner_name,
                token,
                expires_at,
            },
        ))
    }
}

/// S3-compatible object store backed by `az-rustfs`.
#[derive(Clone)]
pub struct S3DriveObjectStore {
    client: Arc<dyn S3StorageClient>,
    bucket: String,
}

impl S3DriveObjectStore {
    /// Creates a new object store and ensures the bucket exists.
    ///
    /// # Errors
    /// Returns [`DriveStoreError`] when bucket initialization fails.
    pub fn new(
        client: Arc<dyn S3StorageClient>,
        bucket: impl Into<String>,
    ) -> DriveStoreResult<Self> {
        let bucket = bucket.into();
        if !client.bucket_exists(&bucket)? {
            client.create_bucket(&bucket)?;
        }
        Ok(Self { client, bucket })
    }
}

#[async_trait]
impl DriveObjectStore for S3DriveObjectStore {
    async fn put_object(&self, object_key: &str, bytes: &[u8]) -> DriveStoreResult<()> {
        let client = Arc::clone(&self.client);
        let bucket = self.bucket.clone();
        let object_key = object_key.to_owned();
        let bytes = bytes.to_vec();
        tokio::task::spawn_blocking(move || {
            client.put_object_bytes(&bucket, &object_key, &bytes, None, &BTreeMap::new())
        })
        .await
        .map_err(|err| DriveStoreError::ObjectStorage(err.to_string()))??;
        Ok(())
    }

    async fn get_object(&self, object_key: &str) -> DriveStoreResult<Vec<u8>> {
        let client = Arc::clone(&self.client);
        let bucket = self.bucket.clone();
        let object_key = object_key.to_owned();
        tokio::task::spawn_blocking(move || client.get_object(&bucket, &object_key))
            .await
            .map_err(|err| DriveStoreError::ObjectStorage(err.to_string()))?
            .map_err(Into::into)
    }

    async fn delete_object(&self, object_key: &str) -> DriveStoreResult<()> {
        let client = Arc::clone(&self.client);
        let bucket = self.bucket.clone();
        let object_key = object_key.to_owned();
        tokio::task::spawn_blocking(move || client.delete_object(&bucket, &object_key))
            .await
            .map_err(|err| DriveStoreError::ObjectStorage(err.to_string()))??;
        Ok(())
    }

    async fn object_exists(&self, object_key: &str) -> DriveStoreResult<bool> {
        let client = Arc::clone(&self.client);
        let bucket = self.bucket.clone();
        let object_key = object_key.to_owned();
        tokio::task::spawn_blocking(move || client.object_exists(&bucket, &object_key))
            .await
            .map_err(|err| DriveStoreError::ObjectStorage(err.to_string()))?
            .map_err(Into::into)
    }
}

fn row_to_entry(
    row: (
        Uuid,
        String,
        String,
        String,
        String,
        i64,
        Option<String>,
        bool,
        DateTime<Utc>,
    ),
) -> DriveStoreResult<DriveEntry> {
    let (
        id,
        space_id,
        root_alias,
        relative_path,
        kind,
        latest_version,
        latest_hash,
        deleted,
        updated_at,
    ) = row;
    Ok(DriveEntry {
        id,
        key: EntryKey::new(
            space_id,
            RootAlias::parse(&root_alias)
                .map_err(|err| DriveStoreError::ObjectStorage(err.to_string()))?,
            RelativePath::parse(&relative_path)
                .map_err(|err| DriveStoreError::ObjectStorage(err.to_string()))?,
        ),
        kind: DriveEntryKind::parse(&kind),
        latest_version: u64::try_from(latest_version).map_err(|_| {
            DriveStoreError::ObjectStorage("negative version in database".to_owned())
        })?,
        latest_hash,
        deleted,
        updated_at,
    })
}

fn row_to_version(
    row: (Uuid, Uuid, i64, String, String, i64, String, DateTime<Utc>),
) -> DriveStoreResult<DriveVersion> {
    let (id, entry_id, version, content_hash, object_key, size_bytes, device_id, modified_at) = row;
    Ok(DriveVersion {
        id,
        entry_id,
        version: u64::try_from(version).map_err(|_| {
            DriveStoreError::ObjectStorage("negative version in database".to_owned())
        })?,
        content_hash,
        object_key,
        size_bytes: u64::try_from(size_bytes)
            .map_err(|_| DriveStoreError::ObjectStorage("negative size in database".to_owned()))?,
        device_id,
        modified_at,
    })
}

fn row_to_conflict(
    row: (
        Uuid,
        Uuid,
        Option<i64>,
        String,
        String,
        String,
        String,
        bool,
        DateTime<Utc>,
    ),
) -> DriveStoreResult<DriveConflict> {
    let (
        id,
        entry_id,
        base_version,
        local_hash,
        remote_hash,
        device_id,
        conflict_path,
        resolved,
        created_at,
    ) = row;
    Ok(DriveConflict {
        id,
        entry_id,
        base_version: base_version.map(u64::try_from).transpose().map_err(|_| {
            DriveStoreError::ObjectStorage("negative version in database".to_owned())
        })?,
        local_hash,
        remote_hash,
        device_id,
        conflict_path,
        resolved,
        created_at,
    })
}

fn to_i64_version(value: u64) -> DriveStoreResult<i64> {
    i64::try_from(value).map_err(|_| DriveStoreError::VersionOutOfRange(value))
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        DRIVE_MIGRATION_SQL, DriveEntryKind, DriveMetadataStore, DriveObjectStore, DriveVersion,
        InMemoryDriveMetadataStore, InMemoryDriveObjectStore,
    };
    use az_drive_core::{EntryKey, RelativePath, RootAlias, content_hash, object_key_for_hash};
    use chrono::Utc;
    use uuid::Uuid;

    fn key() -> EntryKey {
        EntryKey::new(
            "main",
            RootAlias::parse("workspace").expect("alias should parse"),
            RelativePath::parse("docs/a.md").expect("path should parse"),
        )
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

    #[test]
    fn migration_declares_drive_entries_table() {
        assert!(DRIVE_MIGRATION_SQL.contains("CREATE TABLE IF NOT EXISTS drive_entries"));
    }
}
