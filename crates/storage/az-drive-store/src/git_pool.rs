use async_trait::async_trait;
use az_derive_aliases::{apply, plain_eq, serde_eq};
use az_drive_core::{EntryKey, RelativePath, RootAlias};
use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

use crate::{
    DriveConflict, DriveEntry, DriveEntryKind, DriveIgnoredPath, DriveLock, DriveMetadataStore,
    DriveObjectStore, DriveStoreError, DriveStoreResult, DriveSuspendedPath, DriveSyncCoordinator,
    DriveSyncQueueItem, DriveSyncTaskStatus, DriveVersion,
};

pub const DEFAULT_GIT_POOL_LIMIT_BYTES: u64 = 8 * 1024 * 1024 * 1024;
pub const DEFAULT_AUTO_GIT_POOL_PREFIX: &str = "auto";
const SCHEMA_VERSION: u32 = 1;
const CONTROL_DIR: &str = "control";
const POOLS_DIR: &str = "pools";
const POOL_INDEX_DIR: &str = "aio-pool-index";

#[apply(serde_eq)]
pub struct GitPoolConfig {
    pub root: PathBuf,
    pub owner_drive_id: String,
    #[serde(default)]
    pub control_remote: Option<String>,
    #[serde(default = "default_pool_limit_bytes")]
    pub default_pool_limit_bytes: u64,
    #[serde(default)]
    pub auto_pool_root: Option<PathBuf>,
    #[serde(default = "default_auto_pool_prefix")]
    pub auto_pool_prefix: String,
}

impl GitPoolConfig {
    #[must_use]
    pub fn new(root: PathBuf, owner_drive_id: impl Into<String>) -> Self {
        Self {
            root,
            owner_drive_id: owner_drive_id.into(),
            control_remote: None,
            default_pool_limit_bytes: DEFAULT_GIT_POOL_LIMIT_BYTES,
            auto_pool_root: None,
            auto_pool_prefix: default_auto_pool_prefix(),
        }
    }
}

#[apply(serde_eq)]
pub struct GitPoolRepoConfig {
    pub name: String,
    pub remote_url: String,
    pub owner_drive_id: String,
    pub readonly: bool,
    pub max_size_bytes: u64,
    pub used_bytes: u64,
}

#[apply(serde_eq)]
pub struct GitPoolMountConfig {
    pub name: String,
    pub remote_url: String,
    pub owner_drive_id: String,
    pub readonly: bool,
}

#[apply(serde_eq)]
struct DriveRecord {
    schema_version: u32,
    owner_drive_id: String,
}

#[apply(serde_eq)]
struct EntryRecord {
    entry: DriveEntry,
    pool_name: Option<String>,
}

#[apply(serde_eq)]
struct VersionRecord {
    version: DriveVersion,
}

#[apply(serde_eq)]
struct IgnoredRecord {
    ignored: DriveIgnoredPath,
}

#[apply(serde_eq)]
struct ConflictRecord {
    conflict: DriveConflict,
}

#[apply(serde_eq)]
struct SyncQueueRecord {
    item: DriveSyncQueueItem,
}

#[apply(serde_eq)]
struct SuspendedRecord {
    suspension: DriveSuspendedPath,
}

#[apply(plain_eq)]
pub struct GitPoolDriveStore {
    config: GitPoolConfig,
}

impl GitPoolDriveStore {
    pub fn open(config: GitPoolConfig) -> DriveStoreResult<Self> {
        let store = Self { config };
        store.ensure_layout()?;
        Ok(store)
    }

    #[must_use]
    pub fn config(&self) -> &GitPoolConfig {
        &self.config
    }

    pub fn init_pool(
        &self,
        name: &str,
        remote_url: &str,
        max_size_bytes: Option<u64>,
    ) -> DriveStoreResult<GitPoolRepoConfig> {
        validate_pool_name(name)?;
        let pool = GitPoolRepoConfig {
            name: name.to_owned(),
            remote_url: remote_url.to_owned(),
            owner_drive_id: self.config.owner_drive_id.clone(),
            readonly: false,
            max_size_bytes: max_size_bytes.unwrap_or(self.config.default_pool_limit_bytes),
            used_bytes: 0,
        };
        self.ensure_pool_worktree(&pool)?;
        write_json(&self.pool_config_path(name), &pool)?;
        self.write_drive_record()?;
        self.flush_sync_blocking()?;
        Ok(pool)
    }

    pub fn mount_pool(
        &self,
        name: &str,
        remote_url: &str,
        owner_drive_id: &str,
        readonly: bool,
    ) -> DriveStoreResult<GitPoolMountConfig> {
        validate_pool_name(name)?;
        let mount = GitPoolMountConfig {
            name: name.to_owned(),
            remote_url: remote_url.to_owned(),
            owner_drive_id: owner_drive_id.to_owned(),
            readonly,
        };
        let pool = GitPoolRepoConfig {
            name: name.to_owned(),
            remote_url: remote_url.to_owned(),
            owner_drive_id: owner_drive_id.to_owned(),
            readonly: true,
            max_size_bytes: 0,
            used_bytes: 0,
        };
        self.ensure_pool_worktree(&pool)?;
        write_json(&self.mount_config_path(name), &mount)?;
        self.flush_sync_blocking()?;
        Ok(mount)
    }

    pub fn unmount_pool(&self, name: &str) -> DriveStoreResult<()> {
        validate_pool_name(name)?;
        remove_file_if_exists(&self.mount_config_path(name))?;
        self.flush_sync_blocking()
    }

    pub fn list_pools(&self) -> DriveStoreResult<Vec<GitPoolRepoConfig>> {
        let mut pools = Vec::new();
        for path in json_files(&self.control_path().join("pools"))? {
            pools.push(read_json(&path)?);
        }
        pools.sort_by(|left: &GitPoolRepoConfig, right| left.name.cmp(&right.name));
        Ok(pools)
    }

