//! 无头实时网盘代理。
//!
//! 该守护进程在设计上不依赖 GUI。它通过轮询循环监听托管路径，
//! 对本地与远程版本进行对账，并自动写入冲突副本，
//! 无需手动的 Git 式人工干预。

use anyhow::Context;
use az_drive_core::api::{
    ChangeDecision, EntryKey, HostPathMapping, RelativePath, RootAlias, RootRegistry, conflict_file_name,
    content_hash, decide_local_change, expand_path_expression, normalize_absolute_path,
    object_key_for_hash, try_safe_text_merge,
};
use az_drive_store::api::{
    DriveConflict, DriveEntry, DriveEntryKind, DriveIgnoredPath, DriveMetadataStore,
    DriveObjectStore, DriveSuspendedPath, DriveSyncCoordinator, DriveSyncQueueItem,
    DriveSyncTaskKind, DriveSyncTaskStatus, DriveVersion, NoopDriveSyncCoordinator,
};
use chrono::Utc;
use ignore::WalkBuilder;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::local_state::{
    HostedPathState, HostedRootState, LocalConflictState, LocalRootState, LocalState,
    LocalStateStore,
};

/// Agent configuration that is stable across CLI and future AIO embedding.
#[derive(Clone)]
pub struct DriveAgentConfig {
    /// Primary owner Drive namespace.
    pub space_id: String,
    /// Additional owner Drive namespaces visible for read-side fusion.
    pub fused_space_ids: Vec<String>,
    /// Owner Drive namespaces whose remote-only entries should be materialized during sync.
    pub auto_materialize_space_ids: Vec<String>,
    /// Local device id.
    pub device_id: String,
    /// Human-readable device name used in conflict copies.
    pub device_name: String,
    /// Poll interval for the daemon loop.
    pub poll_interval: Duration,
    /// Optional callback invoked after each file content change is synced.
    /// Receives the `remote_path` (root-relative) of the changed file.
    pub on_file_synced: Option<Arc<dyn Fn(String) + Send + Sync>>,
}


impl std::fmt::Debug for DriveAgentConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DriveAgentConfig")
            .field("space_id", &self.space_id)
            .field("fused_space_ids", &self.fused_space_ids)
            .field("auto_materialize_space_ids", &self.auto_materialize_space_ids)
            .field("device_id", &self.device_id)
            .field("device_name", &self.device_name)
            .field("poll_interval", &self.poll_interval)
            .field("on_file_synced", &self.on_file_synced.as_ref().map(|_| ".."))
            .finish()
    }
}

impl DriveAgentConfig {
    /// Creates a config with stable defaults.
    #[must_use]
    pub fn new(space_id: impl Into<String>, device_id: String, device_name: String) -> Self {
        let space_id = space_id.into();
        Self {
            space_id,
            fused_space_ids: Vec::new(),
            auto_materialize_space_ids: Vec::new(),
            device_id,
            device_name,
            poll_interval: Duration::from_secs(2),
            on_file_synced: None,
        }
    }

    /// Adds read-side fused owner Drives.
    #[must_use]
    pub fn with_fused_space_ids(mut self, spaces: impl IntoIterator<Item = String>) -> Self {
        for space in spaces {
            if !space.trim().is_empty()
                && space != self.space_id
                && !self.fused_space_ids.contains(&space)
            {
                self.fused_space_ids.push(space);
            }
        }
        self
    }

    /// Adds owner Drives whose remote-only entries are automatically materialized
    /// during normal bidirectional sync.
    #[must_use]
    pub fn with_auto_materialize_space_ids(
        mut self,
        spaces: impl IntoIterator<Item = String>,
    ) -> Self {
        for space in spaces {
            if !space.trim().is_empty() && !self.auto_materialize_space_ids.contains(&space) {
                self.auto_materialize_space_ids.push(space);
            }
        }
        self
    }

    /// Sets a callback invoked after each file content change is synced.
    #[must_use]
    pub fn with_on_file_synced(
        mut self,
        f: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        self.on_file_synced = Some(Arc::new(f));
        self
    }

    fn visible_space_ids(&self) -> Vec<String> {
        let mut spaces = vec![self.space_id.clone()];
        for space in &self.fused_space_ids {
            if !spaces.contains(space) {
                spaces.push(space.clone());
            }
        }
        spaces
    }
}

/// Summary for CLI status output.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HostedStatus {
    /// Owner Drive id.
    pub owner_drive_id: String,
    /// Local path.
    pub local_path: PathBuf,
    /// Remote key.
    pub remote_path: String,
    /// Last synchronized version.
    pub base_version: Option<u64>,
    /// Last local hash.
    pub content_hash: Option<String>,
    /// Whether the local file currently exists.
    pub exists: bool,
}

/// Status category for `drive ls` output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TrackedItemStatus {
    /// Local item is tracked and currently exists.
    Tracked,
    /// Local item is tracked but missing on disk.
    MissingLocal,
    /// Remote item has no local tracked counterpart on this device.
    RemoteOnly,
    /// Item is excluded by an ignore rule.
    Ignored,
    /// Automatic sync is suspended until an unresolved conflict is resolved.
    ConflictSuspended,
    /// Hosted directory root.
    Root,
}

impl TrackedItemStatus {
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

/// Provenance for a tracked listing row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TrackedItemSource {
    /// Device-local state or scan.
    Local,
    /// Server-side metadata.
    Remote,
    /// Present both locally and remotely.
    Both,
    /// Shared database ignore rule.
    DbIgnore,
    /// `.gitignore` or global git ignore rule.
    Gitignore,
    /// Conflict suspension metadata.
    Suspended,
}

impl TrackedItemSource {
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

/// Options for listing tracked drive paths.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ListTrackedOptions {
    /// Include remote metadata entries.
    pub include_remote: bool,
    /// Include and merge local and remote entries.
    pub include_all: bool,
    /// Return hosted directory roots instead of file entries.
    pub roots_only: bool,
    /// Include excluded paths in the output.
    pub include_ignored: bool,
}

/// Unified row returned by `drive ls`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TrackedItem {
    /// Listing status.
    pub status: TrackedItemStatus,
    /// Listing source.
    pub source: TrackedItemSource,
    /// Device-local path when one can be resolved.
    pub local_path: Option<PathBuf>,
    /// User-facing path with `$HOME` when applicable.
    pub display_path: String,
    /// Cross-device canonical path, for example `home/.agents/skills/foo`.
    pub canonical_path: String,
    /// Full remote key, for example `main/home/.agents/skills/foo`.
    pub remote_path: String,
    /// Owner Drive id.
    pub owner_drive_id: String,
    /// Remote root alias.
    pub root_alias: String,
    /// Remote relative path below the root alias.
    pub relative_path: String,
    /// Last synchronized or remote version.
    pub base_version: Option<u64>,
    /// Last local or remote content hash.
    pub content_hash: Option<String>,
    /// Whether the local path currently exists.
    pub exists: bool,
}

/// Result status for materializing remote entries onto the current device.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PullRemoteStatus {
    /// Remote bytes were written locally and the file is now hosted.
    Pulled,
    /// Local file already matched the remote version and was marked hosted.
    AlreadyCurrent,
    /// A local file exists with different content and `overwrite` was not set.
    SkippedExisting,
    /// The row is ignored by a shared database ignore rule.
    SkippedIgnored,
    /// The row is suspended by an unresolved conflict.
    SkippedSuspended,
    /// The row has no downloadable remote version.
    SkippedNoVersion,
    /// The command only reported what would happen.
    DryRun,
}

impl PullRemoteStatus {
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

/// Options for pulling remote entries onto the current device.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PullRemoteOptions {
    /// Overwrite a conflicting local file with the remote version.
    pub overwrite: bool,
    /// Report target paths without writing files or local state.
    pub dry_run: bool,
}