    pub fn list_mounts(&self) -> DriveStoreResult<Vec<GitPoolMountConfig>> {
        let mut mounts = Vec::new();
        for path in json_files(&self.control_path().join("mounts"))? {
            mounts.push(read_json(&path)?);
        }
        mounts.sort_by(|left: &GitPoolMountConfig, right| left.name.cmp(&right.name));
        Ok(mounts)
    }

    pub fn backend_status(&self) -> DriveStoreResult<serde_json::Value> {
        Ok(serde_json::json!({
            "backend": "git_pool",
            "root": self.config.root.clone(),
            "control": self.control_path(),
            "owner_drive_id": self.config.owner_drive_id.clone(),
            "auto_pool_root": self.config.auto_pool_root.clone(),
            "auto_pool_prefix": self.config.auto_pool_prefix.clone(),
            "pools": self.list_pools()?,
            "mounts": self.list_mounts()?,
        }))
    }

    fn ensure_layout(&self) -> DriveStoreResult<()> {
        fs::create_dir_all(&self.config.root).map_err(|err| io_error("init_failed", err))?;
        self.ensure_control_worktree()?;
        for dir in [
            self.control_path().join("pools"),
            self.control_path().join("mounts"),
            self.control_path().join("index"),
            self.control_path().join("versions"),
            self.control_path().join("ignored"),
            self.control_path().join("conflicts"),
            self.control_path().join("sync-queue"),
            self.control_path().join("suspended"),
        ] {
            fs::create_dir_all(&dir).map_err(|err| io_error("init_failed", err))?;
        }
        self.write_drive_record()?;
        self.flush_sync_blocking()?;
        Ok(())
    }

    fn write_drive_record(&self) -> DriveStoreResult<()> {
        write_json(
            &self.control_path().join("drive.json"),
            &DriveRecord {
                schema_version: SCHEMA_VERSION,
                owner_drive_id: self.config.owner_drive_id.clone(),
            },
        )
    }

    fn ensure_control_worktree(&self) -> DriveStoreResult<()> {
        ensure_worktree(
            &self.control_path(),
            self.config.control_remote.as_deref(),
            "control_pull_failed",
        )
    }

    fn ensure_pool_worktree(&self, pool: &GitPoolRepoConfig) -> DriveStoreResult<()> {
        let path = self.pool_path(&pool.name);
        ensure_worktree(&path, Some(&pool.remote_url), "pool_pull_failed")?;
        for dir in [
            path.join("objects/sha256"),
            path.join(POOL_INDEX_DIR).join("entries"),
            path.join(POOL_INDEX_DIR).join("versions"),
        ] {
            fs::create_dir_all(&dir).map_err(|err| io_error("init_failed", err))?;
        }
        commit_and_push_repo(&path, "initialize aio drive pool", "push_failed")
    }

    fn ensure_pool_checkout(&self, pool: &GitPoolRepoConfig) -> DriveStoreResult<()> {
        ensure_worktree(
            &self.pool_path(&pool.name),
            Some(&pool.remote_url),
            "pool_pull_failed",
        )
    }

    fn control_path(&self) -> PathBuf {
        self.config.root.join(CONTROL_DIR)
    }

    fn pools_path(&self) -> PathBuf {
        self.config.root.join(POOLS_DIR)
    }

    fn pool_path(&self, name: &str) -> PathBuf {
        self.pools_path().join(name)
    }

    fn pool_config_path(&self, name: &str) -> PathBuf {
        self.control_path()
            .join("pools")
            .join(format!("{name}.json"))
    }

    fn mount_config_path(&self, name: &str) -> PathBuf {
        self.control_path()
            .join("mounts")
            .join(format!("{name}.json"))
    }

    fn entry_path(&self, id: Uuid) -> PathBuf {
        self.control_path().join("index").join(format!("{id}.json"))
    }

    fn version_path(&self, entry_id: Uuid, version: u64) -> PathBuf {
        self.control_path()
            .join("versions")
            .join(entry_id.to_string())
            .join(format!("{version}.json"))
    }

    fn ignored_path(&self, id: Uuid) -> PathBuf {
        self.control_path()
            .join("ignored")
            .join(format!("{id}.json"))
    }

    fn conflict_path(&self, id: Uuid) -> PathBuf {
        self.control_path()
            .join("conflicts")
            .join(format!("{id}.json"))
    }

    fn sync_queue_path(&self, id: Uuid) -> PathBuf {
        self.control_path()
            .join("sync-queue")
            .join(format!("{id}.json"))
    }

    fn suspended_path(&self, entry_id: Uuid) -> PathBuf {
        self.control_path()
            .join("suspended")
            .join(format!("{entry_id}.json"))
    }

    fn pool_entry_path(&self, pool_name: &str, id: Uuid) -> PathBuf {
        self.pool_path(pool_name)
            .join(POOL_INDEX_DIR)
            .join("entries")
            .join(format!("{id}.json"))
    }

    fn pool_version_path(&self, pool_name: &str, entry_id: Uuid, version: u64) -> PathBuf {
        self.pool_path(pool_name)
            .join(POOL_INDEX_DIR)
            .join("versions")
            .join(entry_id.to_string())
            .join(format!("{version}.json"))
    }

    fn load_control_entry_records(&self) -> DriveStoreResult<Vec<EntryRecord>> {
        json_files(&self.control_path().join("index"))?
            .into_iter()
            .map(|path| read_json(&path))
            .collect()
    }

    fn load_all_entry_records(&self) -> DriveStoreResult<Vec<EntryRecord>> {
        let mut by_id = BTreeMap::new();
        for record in self.load_control_entry_records()? {
            by_id.insert(record.entry.id, record);
        }
        for mount in self.list_mounts()? {
            let entries_dir = self
                .pool_path(&mount.name)
                .join(POOL_INDEX_DIR)
                .join("entries");
            for path in json_files(&entries_dir)? {
                let record: EntryRecord = read_json(&path)?;
                by_id.entry(record.entry.id).or_insert(record);
            }
        }
        Ok(by_id.into_values().collect())
    }

    fn load_control_entry_record_by_id(&self, id: Uuid) -> DriveStoreResult<Option<EntryRecord>> {
        let path = self.entry_path(id);
        if path.exists() {
            return Ok(Some(read_json(&path)?));
        }
        Ok(None)
    }

    fn save_entry_record(&self, mut record: EntryRecord) -> DriveStoreResult<()> {
        record.entry.updated_at = Utc::now();
        write_json(&self.entry_path(record.entry.id), &record)?;
        if let Some(pool_name) = &record.pool_name {
            write_json(&self.pool_entry_path(pool_name, record.entry.id), &record)?;
            commit_and_push_repo(
                &self.pool_path(pool_name),
                "update aio drive pool index",
                "push_failed",
            )?;
        }
        Ok(())
    }

    fn load_version_records(&self, entry_id: Uuid) -> DriveStoreResult<Vec<VersionRecord>> {
        let mut versions = Vec::new();
        let control_dir = self
            .control_path()
            .join("versions")
            .join(entry_id.to_string());
        for path in json_files(&control_dir)? {
            versions.push(read_json(&path)?);
        }
        if versions.is_empty() {
            for mount in self.list_mounts()? {
                let dir = self
                    .pool_path(&mount.name)
                    .join(POOL_INDEX_DIR)
                    .join("versions")
                    .join(entry_id.to_string());
                for path in json_files(&dir)? {
                    versions.push(read_json(&path)?);
                }
            }
        }
        versions.sort_by_key(|record: &VersionRecord| record.version.version);
        Ok(versions)
    }

    fn select_writable_pool(&self, bytes_len: u64) -> DriveStoreResult<GitPoolRepoConfig> {
        let pools = self.list_pools()?;
        if let Some(pool) = pools
            .into_iter()
            .filter(|pool| !pool.readonly)
            .find(|pool| pool.used_bytes.saturating_add(bytes_len) <= pool.max_size_bytes)
        {
            return Ok(pool);
        }
        if let Some(pool) = self.try_auto_expand_pool(bytes_len)? {
            return Ok(pool);
        }
        Err(git_pool_error(
            "no_writable_pool_capacity",
            "没有可写 Git pool 或所有 pool 已超过容量阈值；请运行 aio drive pool add 增加仓库，或配置 auto_pool_root 自动扩容",
        ))
    }

    fn try_auto_expand_pool(&self, bytes_len: u64) -> DriveStoreResult<Option<GitPoolRepoConfig>> {
        let Some(root) = &self.config.auto_pool_root else {
            return Ok(None);
        };
        let name = self.next_auto_pool_name()?;
        let remote_url = root.join(format!("{name}.git"));
        let remote_url = remote_url.to_string_lossy().into_owned();
        let max_size_bytes = self.config.default_pool_limit_bytes.max(bytes_len);
        Ok(Some(self.init_pool(
            &name,
            &remote_url,
            Some(max_size_bytes),
        )?))
    }

    fn next_auto_pool_name(&self) -> DriveStoreResult<String> {
        let prefix = self.config.auto_pool_prefix.trim();
        let prefix = if prefix.is_empty() {
            DEFAULT_AUTO_GIT_POOL_PREFIX
        } else {
            prefix
        };
        for index in 1..=99_999_u32 {
            let name = format!("{prefix}-{index:04}");
            if self.pool_config_path(&name).exists() || self.mount_config_path(&name).exists() {
                continue;
            }
            validate_pool_name(&name)?;
            return Ok(name);
        }
        Err(git_pool_error(
            "init_failed",
            "自动扩容 pool 编号已耗尽，请调整 auto_pool_prefix 或清理旧 pool",
        ))
    }

    fn pool_containing_object(
        &self,
        object_key: &str,
        include_readonly: bool,
    ) -> DriveStoreResult<Option<String>> {
        for pool in self.list_pools()? {
            if !include_readonly && pool.readonly {
                continue;
            }
            if self.pool_object_path(&pool.name, object_key).exists() {
                return Ok(Some(pool.name));
            }
        }
        if include_readonly {
            for mount in self.list_mounts()? {
                if self.pool_object_path(&mount.name, object_key).exists() {
                    return Ok(Some(mount.name));
                }
            }
        }
        Ok(None)
    }

    fn pool_object_path(&self, pool_name: &str, object_key: &str) -> PathBuf {
        let suffix = object_key
            .strip_prefix("objects/sha256/")
            .unwrap_or(object_key)
            .trim_matches('/');
        let shard = suffix.get(0..2).unwrap_or("xx");
        self.pool_path(pool_name)
            .join("objects")
            .join("sha256")
            .join(shard)
            .join(suffix)
    }

    fn update_pool_used_bytes(&self, pool_name: &str, delta: u64) -> DriveStoreResult<()> {
        let path = self.pool_config_path(pool_name);
        let mut pool: GitPoolRepoConfig = read_json(&path)?;
        pool.used_bytes = pool.used_bytes.saturating_add(delta);
        write_json(&path, &pool)
    }