/// Result row returned by remote materialization.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PullRemoteItem {
    /// Pull result status.
    pub status: PullRemoteStatus,
    /// Device-local path selected for the remote key.
    pub local_path: PathBuf,
    /// User-facing path with `$HOME` when applicable.
    pub display_path: String,
    /// Cross-device canonical path.
    pub canonical_path: String,
    /// Full remote key.
    pub remote_path: String,
    /// Owner Drive id.
    pub owner_drive_id: String,
    /// Last remote version.
    pub base_version: Option<u64>,
    /// Last remote content hash.
    pub content_hash: Option<String>,
    /// Whether the local file exists after the operation.
    pub exists: bool,
}

/// Conflict resolution action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConflictResolution {
    /// Keep the current remote version and only clear the suspension.
    KeepRemote,
    /// Restore the local conflict copy and upload it on the next sync.
    KeepLocal,
    /// Use a user-provided merged file and upload it on the next sync.
    UseMerged(PathBuf),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RemoteMaterializeMode {
    All,
    UntrackedOnly,
}

struct ConflictRestoreRequest<'a> {
    local_path: &'a Path,
    entry_id: Uuid,
    base_version: Option<u64>,
    local_hash: &'a str,
    remote: &'a DriveVersion,
    local_bytes: &'a [u8],
}

/// Headless realtime drive agent.
#[derive(Clone)]
pub struct DriveAgent {
    metadata: Arc<dyn DriveMetadataStore>,
    objects: Arc<dyn DriveObjectStore>,
    sync: Arc<dyn DriveSyncCoordinator>,
    state_store: LocalStateStore,
    config: DriveAgentConfig,
}

impl DriveAgent {
    /// Creates a new drive agent.
    #[must_use]
    pub fn new(
        metadata: Arc<dyn DriveMetadataStore>,
        objects: Arc<dyn DriveObjectStore>,
        state_store: LocalStateStore,
        config: DriveAgentConfig,
    ) -> Self {
        Self {
            metadata,
            objects,
            sync: Arc::new(NoopDriveSyncCoordinator),
            state_store,
            config,
        }
    }

    /// Creates a new drive agent with an explicit store synchronization
    /// coordinator.
    #[must_use]
    pub fn new_with_sync(
        metadata: Arc<dyn DriveMetadataStore>,
        objects: Arc<dyn DriveObjectStore>,
        sync: Arc<dyn DriveSyncCoordinator>,
        state_store: LocalStateStore,
        config: DriveAgentConfig,
    ) -> Self {
        Self {
            metadata,
            objects,
            sync,
            state_store,
            config,
        }
    }

    /// Loads local state.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when state loading fails.
    pub async fn state(&self) -> anyhow::Result<LocalState> {
        self.state_store.load_or_init().await
    }

    /// Adds a local root alias.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when the alias/path is invalid or state save fails.
    pub async fn add_root(&self, alias: &str, path: &str) -> anyhow::Result<Vec<LocalRootState>> {
        let _state_lock = self.state_store.acquire_write_lock()?;
        let alias = RootAlias::parse(alias)?;
        let path = normalize_absolute_path(&expand_path_expression(path))?;
        let mut state = self.state_store.load_or_init().await?;
        state.roots.retain(|root| root.alias != alias.as_str());
        state.roots.push(LocalRootState {
            alias: alias.to_string(),
            path,
        });
        state
            .roots
            .sort_by(|left, right| left.alias.cmp(&right.alias));
        self.state_store.save(&state).await?;
        Ok(state.roots)
    }

    /// Lists roots, including the implicit `home` root when absent from state.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when local state or home root resolution fails.
    pub async fn list_roots(&self) -> anyhow::Result<Vec<LocalRootState>> {
        let state = self.state_store.load_or_init().await?;
        let registry = registry_from_state(&state)?;
        Ok(registry
            .list_roots()
            .into_iter()
            .map(|root| LocalRootState {
                alias: root.alias.to_string(),
                path: root.local_path,
            })
            .collect())
    }

    /// Hosts a file or directory for realtime synchronization.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when path mapping, local I/O, or remote store
    /// operations fail.
    pub async fn host_path(
        &self,
        path: &str,
        root_alias: Option<&str>,
        remote_path: Option<&str>,
    ) -> anyhow::Result<Vec<HostedStatus>> {
        self.sync.prepare_sync().await?;
        let _state_lock = self.state_store.acquire_write_lock()?;
        let mut state = self.state_store.load_or_init().await?;
        let registry = registry_from_state(&state)?;
        let preferred_alias = root_alias.map(RootAlias::parse).transpose()?;
        let requested = normalize_absolute_path(&expand_path_expression(path))?;
        let metadata = tokio::fs::metadata(&requested)
            .await
            .with_context(|| format!("io error at {}", requested.display()))?;

        if metadata.is_dir() {
            let root_mapping = registry.resolve_host_path(&requested, preferred_alias.as_ref())?;
            let key = EntryKey::new(
                self.config.space_id.clone(),
                root_mapping.root_alias.clone(),
                root_mapping.relative_path.clone(),
            );
            self.metadata.delete_ignored_path(&key).await?;
            upsert_hosted_root_state(
                &mut state,
                HostedRootState {
                    local_path: root_mapping.local_abs_path,
                    space_id: self.config.space_id.clone(),
                    root_alias: root_mapping.root_alias.to_string(),
                    relative_path: root_mapping.relative_path.to_string(),
                    hosted_at: Utc::now(),
                },
            );
        }

        let files = if metadata.is_dir() {
            collect_files(&requested)?
        } else {
            vec![requested.clone()]
        };

        let ignored = self.list_visible_ignored_paths().await?;
        let mut statuses = Vec::new();
        for file in files {
            let mut mapping = registry.resolve_host_path(&file, preferred_alias.as_ref())?;
            if let Some(remote_path) = remote_path
                && files_count_for_single_remote(metadata.is_dir())
            {
                mapping.relative_path = RelativePath::parse(remote_path)?;
            }
            let key = EntryKey::new(
                self.config.space_id.clone(),
                mapping.root_alias.clone(),
                mapping.relative_path.clone(),
            );
            if metadata.is_dir() && is_key_ignored(&key, &ignored) {
                continue;
            }
            let status = match self.host_file(&mut state, mapping).await {
                Ok(status) => status,
                Err(error) if metadata.is_dir() && is_skippable_directory_host_error(&error) => {
                    continue;
                }
                Err(error) => return Err(error),
            };
            statuses.push(status);
        }

        self.state_store.save(&state).await?;
        self.sync.flush_sync().await?;
        Ok(statuses)
    }

    /// Cancels local hosting without deleting local or remote content.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when state loading or saving fails.
    pub async fn unhost_path(&self, path: &str) -> anyhow::Result<usize> {
        self.sync.prepare_sync().await?;
        let _state_lock = self.state_store.acquire_write_lock()?;
        let requested = normalize_absolute_path(&expand_path_expression(path))?;
        let mut state = self.state_store.load_or_init().await?;
        let registry = registry_from_state(&state)?;
        let mut ignored_added = false;
        for root in &state.hosted_roots {
            if requested != root.local_path && requested.starts_with(&root.local_path) {
                let root_alias = RootAlias::parse(&root.root_alias)?;
                let mapping = registry.resolve_host_path(&requested, Some(&root_alias))?;
                let key = EntryKey::new(
                    root.space_id.clone(),
                    mapping.root_alias,
                    mapping.relative_path,
                );
                self.metadata
                    .upsert_ignored_path(&key, &self.config.device_id)
                    .await?;
                ignored_added = true;
            }
        }
        let before = state.hosted.len() + state.hosted_roots.len();
        state.hosted.retain(|hosted| {
            hosted.local_path != requested && !hosted.local_path.starts_with(&requested)
        });
        state.hosted_roots.retain(|hosted| {
            hosted.local_path != requested && !hosted.local_path.starts_with(&requested)
        });
        let removed = before
            .saturating_sub(state.hosted.len() + state.hosted_roots.len())
            .max(usize::from(ignored_added));
        self.state_store.save(&state).await?;
        self.sync.flush_sync().await?;
        Ok(removed)
    }