    fn prepare_sync_blocking(&self) -> DriveStoreResult<()> {
        self.ensure_layout()?;
        pull_repo(&self.control_path(), "control_pull_failed")?;
        for pool in self.list_pools()? {
            self.ensure_pool_worktree(&pool)?;
            pull_repo(&self.pool_path(&pool.name), "pool_pull_failed")?;
        }
        for mount in self.list_mounts()? {
            self.ensure_pool_checkout(&GitPoolRepoConfig {
                name: mount.name.clone(),
                remote_url: mount.remote_url.clone(),
                owner_drive_id: mount.owner_drive_id.clone(),
                readonly: true,
                max_size_bytes: 0,
                used_bytes: 0,
            })?;
            pull_repo(&self.pool_path(&mount.name), "pool_pull_failed")?;
        }
        Ok(())
    }

    fn flush_sync_blocking(&self) -> DriveStoreResult<()> {
        commit_and_push_repo(
            &self.control_path(),
            "update aio drive control",
            "push_failed",
        )?;
        for pool in self.list_pools()? {
            if !pool.readonly {
                commit_and_push_repo(
                    &self.pool_path(&pool.name),
                    "update aio drive pool",
                    "push_failed",
                )?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl DriveSyncCoordinator for GitPoolDriveStore {
    async fn prepare_sync(&self) -> DriveStoreResult<()> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || store.prepare_sync_blocking())
            .await
            .map_err(|err| git_pool_error("control_pull_failed", err.to_string()))?
    }

    async fn flush_sync(&self) -> DriveStoreResult<()> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || store.flush_sync_blocking())
            .await
            .map_err(|err| git_pool_error("push_failed", err.to_string()))?
    }
}

#[async_trait]
impl DriveMetadataStore for GitPoolDriveStore {
    async fn upsert_entry(
        &self,
        key: &EntryKey,
        kind: DriveEntryKind,
    ) -> DriveStoreResult<DriveEntry> {
        if let Some(record) = self
            .load_control_entry_records()?
            .into_iter()
            .find(|record| record.entry.key == *key)
        {
            return Ok(record.entry);
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
        self.save_entry_record(EntryRecord {
            entry: entry.clone(),
            pool_name: None,
        })?;
        Ok(entry)
    }

    async fn get_entry(&self, key: &EntryKey) -> DriveStoreResult<Option<DriveEntry>> {
        Ok(self
            .load_all_entry_records()?
            .into_iter()
            .find(|record| record.entry.key == *key)
            .map(|record| record.entry))
    }

    async fn get_entry_by_id(&self, id: Uuid) -> DriveStoreResult<Option<DriveEntry>> {
        if let Some(record) = self.load_control_entry_record_by_id(id)? {
            return Ok(Some(record.entry));
        }
        Ok(self
            .load_all_entry_records()?
            .into_iter()
            .find(|record| record.entry.id == id)
            .map(|record| record.entry))
    }

    async fn list_entries(
        &self,
        space_id: &str,
        root_alias: &RootAlias,
        prefix: &RelativePath,
    ) -> DriveStoreResult<Vec<DriveEntry>> {
        let prefix_text = prefix.as_str();
        let mut entries = self
            .load_all_entry_records()?
            .into_iter()
            .map(|record| record.entry)
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
            .collect::<Vec<_>>();
        entries.sort_by(entry_order);
        Ok(entries)
    }

    async fn list_entries_by_space(&self, space_id: &str) -> DriveStoreResult<Vec<DriveEntry>> {
        let mut entries = self
            .load_all_entry_records()?
            .into_iter()
            .map(|record| record.entry)
            .filter(|entry| entry.key.space_id == space_id && !entry.deleted)
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
        let mut count = 0;
        for mut record in self.load_control_entry_records()? {
            if record.entry.key.space_id == from_owner_drive_id {
                record.entry.key.space_id = to_owner_drive_id.to_owned();
                record.entry.updated_at = Utc::now();
                self.save_entry_record(record)?;
                count += 1;
            }
        }
        for path in json_files(&self.control_path().join("ignored"))? {
            let mut record: IgnoredRecord = read_json(&path)?;
            if record.ignored.space_id == from_owner_drive_id {
                record.ignored.space_id = to_owner_drive_id.to_owned();
                record.ignored.updated_at = Utc::now();
                write_json(&path, &record)?;
                count += 1;
            }
        }
        Ok(count)
    }

    async fn upsert_ignored_path(
        &self,
        key: &EntryKey,
        source_device_id: &str,
    ) -> DriveStoreResult<DriveIgnoredPath> {
        for path in json_files(&self.control_path().join("ignored"))? {
            let mut record: IgnoredRecord = read_json(&path)?;
            if record.ignored.space_id == key.space_id
                && record.ignored.root_alias == key.root_alias
                && record.ignored.relative_path == key.relative_path
            {
                record.ignored.source_device_id = source_device_id.to_owned();
                record.ignored.updated_at = Utc::now();
                write_json(&path, &record)?;
                write_aioignore(
                    &self.control_path(),
                    &self.list_ignored_paths(&key.space_id, None, None).await?,
                )?;
                return Ok(record.ignored);
            }
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
        write_json(
            &self.ignored_path(ignored.id),
            &IgnoredRecord {
                ignored: ignored.clone(),
            },
        )?;
        write_aioignore(
            &self.control_path(),
            &self.list_ignored_paths(&key.space_id, None, None).await?,
        )?;
        Ok(ignored)
    }

    async fn delete_ignored_path(&self, key: &EntryKey) -> DriveStoreResult<()> {
        for path in json_files(&self.control_path().join("ignored"))? {
            let record: IgnoredRecord = read_json(&path)?;
            if record.ignored.space_id == key.space_id
                && record.ignored.root_alias == key.root_alias
                && record.ignored.relative_path == key.relative_path
            {
                remove_file_if_exists(&path)?;
            }
        }
        write_aioignore(
            &self.control_path(),
            &self.list_ignored_paths(&key.space_id, None, None).await?,
        )
    }

    async fn list_ignored_paths(
        &self,
        space_id: &str,
        root_alias: Option<&RootAlias>,
        prefix: Option<&RelativePath>,
    ) -> DriveStoreResult<Vec<DriveIgnoredPath>> {
        let mut ignored = Vec::new();
        for path in json_files(&self.control_path().join("ignored"))? {
            let record: IgnoredRecord = read_json(&path)?;
            let row = record.ignored;
            if row.space_id == space_id
                && root_alias.is_none_or(|alias| row.root_alias == *alias)
                && prefix.is_none_or(|prefix| {
                    prefix.is_root()
                        || row.relative_path == *prefix
                        || row
                            .relative_path
                            .as_str()
                            .starts_with(&format!("{}/", prefix.as_str()))
                })
            {
                ignored.push(row);
            }
        }
        ignored.sort_by(ignored_order);
        Ok(ignored)
    }

    async fn delete_entry(&self, key: &EntryKey) -> DriveStoreResult<()> {
        for mut record in self.load_control_entry_records()? {
            if record.entry.key == *key {
                record.entry.deleted = true;
                record.entry.updated_at = Utc::now();
                self.save_entry_record(record)?;
            }
        }
        Ok(())
    }

    async fn insert_version(&self, version: DriveVersion) -> DriveStoreResult<DriveVersion> {
        let mut record = self
            .load_control_entry_record_by_id(version.entry_id)?
            .ok_or_else(|| DriveStoreError::EntryNotFound(version.entry_id.to_string()))?;
        let pool_name = self.pool_containing_object(&version.object_key, false)?;
        record.pool_name = pool_name.clone();
        record.entry.latest_version = version.version;
        record.entry.latest_hash = Some(version.content_hash.clone());
        record.entry.deleted = false;
        record.entry.updated_at = Utc::now();
        let version_record = VersionRecord {
            version: version.clone(),
        };
        write_json(
            &self.version_path(version.entry_id, version.version),
            &version_record,
        )?;
        if let Some(pool_name) = pool_name {
            write_json(
                &self.pool_version_path(&pool_name, version.entry_id, version.version),
                &version_record,
            )?;
        }
        self.save_entry_record(record)?;
        Ok(version)
    }

    async fn latest_version(&self, entry_id: Uuid) -> DriveStoreResult<Option<DriveVersion>> {
        Ok(self
            .load_version_records(entry_id)?
            .into_iter()
            .max_by_key(|record| record.version.version)
            .map(|record| record.version))
    }

    async fn record_conflict(&self, conflict: DriveConflict) -> DriveStoreResult<DriveConflict> {
        write_json(
            &self.conflict_path(conflict.id),
            &ConflictRecord {
                conflict: conflict.clone(),
            },
        )?;
        Ok(conflict)
    }

    async fn list_conflicts(&self, resolved: Option<bool>) -> DriveStoreResult<Vec<DriveConflict>> {
        let mut conflicts = Vec::new();
        for path in json_files(&self.control_path().join("conflicts"))? {
            let record: ConflictRecord = read_json(&path)?;
            if resolved.is_none_or(|value| record.conflict.resolved == value) {
                conflicts.push(record.conflict);
            }
        }
        conflicts.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(conflicts)
    }

    async fn resolve_conflict(&self, conflict_id: Uuid) -> DriveStoreResult<Option<DriveConflict>> {
        let path = self.conflict_path(conflict_id);
        if !path.exists() {
            return Ok(None);
        }
        let mut record: ConflictRecord = read_json(&path)?;
        record.conflict.resolved = true;
        write_json(&path, &record)?;
        Ok(Some(record.conflict))
    }

    async fn enqueue_sync_task(
        &self,
        item: DriveSyncQueueItem,
    ) -> DriveStoreResult<DriveSyncQueueItem> {
        write_json(
            &self.sync_queue_path(item.id),
            &SyncQueueRecord { item: item.clone() },
        )?;
        Ok(item)
    }

    async fn update_sync_task(
        &self,
        id: Uuid,
        status: DriveSyncTaskStatus,
        last_error: Option<&str>,
    ) -> DriveStoreResult<Option<DriveSyncQueueItem>> {
        let path = self.sync_queue_path(id);
        if !path.exists() {
            return Ok(None);
        }
        let mut record: SyncQueueRecord = read_json(&path)?;
        record.item.status = status;
        record.item.updated_at = Utc::now();
        record.item.last_error = last_error.map(str::to_owned);
        if matches!(status, DriveSyncTaskStatus::Running) {
            record.item.attempts = record.item.attempts.saturating_add(1);
        }
        write_json(&path, &record)?;
        Ok(Some(record.item))
    }

    async fn list_sync_queue(
        &self,
        status: Option<DriveSyncTaskStatus>,
    ) -> DriveStoreResult<Vec<DriveSyncQueueItem>> {
        let mut items = Vec::new();
        for path in json_files(&self.control_path().join("sync-queue"))? {
            let record: SyncQueueRecord = read_json(&path)?;
            if status.is_none_or(|status| record.item.status == status) {
                items.push(record.item);
            }
        }
        items.sort_by(|left, right| left.created_at.cmp(&right.created_at));
        Ok(items)
    }

    async fn retry_failed_sync_tasks(&self) -> DriveStoreResult<u64> {
        let mut count = 0;
        for path in json_files(&self.control_path().join("sync-queue"))? {
            let mut record: SyncQueueRecord = read_json(&path)?;
            if record.item.status == DriveSyncTaskStatus::Failed {
                record.item.status = DriveSyncTaskStatus::Pending;
                record.item.last_error = None;
                record.item.updated_at = Utc::now();
                write_json(&path, &record)?;
                count += 1;
            }
        }
        Ok(count)
    }

    async fn upsert_suspended_path(
        &self,
        suspension: DriveSuspendedPath,
    ) -> DriveStoreResult<DriveSuspendedPath> {
        write_json(
            &self.suspended_path(suspension.entry_id),
            &SuspendedRecord {
                suspension: suspension.clone(),
            },
        )?;
        Ok(suspension)
    }

    async fn get_suspended_path(
        &self,
        entry_id: Uuid,
    ) -> DriveStoreResult<Option<DriveSuspendedPath>> {
        let path = self.suspended_path(entry_id);
        if !path.exists() {
            return Ok(None);
        }
        let record: SuspendedRecord = read_json(&path)?;
        Ok(Some(record.suspension))
    }

    async fn list_suspended_paths(&self) -> DriveStoreResult<Vec<DriveSuspendedPath>> {
        let mut items = Vec::new();
        for path in json_files(&self.control_path().join("suspended"))? {
            let record: SuspendedRecord = read_json(&path)?;
            items.push(record.suspension);
        }
        items.sort_by(|left, right| {
            left.space_id
                .cmp(&right.space_id)
                .then_with(|| left.root_alias.cmp(&right.root_alias))
                .then_with(|| left.relative_path.cmp(&right.relative_path))
        });
        Ok(items)
    }

    async fn delete_suspended_path(&self, entry_id: Uuid) -> DriveStoreResult<bool> {
        let path = self.suspended_path(entry_id);
        let existed = path.exists();
        remove_file_if_exists(&path)?;
        Ok(existed)
    }

    async fn acquire_lock(&self, lock: DriveLock) -> DriveStoreResult<DriveLock> {
        Ok(lock)
    }

    async fn release_lock(&self, _entry_id: Uuid, _token: &str) -> DriveStoreResult<bool> {
        Ok(true)
    }

    async fn get_lock(&self, _entry_id: Uuid) -> DriveStoreResult<Option<DriveLock>> {
        Ok(None)
    }
}

#[async_trait]
impl DriveObjectStore for GitPoolDriveStore {
    async fn put_object(&self, object_key: &str, bytes: &[u8]) -> DriveStoreResult<()> {
        if self.pool_containing_object(object_key, false)?.is_some() {
            return Ok(());
        }
        let pool = self.select_writable_pool(bytes.len() as u64)?;
        let path = self.pool_object_path(&pool.name, object_key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| io_error("init_failed", err))?;
        }
        fs::write(&path, bytes).map_err(|err| io_error("object_write_failed", err))?;
        self.update_pool_used_bytes(&pool.name, bytes.len() as u64)?;
        commit_and_push_repo(
            &self.pool_path(&pool.name),
            "store aio drive object",
            "push_failed",
        )?;
        Ok(())
    }

    async fn get_object(&self, object_key: &str) -> DriveStoreResult<Vec<u8>> {
        for pool in self.list_pools()? {
            let path = self.pool_object_path(&pool.name, object_key);
            if path.exists() {
                return fs::read(&path).map_err(|err| io_error("object_read_failed", err));
            }
        }
        for mount in self.list_mounts()? {
            let path = self.pool_object_path(&mount.name, object_key);
            if path.exists() {
                return fs::read(&path).map_err(|err| io_error("object_read_failed", err));
            }
        }
        Err(DriveStoreError::ObjectNotFound(object_key.to_owned()))
    }

    async fn delete_object(&self, object_key: &str) -> DriveStoreResult<()> {
        for pool in self.list_pools()? {
            if pool.readonly {
                continue;
            }
            remove_file_if_exists(&self.pool_object_path(&pool.name, object_key))?;
        }
        Ok(())
    }

    async fn object_exists(&self, object_key: &str) -> DriveStoreResult<bool> {
        Ok(self.pool_containing_object(object_key, true)?.is_some())
    }
}

fn ensure_worktree(
    path: &Path,
    remote: Option<&str>,
    pull_phase: &'static str,
) -> DriveStoreResult<()> {
    if let Some(remote) = remote.filter(|remote| !remote.trim().is_empty()) {
        ensure_local_bare_remote(remote)?;
    }
    if !path.join(".git").exists() {
        if let Some(remote) = remote.filter(|remote| !remote.trim().is_empty()) {
            if path.exists()
                && path
                    .read_dir()
                    .map_err(|err| io_error("init_failed", err))?
                    .next()
                    .is_none()
            {
                fs::remove_dir_all(path).map_err(|err| io_error("init_failed", err))?;
            }
            if !path.exists() {
                run_git_clone(remote, path, pull_phase)?;
            } else {
                run_git(path, ["init"], "init_failed")?;
                ensure_remote(path, remote)?;
            }
        } else {
            fs::create_dir_all(path).map_err(|err| io_error("init_failed", err))?;
            run_git(path, ["init"], "init_failed")?;
        }
    }
    configure_repo_identity(path)?;
    if let Some(remote) = remote.filter(|remote| !remote.trim().is_empty()) {
        ensure_remote(path, remote)?;
        pull_repo(path, pull_phase)?;
    }
    Ok(())
}

fn ensure_remote(path: &Path, remote: &str) -> DriveStoreResult<()> {
    if run_git_allow_failure(path, ["remote", "get-url", "origin"]).is_ok() {
        run_git(path, ["remote", "set-url", "origin", remote], "init_failed")
    } else {
        run_git(path, ["remote", "add", "origin", remote], "init_failed")
    }
}

fn configure_repo_identity(path: &Path) -> DriveStoreResult<()> {
    if run_git_allow_failure(path, ["config", "--get", "user.email"]).is_err() {
        run_git(
            path,
            ["config", "user.email", "aio-drive@local"],
            "init_failed",
        )?;
    }
    if run_git_allow_failure(path, ["config", "--get", "user.name"]).is_err() {
        run_git(path, ["config", "user.name", "AIO Drive"], "init_failed")?;
    }
    Ok(())
}

fn ensure_local_bare_remote(remote: &str) -> DriveStoreResult<()> {
    let path = expand_local_git_url(remote);
    let Some(path) = path else {
        return Ok(());
    };
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| io_error("init_failed", err))?;
    }
    let output = Command::new("git")
        .args(["init", "--bare"])
        .arg(&path)
        .output()
        .map_err(|err| git_pool_error("git_missing", err.to_string()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(git_command_error("init_failed", output))
    }
}