    /// Returns hosted status records.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when state loading fails.
    pub async fn status(&self, path: Option<&str>) -> anyhow::Result<Vec<HostedStatus>> {
        let state = self.state_store.load_or_init().await?;
        let requested = path
            .map(|path| normalize_absolute_path(&expand_path_expression(path)))
            .transpose()?;
        Ok(state
            .hosted
            .iter()
            .filter(|hosted| {
                requested.as_ref().is_none_or(|path| {
                    hosted.local_path == *path || hosted.local_path.starts_with(path)
                })
            })
            .map(hosted_status)
            .collect())
    }

    /// Lists tracked, remote, root, and optionally ignored paths without syncing.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when local state, directory walking, path
    /// mapping, or metadata queries fail.
    pub async fn list_tracked(
        &self,
        path: Option<&str>,
        options: ListTrackedOptions,
    ) -> anyhow::Result<Vec<TrackedItem>> {
        self.sync.prepare_sync().await?;
        let state = self.state_store.load_or_init().await?;
        let registry = registry_from_state(&state)?;
        let requested = path
            .map(|path| normalize_absolute_path(&expand_path_expression(path)))
            .transpose()?;
        let requested_mapping = requested
            .as_ref()
            .and_then(|path| registry.resolve_host_path(path, None).ok());
        let ignored = self
            .metadata
            .list_ignored_paths(&self.config.space_id, None, None)
            .await?;
        let suspended = self.list_visible_suspended_paths().await?;

        if options.roots_only {
            return Ok(list_hosted_root_items(
                &state,
                &registry,
                requested.as_deref(),
                requested_mapping.as_ref(),
            ));
        }

        let include_local = !options.include_remote || options.include_all;
        let include_remote = options.include_remote || options.include_all;
        let mut rows = BTreeMap::new();

        if include_local {
            insert_local_tracked_items(
                &mut rows,
                &state,
                &registry,
                &ignored,
                &suspended,
                requested.as_deref(),
                requested_mapping.as_ref(),
            )?;
        }

        if include_remote {
            let entries = self.list_visible_entries().await?;
            insert_remote_tracked_items(
                &mut rows,
                entries,
                &registry,
                &ignored,
                &suspended,
                requested.as_deref(),
                requested_mapping.as_ref(),
            );
        }

        if options.include_ignored {
            insert_db_ignored_items(
                &mut rows,
                &ignored,
                &registry,
                requested.as_deref(),
                requested_mapping.as_ref(),
            );
            if include_local {
                insert_gitignored_items(
                    &mut rows,
                    &state,
                    &registry,
                    &ignored,
                    requested.as_deref(),
                    requested_mapping.as_ref(),
                )?;
            }
        }

        let mut rows = rows.into_values().collect::<Vec<_>>();
        rows.sort_by(tracked_item_order);
        Ok(rows)
    }

    /// Pulls remote entries into this device's logical roots.
    ///
    /// This is the first-run operation for another computer: it maps remote
    /// keys like `main/home/.agents/skills/x` to this machine's `$HOME` root,
    /// writes the remote bytes, and persists the local hosted state so later
    /// sync cycles can upload local edits or pull newer remote versions.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when state, root mapping, metadata, object
    /// storage, or local file writes fail.
    pub async fn pull_remote(
        &self,
        path: Option<&str>,
        options: PullRemoteOptions,
    ) -> anyhow::Result<Vec<PullRemoteItem>> {
        self.sync.prepare_sync().await?;
        let _state_lock = self.state_store.acquire_write_lock()?;
        let mut state = self.state_store.load_or_init().await?;
        let rows = self
            .materialize_remote_entries(&mut state, path, options, RemoteMaterializeMode::All)
            .await?;
        if !options.dry_run {
            self.state_store.save(&state).await?;
            self.sync.flush_sync().await?;
        }
        Ok(rows)
    }

    async fn materialize_remote_entries(
        &self,
        state: &mut LocalState,
        path: Option<&str>,
        options: PullRemoteOptions,
        mode: RemoteMaterializeMode,
    ) -> anyhow::Result<Vec<PullRemoteItem>> {
        let registry = registry_from_state(state)?;
        let requested = path
            .map(|path| normalize_absolute_path(&expand_path_expression(path)))
            .transpose()?;
        let requested_mapping = requested
            .as_ref()
            .and_then(|path| registry.resolve_host_path(path, None).ok());
        let ignored = self.list_visible_ignored_paths().await?;
        let entries = self.list_visible_entries().await?;
        let mut rows = Vec::new();

        for entry in entries {
            if mode == RemoteMaterializeMode::UntrackedOnly
                && entry.key.space_id != self.config.space_id
                && !self
                    .config
                    .auto_materialize_space_ids
                    .contains(&entry.key.space_id)
            {
                continue;
            }
            if item_filtered_out(&entry.key, requested_mapping.as_ref()) {
                continue;
            }
            let Some(local_path) = local_path_for_key(&registry, &entry.key) else {
                continue;
            };
            if self.metadata.get_suspended_path(entry.id).await?.is_some() {
                rows.push(pull_item_from_entry(
                    PullRemoteStatus::SkippedSuspended,
                    local_path,
                    &entry,
                    &registry,
                    None,
                ));
                continue;
            }
            if mode == RemoteMaterializeMode::UntrackedOnly
                && state_has_hosted_key_or_path(state, &entry.key, &local_path)
            {
                continue;
            }
            if is_key_ignored(&entry.key, &ignored) {
                rows.push(pull_item_from_entry(
                    PullRemoteStatus::SkippedIgnored,
                    local_path,
                    &entry,
                    &registry,
                    None,
                ));
                continue;
            }
            let Some(remote) = self.metadata.latest_version(entry.id).await? else {
                rows.push(pull_item_from_entry(
                    PullRemoteStatus::SkippedNoVersion,
                    local_path,
                    &entry,
                    &registry,
                    None,
                ));
                continue;
            };
            if options.dry_run {
                rows.push(pull_item_from_entry(
                    PullRemoteStatus::DryRun,
                    local_path,
                    &entry,
                    &registry,
                    Some(&remote),
                ));
                continue;
            }

            let task_id = self
                .start_sync_task(
                    DriveSyncTaskKind::Materialize,
                    &entry.key,
                    Some(&local_path),
                )
                .await?;
            let result = async {
                let remote_bytes = self.objects.get_object(&remote.object_key).await?;
                let status = if local_path.exists() {
                    let local_bytes = read_file(&local_path).await?;
                    if content_hash(&local_bytes) == remote.content_hash {
                        PullRemoteStatus::AlreadyCurrent
                    } else if options.overwrite {
                        write_file(&local_path, &remote_bytes).await?;
                        PullRemoteStatus::Pulled
                    } else {
                        return Ok(None);
                    }
                } else {
                    write_file(&local_path, &remote_bytes).await?;
                    PullRemoteStatus::Pulled
                };

                upsert_hosted_state(
                    state,
                    HostedPathState {
                        local_path: local_path.clone(),
                        space_id: entry.key.space_id.clone(),
                        root_alias: entry.key.root_alias.to_string(),
                        relative_path: entry.key.relative_path.to_string(),
                        base_version: Some(remote.version),
                        base_hash: Some(remote.content_hash.clone()),
                        content_hash: Some(remote.content_hash.clone()),
                        hosted_at: Utc::now(),
                        last_synced_at: Some(Utc::now()),
                    },
                );
                Ok(Some(status))
            };
            match result.await {
                Ok(Some(status)) => {
                    self.finish_sync_task(task_id).await?;
                    rows.push(pull_item_from_entry(
                        status,
                        local_path,
                        &entry,
                        &registry,
                        Some(&remote),
                    ));
                }
                Ok(None) => {
                    self.finish_sync_task(task_id).await?;
                    rows.push(pull_item_from_entry(
                        PullRemoteStatus::SkippedExisting,
                        local_path,
                        &entry,
                        &registry,
                        Some(&remote),
                    ));
                }
                Err(error) => {
                    self.fail_sync_task(task_id, &error).await?;
                    return Err(error);
                }
            }
        }

        rows.sort_by(|left, right| {
            left.canonical_path
                .cmp(&right.canonical_path)
                .then_with(|| left.remote_path.cmp(&right.remote_path))
        });
        Ok(rows)
    }

    async fn list_visible_entries(&self) -> anyhow::Result<Vec<DriveEntry>> {
        let mut entries = Vec::new();
        for space_id in self.config.visible_space_ids() {
            entries.extend(self.metadata.list_entries_by_space(&space_id).await?);
        }
        Ok(entries)
    }

    async fn list_visible_ignored_paths(&self) -> anyhow::Result<Vec<DriveIgnoredPath>> {
        let mut ignored = Vec::new();
        for space_id in self.config.visible_space_ids() {
            ignored.extend(
                self.metadata
                    .list_ignored_paths(&space_id, None, None)
                    .await?,
            );
        }
        Ok(ignored)
    }

    async fn list_visible_suspended_paths(&self) -> anyhow::Result<Vec<DriveSuspendedPath>> {
        let visible = self.config.visible_space_ids();
        Ok(self
            .metadata
            .list_suspended_paths()
            .await?
            .into_iter()
            .filter(|item| visible.contains(&item.space_id))
            .collect())
    }

    /// Lists unresolved conflicts from the server-side metadata store.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when store access fails.
    pub async fn conflicts(&self) -> anyhow::Result<Vec<DriveConflict>> {
        self.sync.prepare_sync().await?;
        self.metadata.list_conflicts(Some(false)).await
    }

    /// Lists durable sync queue items.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when store access fails.
    pub async fn sync_queue(
        &self,
        status: Option<DriveSyncTaskStatus>,
    ) -> anyhow::Result<Vec<DriveSyncQueueItem>> {
        self.sync.prepare_sync().await?;
        self.metadata.list_sync_queue(status).await
    }

    /// Marks failed queue items pending and runs one sync pass.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when queue mutation or sync fails.
    pub async fn retry_sync_queue(&self) -> anyhow::Result<u64> {
        self.sync.prepare_sync().await?;
        let retried = self.metadata.retry_failed_sync_tasks().await?;
        self.sync.flush_sync().await?;
        if retried > 0 {
            self.sync_once().await?;
        }
        Ok(retried)
    }

    /// Resolves a suspended conflict.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when metadata, local files, or state writes fail.
    pub async fn resolve_conflict(
        &self,
        conflict_id: Uuid,
        resolution: ConflictResolution,
    ) -> anyhow::Result<Option<DriveConflict>> {
        self.sync.prepare_sync().await?;
        let _state_lock = self.state_store.acquire_write_lock()?;
        let Some(conflict) = self
            .metadata
            .list_conflicts(Some(false))
            .await?
            .into_iter()
            .find(|conflict| conflict.id == conflict_id)
        else {
            return Ok(None);
        };
        let mut state = self.state_store.load_or_init().await?;
        let registry = registry_from_state(&state)?;
        let Some(entry) = self.metadata.get_entry_by_id(conflict.entry_id).await? else {
            return Ok(None);
        };
        let Some(local_path) = local_path_for_key(&registry, &entry.key) else {
            return Ok(None);
        };
        match &resolution {
            ConflictResolution::KeepRemote => {}
            ConflictResolution::KeepLocal => {
                let bytes = read_file(Path::new(&conflict.conflict_path)).await?;
                write_file(&local_path, &bytes).await?;
            }
            ConflictResolution::UseMerged(path) => {
                let path =
                    normalize_absolute_path(&expand_path_expression(path.display().to_string()))?;
                let bytes = read_file(&path).await?;
                write_file(&local_path, &bytes).await?;
            }
        }
        let resolved = self.metadata.resolve_conflict(conflict_id).await?;
        self.metadata
            .delete_suspended_path(conflict.entry_id)
            .await?;
        state.conflicts.retain(|item| item.id != conflict_id);
        self.state_store.save(&state).await?;
        self.sync.flush_sync().await?;
        Ok(resolved)
    }

    /// Migrates legacy `main` state into the current login owner's Drive id.
    ///
    /// The public product model is user/API-key ownership. The storage schema
    /// still uses a namespace field internally, so this method performs the
    /// one-time local and metadata namespace rewrite needed for old installs.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when metadata or local state persistence
    /// fails.
    pub async fn migrate_legacy_owner_drive(
        &self,
        legacy_owner_drive_id: &str,
        owner_drive_id: &str,
    ) -> anyhow::Result<u64> {
        self.sync.prepare_sync().await?;
        let _state_lock = self.state_store.acquire_write_lock()?;
        if legacy_owner_drive_id == owner_drive_id {
            return Ok(0);
        }
        let metadata_count = self
            .metadata
            .migrate_owner_drive_namespace(legacy_owner_drive_id, owner_drive_id)
            .await?;
        let mut state = self.state_store.load_or_init().await?;
        let mut local_count = 0;
        for hosted in &mut state.hosted {
            if hosted.space_id == legacy_owner_drive_id {
                hosted.space_id = owner_drive_id.to_owned();
                local_count += 1;
            }
        }
        for root in &mut state.hosted_roots {
            if root.space_id == legacy_owner_drive_id {
                root.space_id = owner_drive_id.to_owned();
                local_count += 1;
            }
        }
        if local_count > 0 {
            self.state_store.save(&state).await?;
        }
        self.sync.flush_sync().await?;
        Ok(metadata_count + local_count)
    }

    /// Performs one bidirectional synchronization scan for all hosted paths.
    ///
    /// Visible remote entries from fused owner Drives are materialized once before
    /// the local scan, so adding a trusted API key makes that owner's hosted
    /// files participate in normal sync without a separate user action.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] when local I/O or remote store operations fail.
    pub async fn sync_once(&self) -> anyhow::Result<Vec<HostedStatus>> {
        self.sync.prepare_sync().await?;
        let _state_lock = self.state_store.acquire_write_lock()?;
        let mut state = self.state_store.load_or_init().await?;
        self.materialize_remote_entries(
            &mut state,
            None,
            PullRemoteOptions::default(),
            RemoteMaterializeMode::UntrackedOnly,
        )
        .await?;
        self.discover_hosted_root_files(&mut state).await?;
        let mut statuses = Vec::new();
        let mut hosted_records = std::mem::take(&mut state.hosted);
        for mut hosted in hosted_records.drain(..) {
            self.sync_hosted(&mut state, &mut hosted).await?;
            statuses.push(hosted_status(&hosted));
            state.hosted.push(hosted);
        }
        self.state_store.save(&state).await?;
        self.sync.flush_sync().await?;
        Ok(statuses)
    }