fn expand_local_git_url(remote: &str) -> Option<PathBuf> {
    if remote.contains("://")
        || remote.starts_with("git@")
        || remote.contains(':') && !remote.starts_with('/')
    {
        return None;
    }
    Some(expand_home_path(remote))
}

fn pull_repo(path: &Path, phase: &'static str) -> DriveStoreResult<()> {
    if run_git_allow_failure(path, ["remote", "get-url", "origin"]).is_err() {
        return Ok(());
    }
    let fetch = run_git_allow_failure(path, ["fetch", "origin"]);
    if let Err(err) = fetch {
        if err.to_string().contains("couldn't find remote ref")
            || err
                .to_string()
                .contains("does not appear to be a git repository")
        {
            return Ok(());
        }
        return Err(git_pool_error(phase, err.to_string()));
    }
    let branch = current_branch(path).unwrap_or_else(|| "main".to_owned());
    let upstream = format!("origin/{branch}");
    if run_git_allow_failure(path, ["rev-parse", "--verify", upstream.as_str()]).is_ok() {
        run_git(path, ["rebase", upstream.as_str()], phase)?;
    }
    Ok(())
}

fn commit_and_push_repo(
    path: &Path,
    message: &str,
    push_phase: &'static str,
) -> DriveStoreResult<()> {
    if !path.join(".git").exists() {
        return Ok(());
    }
    if worktree_clean(path)? {
        return Ok(());
    }
    run_git(path, ["add", "-A"], "commit_failed")?;
    let commit = run_git_allow_failure(path, ["commit", "-m", message]);
    if let Err(err) = commit
        && !err.to_string().contains("nothing to commit")
    {
        return Err(git_pool_error("commit_failed", err.to_string()));
    }
    if run_git_allow_failure(path, ["remote", "get-url", "origin"]).is_ok() {
        let branch = current_branch(path).unwrap_or_else(|| "main".to_owned());
        let push = run_git_allow_failure(path, ["push", "-u", "origin", branch.as_str()]);
        if let Err(err) = push {
            return Err(git_pool_error(push_phase, err.to_string()));
        }
    }
    Ok(())
}