    /// Runs the polling realtime daemon until interrupted.
    ///
    /// # Errors
    /// Returns [`anyhow::Error`] if a sync cycle fails.
    pub async fn run_polling_daemon(&self) -> anyhow::Result<()> {
        loop {
            self.sync_once().await?;
            tokio::time::sleep(self.config.poll_interval).await;
        }
    }

    async fn start_sync_task(
        &self,
        kind: DriveSyncTaskKind,
        key: &EntryKey,
        local_path: Option<&Path>,
    ) -> anyhow::Result<Uuid> {
        let now = Utc::now();
        let item = DriveSyncQueueItem {
            id: Uuid::new_v4(),
            kind,
            status: DriveSyncTaskStatus::Pending,
            space_id: key.space_id.clone(),
            root_alias: key.root_alias.clone(),
            relative_path: key.relative_path.clone(),
            local_path: local_path.map(|path| path.display().to_string()),
            remote_path: key.remote_path(),
            attempts: 0,
            last_error: None,
            created_at: now,
            updated_at: now,
        };
        let id = item.id;
        self.metadata.enqueue_sync_task(item).await?;
        self.metadata
            .update_sync_task(id, DriveSyncTaskStatus::Running, None)
            .await?;
        Ok(id)
    }

    async fn finish_sync_task(&self, id: Uuid) -> anyhow::Result<()> {
        self.metadata
            .update_sync_task(id, DriveSyncTaskStatus::Done, None)
            .await?;
        Ok(())
    }

    async fn fail_sync_task(&self, id: Uuid, error: &anyhow::Error) -> anyhow::Result<()> {
        self.metadata
            .update_sync_task(id, DriveSyncTaskStatus::Failed, Some(&error.to_string()))
            .await?;
        Ok(())
    }

    async fn host_file(
        &self,
        state: &mut LocalState,
        mapping: HostPathMapping,
    ) -> anyhow::Result<HostedStatus> {
        let bytes = read_file(&mapping.local_abs_path).await?;
        let hash = content_hash(&bytes);
        let object_key = object_key_for_hash(&hash);
        let key = EntryKey::new(
            self.config.space_id.clone(),
            mapping.root_alias.clone(),
            mapping.relative_path.clone(),
        );
        self.metadata.delete_ignored_path(&key).await?;
        let entry = self
            .metadata
            .upsert_entry(&key, DriveEntryKind::File)
            .await?;
        let latest = self.metadata.latest_version(entry.id).await?;

        let (base_version, base_hash) = match latest {
            Some(remote) if remote.content_hash == hash => {
                (Some(remote.version), Some(remote.content_hash))
            }
            Some(remote) => {
                self.write_conflict_and_restore_remote(
                    state,
                    ConflictRestoreRequest {
                        local_path: &mapping.local_abs_path,
                        entry_id: entry.id,
                        base_version: None,
                        local_hash: &hash,
                        remote: &remote,
                        local_bytes: &bytes,
                    },
                )
                .await?;
                (Some(remote.version), Some(remote.content_hash))
            }
            None => {
                if !self.objects.object_exists(&object_key).await? {
                    self.objects.put_object(&object_key, &bytes).await?;
                }
                let version = DriveVersion {
                    id: Uuid::new_v4(),
                    entry_id: entry.id,
                    version: 1,
                    content_hash: hash.clone(),
                    object_key,
                    size_bytes: bytes.len() as u64,
                    device_id: self.config.device_id.clone(),
                    modified_at: Utc::now(),
                };
                self.metadata.insert_version(version).await?;
                (Some(1), Some(hash.clone()))
            }
        };

        let hosted = HostedPathState {
            local_path: mapping.local_abs_path,
            space_id: self.config.space_id.clone(),
            root_alias: mapping.root_alias.to_string(),
            relative_path: mapping.relative_path.to_string(),
            base_version,
            base_hash: base_hash.clone(),
            content_hash: base_hash,
            hosted_at: Utc::now(),
            last_synced_at: Some(Utc::now()),
        };
        let status = hosted_status(&hosted);
        upsert_hosted_state(state, hosted);
        Ok(status)
    }

    async fn sync_hosted(
        &self,
        state: &mut LocalState,
        hosted: &mut HostedPathState,
    ) -> anyhow::Result<()> {
        if !hosted.local_path.exists() {
            return Ok(());
        }
        let key = EntryKey::new(
            hosted.space_id.clone(),
            RootAlias::parse(&hosted.root_alias)?,
            RelativePath::parse(&hosted.relative_path)?,
        );
        let entry = self
            .metadata
            .upsert_entry(&key, DriveEntryKind::File)
            .await?;
        if self.metadata.get_suspended_path(entry.id).await?.is_some() {
            return Ok(());
        }
        let latest = self.metadata.latest_version(entry.id).await?;
        let local_bytes = read_file(&hosted.local_path).await?;
        let local_hash = content_hash(&local_bytes);
        let remote_version = latest.as_ref().map(|version| version.version);

        if let Some(remote) = &latest
            && hosted.content_hash.as_deref() == Some(&local_hash)
            && hosted
                .base_version
                .is_some_and(|base| remote.version > base)
        {
            let task_id = self
                .start_sync_task(DriveSyncTaskKind::Download, &key, Some(&hosted.local_path))
                .await?;
            let result = async {
                let remote_bytes = self.objects.get_object(&remote.object_key).await?;
                write_file(&hosted.local_path, &remote_bytes).await
            };
            if let Err(error) = result.await {
                self.fail_sync_task(task_id, &error).await?;
                return Err(error);
            }
            hosted.base_version = Some(remote.version);
            hosted.base_hash = Some(remote.content_hash.clone());
            hosted.content_hash = Some(remote.content_hash.clone());
            hosted.last_synced_at = Some(Utc::now());
            if let Some(ref notify) = self.config.on_file_synced {
                notify(hosted.relative_path.clone());
            }
            self.finish_sync_task(task_id).await?;
            return Ok(());
        }

        if hosted.content_hash.as_deref() == Some(&local_hash) {
            let object_key = object_key_for_hash(&local_hash);
            let needs_version_repair = latest
                .as_ref()
                .is_none_or(|remote| remote.content_hash != local_hash);
            if !self.objects.object_exists(&object_key).await? || needs_version_repair {
                let task_id = self
                    .start_sync_task(DriveSyncTaskKind::Upload, &key, Some(&hosted.local_path))
                    .await?;
                let version = DriveVersion {
                    id: Uuid::new_v4(),
                    entry_id: entry.id,
                    version: remote_version.unwrap_or(0).saturating_add(1),
                    content_hash: local_hash.clone(),
                    object_key: object_key.clone(),
                    size_bytes: local_bytes.len() as u64,
                    device_id: self.config.device_id.clone(),
                    modified_at: Utc::now(),
                };
                let result = async {
                    if !self.objects.object_exists(&object_key).await? {
                        self.objects.put_object(&object_key, &local_bytes).await?;
                    }
                    if needs_version_repair {
                        self.metadata.insert_version(version).await?;
                    }
                    Ok::<(), anyhow::Error>(())
                };
                if let Err(error) = result.await {
                    self.fail_sync_task(task_id, &error).await?;
                    return Err(error);
                }
                if needs_version_repair {
                    hosted.base_version = Some(remote_version.unwrap_or(0).saturating_add(1));
                    hosted.base_hash = Some(local_hash.clone());
                    hosted.content_hash = Some(local_hash);
                    hosted.last_synced_at = Some(Utc::now());
                    if let Some(ref notify) = self.config.on_file_synced {
                        notify(hosted.relative_path.clone());
                    }
                }
                self.finish_sync_task(task_id).await?;
            }
            return Ok(());
        }

        let remote_hash = latest.as_ref().map(|version| version.content_hash.as_str());
        match decide_local_change(
            hosted.base_version,
            remote_version,
            &local_hash,
            remote_hash,
            None,
            &self.config.device_id,
            Utc::now(),
        ) {
            ChangeDecision::NoopSameContent => {
                if let Some(remote) = latest {
                    hosted.base_version = Some(remote.version);
                    hosted.base_hash = Some(remote.content_hash.clone());
                    hosted.content_hash = Some(remote.content_hash);
                    hosted.last_synced_at = Some(Utc::now());
                }
            }
            ChangeDecision::UploadNewVersion => {
                let task_id = self
                    .start_sync_task(DriveSyncTaskKind::Upload, &key, Some(&hosted.local_path))
                    .await?;
                let object_key = object_key_for_hash(&local_hash);
                let next_version = remote_version.unwrap_or(0).saturating_add(1);
                let version = DriveVersion {
                    id: Uuid::new_v4(),
                    entry_id: entry.id,
                    version: next_version,
                    content_hash: local_hash.clone(),
                    object_key: object_key.clone(),
                    size_bytes: local_bytes.len() as u64,
                    device_id: self.config.device_id.clone(),
                    modified_at: Utc::now(),
                };
                let result = async {
                    if !self.objects.object_exists(&object_key).await? {
                        self.objects.put_object(&object_key, &local_bytes).await?;
                    }
                    self.metadata.insert_version(version).await?;
                    Ok::<_, anyhow::Error>(())
                };
                if let Err(error) = result.await {
                    self.fail_sync_task(task_id, &error).await?;
                    return Err(error);
                }
                hosted.base_version = Some(next_version);
                hosted.base_hash = Some(local_hash.clone());
                hosted.content_hash = Some(local_hash);
                hosted.last_synced_at = Some(Utc::now());
                self.finish_sync_task(task_id).await?;
            }
            ChangeDecision::Conflict => {
                if let Some(remote) = latest {
                    let task_id = self
                        .start_sync_task(
                            DriveSyncTaskKind::Conflict,
                            &key,
                            Some(&hosted.local_path),
                        )
                        .await?;
                    let result = self
                        .try_merge_or_write_conflict(state, hosted, entry.id, remote, &local_bytes)
                        .await;
                    if let Err(error) = result {
                        self.fail_sync_task(task_id, &error).await?;
                        return Err(error);
                    }
                    self.finish_sync_task(task_id).await?;
                }
            }
            ChangeDecision::LockedByOther { .. } => {}
        }
        Ok(())
    }