fn worktree_clean(path: &Path) -> DriveStoreResult<bool> {
    let output = run_git_capture(path, ["status", "--porcelain"], "dirty_pool_state")?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

fn current_branch(path: &Path) -> Option<String> {
    let output = run_git_capture(path, ["branch", "--show-current"], "dirty_pool_state").ok()?;
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!branch.is_empty()).then_some(branch)
}

fn run_git<const N: usize>(
    cwd: &Path,
    args: [&str; N],
    phase: &'static str,
) -> DriveStoreResult<()> {
    let output = run_git_capture(cwd, args, phase)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(git_command_error(phase, output))
    }
}

fn run_git_clone(remote: &str, path: &Path, phase: &'static str) -> DriveStoreResult<()> {
    let output = Command::new("git")
        .arg("clone")
        .arg(remote)
        .arg(path)
        .output()
        .map_err(|err| git_pool_error("git_missing", err.to_string()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(git_command_error(phase, output))
    }
}

fn run_git_allow_failure<const N: usize>(
    cwd: &Path,
    args: [&str; N],
) -> DriveStoreResult<std::process::Output> {
    let output = run_git_capture(cwd, args, "dirty_pool_state")?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(git_command_error("dirty_pool_state", output))
    }
}

fn run_git_capture<const N: usize>(
    cwd: &Path,
    args: [&str; N],
    phase: &'static str,
) -> DriveStoreResult<std::process::Output> {
    Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|err| git_pool_error("git_missing", format!("{phase}: {err}")))
}

fn git_command_error(phase: &'static str, output: std::process::Output) -> DriveStoreError {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let message = if stderr.is_empty() { stdout } else { stderr };
    git_pool_error(phase, message)
}

fn git_pool_error(phase: &'static str, message: impl Into<String>) -> DriveStoreError {
    DriveStoreError::GitPool {
        phase,
        message: message.into(),
    }
}

fn io_error(phase: &'static str, error: io::Error) -> DriveStoreError {
    git_pool_error(phase, error.to_string())
}

fn read_json<T: DeserializeOwned>(path: &Path) -> DriveStoreResult<T> {
    let raw = fs::read_to_string(path).map_err(|err| io_error("object_read_failed", err))?;
    serde_json::from_str(&raw).map_err(|err| git_pool_error("object_read_failed", err.to_string()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> DriveStoreResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| io_error("init_failed", err))?;
    }
    let raw = serde_json::to_vec_pretty(value)
        .map_err(|err| git_pool_error("object_write_failed", err.to_string()))?;
    fs::write(path, raw).map_err(|err| io_error("object_write_failed", err))?;
    Ok(())
}

fn json_files(dir: &Path) -> DriveStoreResult<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).map_err(|err| io_error("object_read_failed", err))? {
        let entry = entry.map_err(|err| io_error("object_read_failed", err))?;
        let path = entry.path();
        if path.extension() == Some(OsStr::new("json")) {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn remove_file_if_exists(path: &Path) -> DriveStoreResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(io_error("object_write_failed", err)),
    }
}