    async fn discover_hosted_root_files(&self, state: &mut LocalState) -> anyhow::Result<()> {
        let registry = registry_from_state(state)?;
        let roots = state.hosted_roots.clone();
        for root in roots {
            if !root.local_path.exists() {
                continue;
            }
            let ignored = self
                .metadata
                .list_ignored_paths(&root.space_id, None, None)
                .await?;
            let root_alias = RootAlias::parse(&root.root_alias)?;
            for file in collect_files(&root.local_path)? {
                if state.hosted.iter().any(|hosted| hosted.local_path == file) {
                    continue;
                }
                let mapping = registry.resolve_host_path(&file, Some(&root_alias))?;
                let key = EntryKey::new(
                    root.space_id.clone(),
                    mapping.root_alias.clone(),
                    mapping.relative_path.clone(),
                );
                if is_key_ignored(&key, &ignored) {
                    continue;
                }
                if let Some(entry) = self.metadata.get_entry(&key).await?
                    && self.metadata.get_suspended_path(entry.id).await?.is_some()
                {
                    continue;
                }
                if let Err(error) = self.host_file(state, mapping).await {
                    if is_skippable_directory_host_error(&error) {
                        continue;
                    }
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    async fn try_merge_or_write_conflict(
        &self,
        state: &mut LocalState,
        hosted: &mut HostedPathState,
        entry_id: Uuid,
        remote: DriveVersion,
        local_bytes: &[u8],
    ) -> anyhow::Result<()> {
        let remote_bytes = self.objects.get_object(&remote.object_key).await?;
        let merged = match &hosted.base_hash {
            Some(base_hash) => {
                let base_key = object_key_for_hash(base_hash);
                let base_bytes = self.objects.get_object(&base_key).await.ok();
                base_bytes
                    .as_deref()
                    .and_then(|base| try_safe_text_merge(base, local_bytes, &remote_bytes))
            }
            None => None,
        };

        if let Some(merged) = merged {
            write_file(&hosted.local_path, &merged).await?;
            let merged_hash = content_hash(&merged);
            let object_key = object_key_for_hash(&merged_hash);
            if !self.objects.object_exists(&object_key).await? {
                self.objects.put_object(&object_key, &merged).await?;
            }
            let next_version = remote.version.saturating_add(1);
            self.metadata
                .insert_version(DriveVersion {
                    id: Uuid::new_v4(),
                    entry_id,
                    version: next_version,
                    content_hash: merged_hash.clone(),
                    object_key,
                    size_bytes: merged.len() as u64,
                    device_id: self.config.device_id.clone(),
                    modified_at: Utc::now(),
                })
                .await?;
            hosted.base_version = Some(next_version);
            hosted.base_hash = Some(merged_hash.clone());
            hosted.content_hash = Some(merged_hash);
            hosted.last_synced_at = Some(Utc::now());
            return Ok(());
        }

        let local_hash = content_hash(local_bytes);
        self.write_conflict_and_restore_remote(
            state,
            ConflictRestoreRequest {
                local_path: &hosted.local_path,
                entry_id,
                base_version: hosted.base_version,
                local_hash: &local_hash,
                remote: &remote,
                local_bytes,
            },
        )
        .await?;
        hosted.base_version = Some(remote.version);
        hosted.base_hash = Some(remote.content_hash.clone());
        hosted.content_hash = Some(remote.content_hash);
        hosted.last_synced_at = Some(Utc::now());
        Ok(())
    }

    async fn write_conflict_and_restore_remote(
        &self,
        state: &mut LocalState,
        request: ConflictRestoreRequest<'_>,
    ) -> anyhow::Result<()> {
        let now = Utc::now();
        let conflict_name = conflict_file_name(request.local_path, &self.config.device_name, now);
        let conflict_path = request
            .local_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(conflict_name);
        write_file(&conflict_path, request.local_bytes).await?;

        let remote_bytes = self.objects.get_object(&request.remote.object_key).await?;
        write_file(request.local_path, &remote_bytes).await?;

        let conflict = self
            .metadata
            .record_conflict(DriveConflict {
                id: Uuid::new_v4(),
                entry_id: request.entry_id,
                base_version: request.base_version,
                local_hash: request.local_hash.to_owned(),
                remote_hash: request.remote.content_hash.clone(),
                device_id: self.config.device_id.clone(),
                conflict_path: conflict_path.display().to_string(),
                resolved: false,
                created_at: now,
            })
            .await?;
        if let Some(entry) = self.metadata.get_entry_by_id(request.entry_id).await? {
            self.metadata
                .upsert_suspended_path(DriveSuspendedPath {
                    id: Uuid::new_v4(),
                    entry_id: request.entry_id,
                    space_id: entry.key.space_id,
                    root_alias: entry.key.root_alias,
                    relative_path: entry.key.relative_path,
                    reason: "conflict".to_owned(),
                    conflict_id: Some(conflict.id),
                    created_at: now,
                    updated_at: now,
                })
                .await?;
        }
        state.conflicts.push(LocalConflictState {
            id: conflict.id,
            conflict_path,
            created_at: now,
        });
        Ok(())
    }
}

fn registry_from_state(state: &LocalState) -> anyhow::Result<RootRegistry> {
    let mut registry = RootRegistry::default_for_device()?;
    for root in &state.roots {
        registry.add_root(RootAlias::parse(&root.alias)?, &root.path)?;
    }
    Ok(registry)
}

fn state_has_hosted_key_or_path(state: &LocalState, key: &EntryKey, local_path: &Path) -> bool {
    state.hosted.iter().any(|hosted| {
        hosted.local_path == local_path
            || (hosted.space_id == key.space_id
                && hosted.root_alias == key.root_alias.to_string()
                && hosted.relative_path == key.relative_path.to_string())
    })
}

fn upsert_hosted_state(state: &mut LocalState, hosted: HostedPathState) {
    state
        .hosted
        .retain(|item| item.local_path != hosted.local_path);
    state.hosted.push(hosted);
    state
        .hosted
        .sort_by(|left, right| left.local_path.cmp(&right.local_path));
}

fn upsert_hosted_root_state(state: &mut LocalState, hosted: HostedRootState) {
    state
        .hosted_roots
        .retain(|item| item.local_path != hosted.local_path);
    state.hosted_roots.push(hosted);
    state
        .hosted_roots
        .sort_by(|left, right| left.local_path.cmp(&right.local_path));
}

fn hosted_status(hosted: &HostedPathState) -> HostedStatus {
    HostedStatus {
        owner_drive_id: hosted.space_id.clone(),
        local_path: hosted.local_path.clone(),
        remote_path: format!(
            "{}/{}/{}",
            hosted.space_id, hosted.root_alias, hosted.relative_path
        ),
        base_version: hosted.base_version,
        content_hash: hosted.content_hash.clone(),
        exists: hosted.local_path.exists(),
    }
}

fn list_hosted_root_items(
    state: &LocalState,
    registry: &RootRegistry,
    requested: Option<&Path>,
    requested_mapping: Option<&HostPathMapping>,
) -> Vec<TrackedItem> {
    let mut rows = state
        .hosted_roots
        .iter()
        .filter_map(|root| {
            let key = key_from_hosted_root(root)?;
            let local_path = root.local_path.clone();
            let item = tracked_item_from_key(
                TrackedItemStatus::Root,
                TrackedItemSource::Local,
                Some(local_path),
                &key,
                None,
                None,
                registry,
            );
            item_matches_filter(&item, requested, requested_mapping).then_some(item)
        })
        .collect::<Vec<_>>();
    rows.sort_by(tracked_item_order);
    rows
}

fn insert_local_tracked_items(
    rows: &mut BTreeMap<String, TrackedItem>,
    state: &LocalState,
    registry: &RootRegistry,
    ignored: &[DriveIgnoredPath],
    suspended: &[DriveSuspendedPath],
    requested: Option<&Path>,
    requested_mapping: Option<&HostPathMapping>,
) -> anyhow::Result<()> {
    let mut hosted_paths = BTreeSet::new();
    for hosted in &state.hosted {
        hosted_paths.insert(hosted.local_path.clone());
        let key = key_from_hosted(hosted)?;
        if is_key_ignored(&key, ignored) {
            continue;
        }
        let status = if is_key_suspended(&key, suspended) {
            TrackedItemStatus::ConflictSuspended
        } else if hosted.local_path.exists() {
            TrackedItemStatus::Tracked
        } else {
            TrackedItemStatus::MissingLocal
        };
        let item = tracked_item_from_key(
            status,
            TrackedItemSource::Local,
            Some(hosted.local_path.clone()),
            &key,
            hosted.base_version,
            hosted.content_hash.clone(),
            registry,
        );
        if item_matches_filter(&item, requested, requested_mapping) {
            rows.insert(item.remote_path.clone(), item);
        }
    }

    for root in &state.hosted_roots {
        if !root.local_path.exists() {
            continue;
        }
        let root_alias = RootAlias::parse(&root.root_alias)?;
        for file in collect_files(&root.local_path)? {
            if hosted_paths.contains(&file) {
                continue;
            }
            let mapping = registry.resolve_host_path(&file, Some(&root_alias))?;
            let key = EntryKey::new(
                root.space_id.clone(),
                mapping.root_alias,
                mapping.relative_path,
            );
            if is_key_ignored(&key, ignored) {
                continue;
            }
            let item = tracked_item_from_key(
                if is_key_suspended(&key, suspended) {
                    TrackedItemStatus::ConflictSuspended
                } else {
                    TrackedItemStatus::Tracked
                },
                TrackedItemSource::Local,
                Some(file),
                &key,
                None,
                None,
                registry,
            );
            if item_matches_filter(&item, requested, requested_mapping) {
                rows.insert(item.remote_path.clone(), item);
            }
        }
    }

    Ok(())
}

fn insert_remote_tracked_items(
    rows: &mut BTreeMap<String, TrackedItem>,
    entries: Vec<DriveEntry>,
    registry: &RootRegistry,
    ignored: &[DriveIgnoredPath],
    suspended: &[DriveSuspendedPath],
    requested: Option<&Path>,
    requested_mapping: Option<&HostPathMapping>,
) {
    for entry in entries {
        if is_key_ignored(&entry.key, ignored) {
            continue;
        }
        let remote_path = entry.key.remote_path();
        if let Some(existing) = rows.get_mut(&remote_path) {
            existing.source = TrackedItemSource::Both;
            if is_key_suspended(&entry.key, suspended) {
                existing.status = TrackedItemStatus::ConflictSuspended;
            }
            existing.base_version = existing.base_version.or(Some(entry.latest_version));
            if existing.content_hash.is_none() {
                existing.content_hash.clone_from(&entry.latest_hash);
            }
            continue;
        }

        let item = tracked_item_from_key(
            if is_key_suspended(&entry.key, suspended) {
                TrackedItemStatus::ConflictSuspended
            } else {
                TrackedItemStatus::RemoteOnly
            },
            if is_key_suspended(&entry.key, suspended) {
                TrackedItemSource::Suspended
            } else {
                TrackedItemSource::Remote
            },
            None,
            &entry.key,
            Some(entry.latest_version),
            entry.latest_hash,
            registry,
        );
        if item_matches_filter(&item, requested, requested_mapping) {
            rows.insert(remote_path, item);
        }
    }
}

fn insert_db_ignored_items(
    rows: &mut BTreeMap<String, TrackedItem>,
    ignored: &[DriveIgnoredPath],
    registry: &RootRegistry,
    requested: Option<&Path>,
    requested_mapping: Option<&HostPathMapping>,
) {
    for ignored in ignored {
        let key = EntryKey::new(
            ignored.space_id.clone(),
            ignored.root_alias.clone(),
            ignored.relative_path.clone(),
        );
        let local_path = local_path_for_key(registry, &key);
        let item = tracked_item_from_key(
            TrackedItemStatus::Ignored,
            TrackedItemSource::DbIgnore,
            local_path,
            &key,
            None,
            None,
            registry,
        );
        if item_matches_filter(&item, requested, requested_mapping) {
            rows.insert(format!("ignored:{}", item.remote_path), item);
        }
    }
}

fn insert_gitignored_items(
    rows: &mut BTreeMap<String, TrackedItem>,
    state: &LocalState,
    registry: &RootRegistry,
    ignored: &[DriveIgnoredPath],
    requested: Option<&Path>,
    requested_mapping: Option<&HostPathMapping>,
) -> anyhow::Result<()> {
    for root in &state.hosted_roots {
        if !root.local_path.exists() {
            continue;
        }
        let root_alias = RootAlias::parse(&root.root_alias)?;
        for file in collect_gitignored_files(&root.local_path)? {
            let mapping = registry.resolve_host_path(&file, Some(&root_alias))?;
            let key = EntryKey::new(
                root.space_id.clone(),
                mapping.root_alias,
                mapping.relative_path,
            );
            if is_key_ignored(&key, ignored) {
                continue;
            }
            let item = tracked_item_from_key(
                TrackedItemStatus::Ignored,
                TrackedItemSource::Gitignore,
                Some(file),
                &key,
                None,
                None,
                registry,
            );
            if item_matches_filter(&item, requested, requested_mapping) {
                rows.insert(format!("gitignore:{}", item.remote_path), item);
            }
        }
    }
    Ok(())
}

fn tracked_item_from_key(
    status: TrackedItemStatus,
    source: TrackedItemSource,
    local_path: Option<PathBuf>,
    key: &EntryKey,
    base_version: Option<u64>,
    content_hash: Option<String>,
    registry: &RootRegistry,
) -> TrackedItem {
    let exists = local_path.as_ref().is_some_and(|path| path.exists());
    TrackedItem {
        status,
        source,
        local_path,
        display_path: display_path_for_key(key, registry),
        canonical_path: canonical_path_for_key(key),
        remote_path: key.remote_path(),
        owner_drive_id: key.space_id.clone(),
        root_alias: key.root_alias.to_string(),
        relative_path: key.relative_path.to_string(),
        base_version,
        content_hash,
        exists,
    }
}

fn pull_item_from_entry(
    status: PullRemoteStatus,
    local_path: PathBuf,
    entry: &DriveEntry,
    registry: &RootRegistry,
    version: Option<&DriveVersion>,
) -> PullRemoteItem {
    PullRemoteItem {
        status,
        exists: local_path.exists(),
        local_path,
        display_path: display_path_for_key(&entry.key, registry),
        canonical_path: canonical_path_for_key(&entry.key),
        remote_path: entry.key.remote_path(),
        owner_drive_id: entry.key.space_id.clone(),
        base_version: version
            .map(|version| version.version)
            .or(Some(entry.latest_version))
            .filter(|version| *version > 0),
        content_hash: version
            .map(|version| version.content_hash.clone())
            .or_else(|| entry.latest_hash.clone()),
    }
}

fn item_filtered_out(
    key: &EntryKey,
    requested_mapping: Option<&HostPathMapping>,
) -> bool {
    requested_mapping.is_some_and(|mapping| {
        key.root_alias != mapping.root_alias
            || !relative_matches_prefix(key.relative_path.as_str(), mapping.relative_path.as_str())
    })
}

fn key_from_hosted(hosted: &HostedPathState) -> anyhow::Result<EntryKey> {
    Ok(EntryKey::new(
        hosted.space_id.clone(),
        RootAlias::parse(&hosted.root_alias)?,
        RelativePath::parse(&hosted.relative_path)?,
    ))
}

fn key_from_hosted_root(root: &HostedRootState) -> Option<EntryKey> {
    Some(EntryKey::new(
        root.space_id.clone(),
        RootAlias::parse(&root.root_alias).ok()?,
        RelativePath::parse(&root.relative_path).ok()?,
    ))
}

fn local_path_for_key(registry: &RootRegistry, key: &EntryKey) -> Option<PathBuf> {
    registry
        .list_roots()
        .into_iter()
        .find(|root| root.alias == key.root_alias)
        .map(|root| root.local_path.join(key.relative_path.to_local_path()))
}

fn display_path_for_key(key: &EntryKey, registry: &RootRegistry) -> String {
    if key.root_alias.as_str() == RootAlias::HOME {
        return display_home_relative_path(&key.relative_path);
    }
    local_path_for_key(registry, key)
        .map(|path| display_local_path(&path))
        .unwrap_or_else(|| canonical_path_for_key(key))
}

fn display_home_relative_path(relative_path: &RelativePath) -> String {
    if relative_path.is_root() {
        "$HOME".to_owned()
    } else {
        format!("$HOME/{}", relative_path.as_str())
    }
}

fn display_local_path(path: &Path) -> String {
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from)
        && let Ok(relative) = path.strip_prefix(home)
    {
        return if relative.as_os_str().is_empty() {
            "$HOME".to_owned()
        } else {
            format!("$HOME/{}", relative.display())
        };
    }
    path.display().to_string()
}

fn canonical_path_for_key(key: &EntryKey) -> String {
    if key.relative_path.is_root() {
        key.root_alias.to_string()
    } else {
        format!("{}/{}", key.root_alias, key.relative_path)
    }
}

fn item_matches_filter(
    item: &TrackedItem,
    requested: Option<&Path>,
    requested_mapping: Option<&HostPathMapping>,
) -> bool {
    if requested.is_none() {
        return true;
    }
    item.local_path
        .as_deref()
        .is_some_and(|local_path| requested.is_some_and(|path| local_path.starts_with(path)))
        || requested_mapping.is_some_and(|mapping| {
            item.root_alias == mapping.root_alias.as_str()
                && relative_matches_prefix(
                    item.relative_path.as_str(),
                    mapping.relative_path.as_str(),
                )
        })
}

fn is_key_ignored(key: &EntryKey, ignored: &[DriveIgnoredPath]) -> bool {
    ignored.iter().any(|ignored| {
        ignored.space_id == key.space_id
            && ignored.root_alias == key.root_alias
            && relative_matches_prefix(key.relative_path.as_str(), ignored.relative_path.as_str())
    })
}

fn is_key_suspended(key: &EntryKey, suspended: &[DriveSuspendedPath]) -> bool {
    suspended.iter().any(|item| {
        item.space_id == key.space_id
            && item.root_alias == key.root_alias
            && item.relative_path == key.relative_path
    })
}

fn relative_matches_prefix(candidate: &str, prefix: &str) -> bool {
    prefix.is_empty() || candidate == prefix || candidate.starts_with(&format!("{prefix}/"))
}

fn tracked_item_order(left: &TrackedItem, right: &TrackedItem) -> std::cmp::Ordering {
    left.canonical_path
        .cmp(&right.canonical_path)
        .then_with(|| left.remote_path.cmp(&right.remote_path))
}

fn collect_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(root)
        .follow_links(false)
        .hidden(false)
        .ignore(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .require_git(false)
        .parents(true)
        .build();
    for entry in walker {
        let Ok(entry) = entry else {
            continue;
        };
        if entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn collect_gitignored_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let allowed = collect_files(root)?.into_iter().collect::<BTreeSet<_>>();
    let all = collect_files_without_ignore_rules(root)?;
    Ok(all
        .into_iter()
        .filter(|file| !allowed.contains(file))
        .collect())
}

fn collect_files_without_ignore_rules(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(root)
        .follow_links(false)
        .hidden(false)
        .ignore(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .require_git(false)
        .parents(false)
        .build();
    for entry in walker {
        let Ok(entry) = entry else {
            continue;
        };
        if entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn files_count_for_single_remote(is_dir: bool) -> bool {
    !is_dir
}

fn is_skippable_directory_host_error(error: &anyhow::Error) -> bool {
    error
        .chain()
        .find_map(|source| source.downcast_ref::<std::io::Error>())
        .is_some_and(|source| {
            matches!(
                source.kind(),
                std::io::ErrorKind::PermissionDenied
                    | std::io::ErrorKind::NotFound
                    | std::io::ErrorKind::Interrupted
            )
        })
}

async fn read_file(path: &Path) -> anyhow::Result<Vec<u8>> {
    tokio::fs::read(path)
        .await
        .with_context(|| format!("io error at {}", path.display()))
}

async fn write_file(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("io error at {}", parent.display()))?;
    }
    tokio::fs::write(path, bytes)
        .await
        .with_context(|| format!("io error at {}", path.display()))
}