fn write_aioignore(control: &Path, ignored: &[DriveIgnoredPath]) -> DriveStoreResult<()> {
    let mut lines = ignored
        .iter()
        .map(|row| {
            if row.relative_path.is_root() {
                format!("{}/{}", row.space_id, row.root_alias)
            } else {
                format!("{}/{}/{}", row.space_id, row.root_alias, row.relative_path)
            }
        })
        .collect::<Vec<_>>();
    lines.sort();
    fs::write(control.join(".aioignore"), lines.join("\n"))
        .map_err(|err| io_error("object_write_failed", err))
}

fn validate_pool_name(name: &str) -> DriveStoreResult<()> {
    if name.trim().is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(git_pool_error(
            "init_failed",
            "pool 名称只能包含字母、数字、`-`、`_` 和 `.`",
        ));
    }
    Ok(())
}

fn expand_home_path(raw: &str) -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        if raw == "~" {
            return PathBuf::from(home);
        }
        if let Some(rest) = raw.strip_prefix("~/") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(raw)
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

fn default_pool_limit_bytes() -> u64 {
    DEFAULT_GIT_POOL_LIMIT_BYTES
}

fn default_auto_pool_prefix() -> String {
    DEFAULT_AUTO_GIT_POOL_PREFIX.to_owned()
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_AUTO_GIT_POOL_PREFIX, GitPoolConfig, GitPoolDriveStore};
    use crate::{DriveEntryKind, DriveMetadataStore, DriveObjectStore, DriveVersion};
    use az_drive_core::{EntryKey, RelativePath, RootAlias, content_hash, object_key_for_hash};
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn git_pool_store_round_trips_metadata_and_object()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let store = GitPoolDriveStore::open(GitPoolConfig::new(
            temp.path().join("drive"),
            "user-zjarlin",
        ))?;
        let pool_remote = temp.path().join("pool.git");
        store.init_pool("main", &pool_remote.to_string_lossy(), Some(1024 * 1024))?;

        let key = EntryKey::new(
            "user-zjarlin",
            RootAlias::parse("home")?,
            RelativePath::parse(".agents/skills/demo/SKILL.md")?,
        );
        let entry = store.upsert_entry(&key, DriveEntryKind::File).await?;
        let bytes = b"hello";
        let hash = content_hash(bytes);
        let object_key = object_key_for_hash(&hash);
        store.put_object(&object_key, bytes).await?;
        store
            .insert_version(DriveVersion {
                id: Uuid::new_v4(),
                entry_id: entry.id,
                version: 1,
                content_hash: hash.clone(),
                object_key: object_key.clone(),
                size_bytes: bytes.len() as u64,
                device_id: "device-a".to_owned(),
                modified_at: Utc::now(),
            })
            .await?;

        let reopened = GitPoolDriveStore::open(GitPoolConfig::new(
            temp.path().join("drive"),
            "user-zjarlin",
        ))?;
        let latest = reopened
            .latest_version(entry.id)
            .await?
            .expect("latest version should exist");
        assert_eq!(latest.content_hash, hash);
        assert_eq!(reopened.get_object(&object_key).await?, bytes);
        Ok(())
    }

    #[tokio::test]
    async fn git_pool_store_reports_capacity_when_no_writable_pool_fits()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let store = GitPoolDriveStore::open(GitPoolConfig::new(
            temp.path().join("drive"),
            "user-zjarlin",
        ))?;
        let pool_remote = temp.path().join("pool.git");
        store.init_pool("main", &pool_remote.to_string_lossy(), Some(1))?;

        let result = store.put_object("objects/sha256/demo", b"too large").await;

        let error = result.expect_err("oversized object should fail");
        assert!(error.to_string().contains("no_writable_pool_capacity"));
        Ok(())
    }

    #[tokio::test]
    async fn git_pool_metadata_can_record_versions_without_content_pool()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let store = GitPoolDriveStore::open(GitPoolConfig::new(
            temp.path().join("drive"),
            "user-zjarlin",
        ))?;
        let key = EntryKey::new(
            "user-zjarlin",
            RootAlias::parse("home")?,
            RelativePath::parse(".agents/skills/demo/SKILL.md")?,
        );
        let entry = store.upsert_entry(&key, DriveEntryKind::File).await?;

        store
            .insert_version(DriveVersion {
                id: Uuid::new_v4(),
                entry_id: entry.id,
                version: 1,
                content_hash: "hash-demo".to_owned(),
                object_key: "objects/sha256/demo".to_owned(),
                size_bytes: 4,
                device_id: "device-a".to_owned(),
                modified_at: Utc::now(),
            })
            .await?;

        let latest = store
            .latest_version(entry.id)
            .await?
            .expect("latest version should exist");
        assert_eq!(latest.object_key, "objects/sha256/demo");
        Ok(())
    }

    #[tokio::test]
    async fn git_pool_store_auto_expands_when_capacity_is_exhausted()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let mut config = GitPoolConfig::new(temp.path().join("drive"), "user-zjarlin");
        config.auto_pool_root = Some(temp.path().join("auto-pools"));
        config.default_pool_limit_bytes = 4;
        let store = GitPoolDriveStore::open(config)?;
        let pool_remote = temp.path().join("pool.git");
        store.init_pool("main", &pool_remote.to_string_lossy(), Some(1))?;

        store
            .put_object("objects/sha256/demo", b"too large")
            .await?;

        let pools = store.list_pools()?;
        assert_eq!(pools.len(), 2);
        let pool = pools
            .iter()
            .find(|pool| pool.name == format!("{DEFAULT_AUTO_GIT_POOL_PREFIX}-0001"))
            .expect("auto-created pool should exist");
        assert_eq!(pool.used_bytes, "too large".len() as u64);
        assert_eq!(pool.max_size_bytes, "too large".len() as u64);
        assert!(
            temp.path()
                .join("auto-pools")
                .join(format!("{DEFAULT_AUTO_GIT_POOL_PREFIX}-0001.git"))
                .exists()
        );
        Ok(())
    }
}
