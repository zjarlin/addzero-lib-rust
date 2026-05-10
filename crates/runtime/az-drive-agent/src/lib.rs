#![forbid(unsafe_code)]

//! 无头实时网盘代理。
//!
//! 该守护进程在设计上不依赖 GUI。它通过轮询循环监听托管路径，
//! 对本地与远程版本进行对账，并自动写入冲突副本，
//! 无需手动的 Git 式人工干预。

use az_drive_core::{
    ChangeDecision, EntryKey, RelativePath, RootAlias, RootRegistry, conflict_file_name,
    content_hash, decide_local_change, expand_path_expression, normalize_absolute_path,
    object_key_for_hash, try_safe_text_merge,
};
use az_drive_store::{
    DriveConflict, DriveEntry, DriveEntryKind, DriveIgnoredPath, DriveMetadataStore,
    DriveObjectStore, DriveStoreError, DriveVersion,
};
use chrono::{DateTime, Utc};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

/// Result alias for agent operations.
pub type DriveAgentResult<T> = Result<T, DriveAgentError>;

/// Errors raised by the local drive agent.
#[derive(Debug, Error)]
pub enum DriveAgentError {
    /// Core path or conflict logic failed.
    #[error("drive core error: {0}")]
    Core(#[from] az_drive_core::DriveCoreError),
    /// Metadata or object store operation failed.
    #[error("drive store error: {0}")]
    Store(#[from] DriveStoreError),
    /// File-system operation failed with path context.
    #[error("io error at {path}: {source}")]
    Io {
        /// Path associated with the I/O failure.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Local state JSON could not be decoded.
    #[error("local state json error at {path}: {source}")]
    Json {
        /// State file path.
        path: PathBuf,
        /// Decode or encode error.
        #[source]
        source: serde_json::Error,
    },
    /// Directory walking failed.
    #[error("walkdir error: {0}")]
    WalkDir(String),
}

/// Agent configuration that is stable across CLI and future AIO embedding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveAgentConfig {
    /// Default drive space.
    pub space_id: String,
    /// Additional spaces visible for read-side fusion.
    pub fused_space_ids: Vec<String>,
    /// Local device id.
    pub device_id: String,
    /// Human-readable device name used in conflict copies.
    pub device_name: String,
    /// Poll interval for the daemon loop.
    pub poll_interval: Duration,
}

impl DriveAgentConfig {
    /// Creates a config with stable defaults.
    #[must_use]
    pub fn new(space_id: impl Into<String>, device_id: String, device_name: String) -> Self {
        Self {
            space_id: space_id.into(),
            fused_space_ids: Vec::new(),
            device_id,
            device_name,
            poll_interval: Duration::from_secs(2),
        }
    }

    /// Adds read-side fused spaces.
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

/// Local root persisted in the device-private state file.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalRootState {
    /// Logical root alias.
    pub alias: String,
    /// Device-local path.
    pub path: PathBuf,
}

/// Local hosted path mapping persisted on each device.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HostedPathState {
    /// Device-local absolute path.
    pub local_path: PathBuf,
    /// Remote space id.
    pub space_id: String,
    /// Remote root alias.
    pub root_alias: String,
    /// Remote relative path.
    pub relative_path: String,
    /// Last synchronized remote version.
    pub base_version: Option<u64>,
    /// Last synchronized content hash.
    pub base_hash: Option<String>,
    /// Last observed local content hash.
    pub content_hash: Option<String>,
    /// Hosting creation time.
    pub hosted_at: DateTime<Utc>,
    /// Last successful synchronization.
    pub last_synced_at: Option<DateTime<Utc>>,
}

/// Local directory root whose descendants are hosted for synchronization.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HostedRootState {
    /// Device-local absolute directory path.
    pub local_path: PathBuf,
    /// Remote space id.
    pub space_id: String,
    /// Remote root alias.
    pub root_alias: String,
    /// Remote relative path for the hosted directory.
    pub relative_path: String,
    /// Hosting creation time.
    pub hosted_at: DateTime<Utc>,
}

/// Device-local conflict projection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalConflictState {
    /// Server conflict id.
    pub id: Uuid,
    /// Local conflict copy path.
    pub conflict_path: PathBuf,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Device-private state. This file is not server truth.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalState {
    /// State schema version.
    pub state_version: u32,
    /// Stable device id.
    pub device_id: String,
    /// Human-readable device name.
    pub device_name: String,
    /// Local root mappings.
    pub roots: Vec<LocalRootState>,
    /// Hosted local paths.
    pub hosted: Vec<HostedPathState>,
    /// Hosted directory roots whose descendants are discovered on sync.
    #[serde(default)]
    pub hosted_roots: Vec<HostedRootState>,
    /// Locally observed conflicts.
    pub conflicts: Vec<LocalConflictState>,
}

impl LocalState {
    /// Creates a new local state document.
    #[must_use]
    pub fn new(device_name: String) -> Self {
        Self {
            state_version: 1,
            device_id: Uuid::new_v4().to_string(),
            device_name,
            roots: Vec::new(),
            hosted: Vec::new(),
            hosted_roots: Vec::new(),
            conflicts: Vec::new(),
        }
    }
}

/// Summary for CLI status output.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HostedStatus {
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackedItemStatus {
    /// Local item is tracked and currently exists.
    Tracked,
    /// Local item is tracked but missing on disk.
    MissingLocal,
    /// Remote item has no local tracked counterpart on this device.
    RemoteOnly,
    /// Item is excluded by an ignore rule.
    Ignored,
    /// Hosted directory root.
    Root,
}

/// Provenance for a tracked listing row.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRemoteStatus {
    /// Remote bytes were written locally and the file is now hosted.
    Pulled,
    /// Local file already matched the remote version and was marked hosted.
    AlreadyCurrent,
    /// A local file exists with different content and `overwrite` was not set.
    SkippedExisting,
    /// The row is ignored by a shared database ignore rule.
    SkippedIgnored,
    /// The row has no downloadable remote version.
    SkippedNoVersion,
    /// The command only reported what would happen.
    DryRun,
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    /// Last remote version.
    pub base_version: Option<u64>,
    /// Last remote content hash.
    pub content_hash: Option<String>,
    /// Whether the local file exists after the operation.
    pub exists: bool,
}

struct ConflictRestoreRequest<'a> {
    local_path: &'a Path,
    entry_id: Uuid,
    base_version: Option<u64>,
    local_hash: &'a str,
    remote: &'a DriveVersion,
    local_bytes: &'a [u8],
}

/// File-backed local state store.
#[derive(Clone, Debug)]
pub struct LocalStateStore {
    path: PathBuf,
}

impl LocalStateStore {
    /// Creates a state store at an explicit path.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Resolves the default state file path.
    #[must_use]
    pub fn default_path() -> PathBuf {
        if let Some(path) = std::env::var_os("AZ_DRIVE_STATE") {
            return PathBuf::from(path);
        }
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("addzero")
            .join("drive")
            .join("state.json")
    }

    /// Returns the state file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads state, creating a new in-memory document when the file is absent.
    ///
    /// # Errors
    /// Returns [`DriveAgentError`] when file reading or JSON decoding fails.
    pub async fn load_or_init(&self) -> DriveAgentResult<LocalState> {
        match tokio::fs::read(&self.path).await {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|source| DriveAgentError::Json {
                path: self.path.clone(),
                source,
            }),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                Ok(LocalState::new(default_device_name()))
            }
            Err(source) => Err(DriveAgentError::Io {
                path: self.path.clone(),
                source,
            }),
        }
    }

    /// Saves state atomically enough for a single local daemon.
    ///
    /// # Errors
    /// Returns [`DriveAgentError`] when parent creation, JSON encoding, or write
    /// fails.
    pub async fn save(&self, state: &LocalState) -> DriveAgentResult<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|source| DriveAgentError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
        }
        let bytes = serde_json::to_vec_pretty(state).map_err(|source| DriveAgentError::Json {
            path: self.path.clone(),
            source,
        })?;
        tokio::fs::write(&self.path, bytes)
            .await
            .map_err(|source| DriveAgentError::Io {
                path: self.path.clone(),
                source,
            })
    }
}

/// Headless realtime drive agent.
#[derive(Clone)]
pub struct DriveAgent {
    metadata: Arc<dyn DriveMetadataStore>,
    objects: Arc<dyn DriveObjectStore>,
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
            state_store,
            config,
        }
    }

    /// Loads local state.
    ///
    /// # Errors
    /// Returns [`DriveAgentError`] when state loading fails.
    pub async fn state(&self) -> DriveAgentResult<LocalState> {
        self.state_store.load_or_init().await
    }

    /// Adds a local root alias.
    ///
    /// # Errors
    /// Returns [`DriveAgentError`] when the alias/path is invalid or state save fails.
    pub async fn add_root(&self, alias: &str, path: &str) -> DriveAgentResult<Vec<LocalRootState>> {
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
    /// Returns [`DriveAgentError`] when local state or home root resolution fails.
    pub async fn list_roots(&self) -> DriveAgentResult<Vec<LocalRootState>> {
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
    /// Returns [`DriveAgentError`] when path mapping, local I/O, or remote store
    /// operations fail.
    pub async fn host_path(
        &self,
        path: &str,
        root_alias: Option<&str>,
        remote_path: Option<&str>,
    ) -> DriveAgentResult<Vec<HostedStatus>> {
        let mut state = self.state_store.load_or_init().await?;
        let registry = registry_from_state(&state)?;
        let preferred_alias = root_alias.map(RootAlias::parse).transpose()?;
        let requested = normalize_absolute_path(&expand_path_expression(path))?;
        let metadata =
            tokio::fs::metadata(&requested)
                .await
                .map_err(|source| DriveAgentError::Io {
                    path: requested.clone(),
                    source,
                })?;

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
        Ok(statuses)
    }

    /// Cancels local hosting without deleting local or remote content.
    ///
    /// # Errors
    /// Returns [`DriveAgentError`] when state loading or saving fails.
    pub async fn unhost_path(&self, path: &str) -> DriveAgentResult<usize> {
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
        Ok(removed)
    }

    /// Returns hosted status records.
    ///
    /// # Errors
    /// Returns [`DriveAgentError`] when state loading fails.
    pub async fn status(&self, path: Option<&str>) -> DriveAgentResult<Vec<HostedStatus>> {
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
    /// Returns [`DriveAgentError`] when local state, directory walking, path
    /// mapping, or metadata queries fail.
    pub async fn list_tracked(
        &self,
        path: Option<&str>,
        options: ListTrackedOptions,
    ) -> DriveAgentResult<Vec<TrackedItem>> {
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
    /// Returns [`DriveAgentError`] when state, root mapping, metadata, object
    /// storage, or local file writes fail.
    pub async fn pull_remote(
        &self,
        path: Option<&str>,
        options: PullRemoteOptions,
    ) -> DriveAgentResult<Vec<PullRemoteItem>> {
        let mut state = self.state_store.load_or_init().await?;
        let registry = registry_from_state(&state)?;
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
            if item_filtered_out(&entry.key, requested_mapping.as_ref()) {
                continue;
            }
            let Some(local_path) = local_path_for_key(&registry, &entry.key) else {
                continue;
            };
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

            let remote_bytes = self.objects.get_object(&remote.object_key).await?;
            let status = if local_path.exists() {
                let local_bytes = read_file(&local_path).await?;
                if content_hash(&local_bytes) == remote.content_hash {
                    PullRemoteStatus::AlreadyCurrent
                } else if options.overwrite {
                    write_file(&local_path, &remote_bytes).await?;
                    PullRemoteStatus::Pulled
                } else {
                    rows.push(pull_item_from_entry(
                        PullRemoteStatus::SkippedExisting,
                        local_path,
                        &entry,
                        &registry,
                        Some(&remote),
                    ));
                    continue;
                }
            } else {
                write_file(&local_path, &remote_bytes).await?;
                PullRemoteStatus::Pulled
            };

            upsert_hosted_state(
                &mut state,
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
            rows.push(pull_item_from_entry(
                status,
                local_path,
                &entry,
                &registry,
                Some(&remote),
            ));
        }

        if !options.dry_run {
            self.state_store.save(&state).await?;
        }
        rows.sort_by(|left, right| {
            left.canonical_path
                .cmp(&right.canonical_path)
                .then_with(|| left.remote_path.cmp(&right.remote_path))
        });
        Ok(rows)
    }

    async fn list_visible_entries(&self) -> DriveAgentResult<Vec<DriveEntry>> {
        let mut entries = Vec::new();
        for space_id in self.config.visible_space_ids() {
            entries.extend(self.metadata.list_entries_by_space(&space_id).await?);
        }
        Ok(entries)
    }

    async fn list_visible_ignored_paths(&self) -> DriveAgentResult<Vec<DriveIgnoredPath>> {
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

    /// Lists unresolved conflicts from the server-side metadata store.
    ///
    /// # Errors
    /// Returns [`DriveAgentError`] when store access fails.
    pub async fn conflicts(&self) -> DriveAgentResult<Vec<DriveConflict>> {
        Ok(self.metadata.list_conflicts(Some(false)).await?)
    }

    /// Performs one synchronization scan for all hosted paths.
    ///
    /// # Errors
    /// Returns [`DriveAgentError`] when local I/O or remote store operations fail.
    pub async fn sync_once(&self) -> DriveAgentResult<Vec<HostedStatus>> {
        let mut state = self.state_store.load_or_init().await?;
        self.discover_hosted_root_files(&mut state).await?;
        let mut statuses = Vec::new();
        let mut hosted_records = std::mem::take(&mut state.hosted);
        for mut hosted in hosted_records.drain(..) {
            self.sync_hosted(&mut state, &mut hosted).await?;
            statuses.push(hosted_status(&hosted));
            state.hosted.push(hosted);
        }
        self.state_store.save(&state).await?;
        Ok(statuses)
    }

    /// Runs the polling realtime daemon until interrupted.
    ///
    /// # Errors
    /// Returns [`DriveAgentError`] if a sync cycle fails.
    pub async fn run_polling_daemon(&self) -> DriveAgentResult<()> {
        loop {
            self.sync_once().await?;
            tokio::time::sleep(self.config.poll_interval).await;
        }
    }

    async fn host_file(
        &self,
        state: &mut LocalState,
        mapping: az_drive_core::HostPathMapping,
    ) -> DriveAgentResult<HostedStatus> {
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
    ) -> DriveAgentResult<()> {
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
        let latest = self.metadata.latest_version(entry.id).await?;
        let local_bytes = read_file(&hosted.local_path).await?;
        let local_hash = content_hash(&local_bytes);

        if let Some(remote) = &latest
            && hosted.content_hash.as_deref() == Some(&local_hash)
            && hosted
                .base_version
                .is_some_and(|base| remote.version > base)
        {
            let remote_bytes = self.objects.get_object(&remote.object_key).await?;
            write_file(&hosted.local_path, &remote_bytes).await?;
            hosted.base_version = Some(remote.version);
            hosted.base_hash = Some(remote.content_hash.clone());
            hosted.content_hash = Some(remote.content_hash.clone());
            hosted.last_synced_at = Some(Utc::now());
            return Ok(());
        }

        if hosted.content_hash.as_deref() == Some(&local_hash) {
            return Ok(());
        }

        let remote_version = latest.as_ref().map(|version| version.version);
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
                let object_key = object_key_for_hash(&local_hash);
                if !self.objects.object_exists(&object_key).await? {
                    self.objects.put_object(&object_key, &local_bytes).await?;
                }
                let next_version = remote_version.unwrap_or(0).saturating_add(1);
                let version = DriveVersion {
                    id: Uuid::new_v4(),
                    entry_id: entry.id,
                    version: next_version,
                    content_hash: local_hash.clone(),
                    object_key,
                    size_bytes: local_bytes.len() as u64,
                    device_id: self.config.device_id.clone(),
                    modified_at: Utc::now(),
                };
                self.metadata.insert_version(version).await?;
                hosted.base_version = Some(next_version);
                hosted.base_hash = Some(local_hash.clone());
                hosted.content_hash = Some(local_hash);
                hosted.last_synced_at = Some(Utc::now());
            }
            ChangeDecision::Conflict => {
                if let Some(remote) = latest {
                    self.try_merge_or_write_conflict(state, hosted, entry.id, remote, &local_bytes)
                        .await?;
                }
            }
            ChangeDecision::LockedByOther { .. } => {}
        }
        Ok(())
    }

    async fn discover_hosted_root_files(&self, state: &mut LocalState) -> DriveAgentResult<()> {
        let registry = registry_from_state(state)?;
        let roots = state.hosted_roots.clone();
        let ignored = self
            .metadata
            .list_ignored_paths(&self.config.space_id, None, None)
            .await?;
        for root in roots {
            if !root.local_path.exists() {
                continue;
            }
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
    ) -> DriveAgentResult<()> {
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
    ) -> DriveAgentResult<()> {
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
        state.conflicts.push(LocalConflictState {
            id: conflict.id,
            conflict_path,
            created_at: now,
        });
        Ok(())
    }
}

fn registry_from_state(state: &LocalState) -> DriveAgentResult<RootRegistry> {
    let mut registry = RootRegistry::default_for_device()?;
    for root in &state.roots {
        registry.add_root(RootAlias::parse(&root.alias)?, &root.path)?;
    }
    Ok(registry)
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
    requested_mapping: Option<&az_drive_core::HostPathMapping>,
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
    requested: Option<&Path>,
    requested_mapping: Option<&az_drive_core::HostPathMapping>,
) -> DriveAgentResult<()> {
    let mut hosted_paths = BTreeSet::new();
    for hosted in &state.hosted {
        hosted_paths.insert(hosted.local_path.clone());
        let key = key_from_hosted(hosted)?;
        if is_key_ignored(&key, ignored) {
            continue;
        }
        let status = if hosted.local_path.exists() {
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
                TrackedItemStatus::Tracked,
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
    requested: Option<&Path>,
    requested_mapping: Option<&az_drive_core::HostPathMapping>,
) {
    for entry in entries {
        if is_key_ignored(&entry.key, ignored) {
            continue;
        }
        let remote_path = entry.key.remote_path();
        if let Some(existing) = rows.get_mut(&remote_path) {
            existing.source = TrackedItemSource::Both;
            existing.base_version = existing.base_version.or(Some(entry.latest_version));
            if existing.content_hash.is_none() {
                existing.content_hash.clone_from(&entry.latest_hash);
            }
            continue;
        }

        let item = tracked_item_from_key(
            TrackedItemStatus::RemoteOnly,
            TrackedItemSource::Remote,
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
    requested_mapping: Option<&az_drive_core::HostPathMapping>,
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
    requested_mapping: Option<&az_drive_core::HostPathMapping>,
) -> DriveAgentResult<()> {
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
    requested_mapping: Option<&az_drive_core::HostPathMapping>,
) -> bool {
    requested_mapping.is_some_and(|mapping| {
        key.root_alias != mapping.root_alias
            || !relative_matches_prefix(key.relative_path.as_str(), mapping.relative_path.as_str())
    })
}

fn key_from_hosted(hosted: &HostedPathState) -> DriveAgentResult<EntryKey> {
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
    requested_mapping: Option<&az_drive_core::HostPathMapping>,
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

fn relative_matches_prefix(candidate: &str, prefix: &str) -> bool {
    prefix.is_empty() || candidate == prefix || candidate.starts_with(&format!("{prefix}/"))
}

fn tracked_item_order(left: &TrackedItem, right: &TrackedItem) -> std::cmp::Ordering {
    left.canonical_path
        .cmp(&right.canonical_path)
        .then_with(|| left.remote_path.cmp(&right.remote_path))
}

fn collect_files(root: &Path) -> DriveAgentResult<Vec<PathBuf>> {
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

fn collect_gitignored_files(root: &Path) -> DriveAgentResult<Vec<PathBuf>> {
    let allowed = collect_files(root)?.into_iter().collect::<BTreeSet<_>>();
    let all = collect_files_without_ignore_rules(root)?;
    Ok(all
        .into_iter()
        .filter(|file| !allowed.contains(file))
        .collect())
}

fn collect_files_without_ignore_rules(root: &Path) -> DriveAgentResult<Vec<PathBuf>> {
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

fn is_skippable_directory_host_error(error: &DriveAgentError) -> bool {
    matches!(
        error,
        DriveAgentError::Io { source, .. }
            if matches!(
                source.kind(),
                ErrorKind::PermissionDenied | ErrorKind::NotFound | ErrorKind::Interrupted
            )
    )
}

async fn read_file(path: &Path) -> DriveAgentResult<Vec<u8>> {
    tokio::fs::read(path)
        .await
        .map_err(|source| DriveAgentError::Io {
            path: path.to_path_buf(),
            source,
        })
}

async fn write_file(path: &Path, bytes: &[u8]) -> DriveAgentResult<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|source| DriveAgentError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
    }
    tokio::fs::write(path, bytes)
        .await
        .map_err(|source| DriveAgentError::Io {
            path: path.to_path_buf(),
            source,
        })
}

fn default_device_name() -> String {
    std::env::var("AZ_DRIVE_DEVICE_NAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "device".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        DriveAgent, DriveAgentConfig, ListTrackedOptions, LocalStateStore, PullRemoteOptions,
        PullRemoteStatus, TrackedItemSource, TrackedItemStatus, display_path_for_key,
    };
    use az_drive_core::{EntryKey, RelativePath, RootAlias, RootRegistry};
    use az_drive_store::{InMemoryDriveMetadataStore, InMemoryDriveObjectStore};
    use std::sync::Arc;
    use tempfile::TempDir;

    fn agent(
        temp: &TempDir,
        name: &str,
        metadata: Arc<InMemoryDriveMetadataStore>,
        objects: Arc<InMemoryDriveObjectStore>,
    ) -> DriveAgent {
        DriveAgent::new(
            metadata,
            objects,
            LocalStateStore::new(temp.path().join(format!("{name}.json"))),
            DriveAgentConfig::new("main", format!("device-{name}"), name.to_owned()),
        )
    }

    #[tokio::test]
    async fn host_path_tracks_relative_path_below_root() {
        let temp = TempDir::new().expect("temp dir should exist");
        let metadata = Arc::new(InMemoryDriveMetadataStore::new());
        let objects = Arc::new(InMemoryDriveObjectStore::new());
        let agent = agent(&temp, "a", metadata, objects);
        let root = temp.path().join("workspace");
        let file = root.join("docs/a.md");
        tokio::fs::create_dir_all(file.parent().expect("file should have parent"))
            .await
            .expect("parent should be created");
        tokio::fs::write(&file, b"hello")
            .await
            .expect("file should be written");
        agent
            .add_root("workspace", root.to_str().expect("utf8 path"))
            .await
            .expect("root should add");

        let statuses = agent
            .host_path(file.to_str().expect("utf8 path"), None, None)
            .await
            .expect("file should host");

        assert_eq!(statuses[0].remote_path, "main/workspace/docs/a.md");
    }

    #[tokio::test]
    async fn unhost_path_keeps_local_file() {
        let temp = TempDir::new().expect("temp dir should exist");
        let metadata = Arc::new(InMemoryDriveMetadataStore::new());
        let objects = Arc::new(InMemoryDriveObjectStore::new());
        let agent = agent(&temp, "a", metadata, objects);
        let file = temp.path().join("a.txt");
        tokio::fs::write(&file, b"hello")
            .await
            .expect("file should be written");
        agent
            .add_root("workspace", temp.path().to_str().expect("utf8 path"))
            .await
            .expect("root should add");

        agent
            .host_path(file.to_str().expect("utf8 path"), None, Some("a.txt"))
            .await
            .expect("file should host");
        let removed = agent
            .unhost_path(file.to_str().expect("utf8 path"))
            .await
            .expect("file should unhost");

        assert_eq!(removed, 1);
        assert!(file.exists());
    }

    #[tokio::test]
    async fn hosted_directory_discovers_children_and_respects_gitignore() {
        let temp = TempDir::new().expect("temp dir should exist");
        let metadata = Arc::new(InMemoryDriveMetadataStore::new());
        let objects = Arc::new(InMemoryDriveObjectStore::new());
        let agent = agent(&temp, "a", metadata, objects);
        let root = temp.path().join("workspace");
        let visible = root.join("docs/a.md");
        let ignored_dir_file = root.join("target/generated.txt");
        let ignored_glob_file = root.join("notes/debug.log");
        tokio::fs::create_dir_all(visible.parent().expect("visible parent"))
            .await
            .expect("visible parent should be created");
        tokio::fs::create_dir_all(ignored_dir_file.parent().expect("ignored dir parent"))
            .await
            .expect("ignored dir parent should be created");
        tokio::fs::create_dir_all(ignored_glob_file.parent().expect("ignored glob parent"))
            .await
            .expect("ignored glob parent should be created");
        tokio::fs::write(root.join(".gitignore"), b"target/\n*.log\n")
            .await
            .expect("gitignore should be written");
        tokio::fs::write(&visible, b"visible")
            .await
            .expect("visible file should be written");
        tokio::fs::write(&ignored_dir_file, b"ignored")
            .await
            .expect("ignored dir file should be written");
        tokio::fs::write(&ignored_glob_file, b"ignored")
            .await
            .expect("ignored glob file should be written");
        agent
            .add_root("workspace", root.to_str().expect("utf8 path"))
            .await
            .expect("root should add");

        agent
            .host_path(root.to_str().expect("utf8 path"), None, None)
            .await
            .expect("directory should host");

        let initial = agent.status(None).await.expect("status should load");
        // The hosted directory must include normal descendants and exclude gitignored paths.
        assert!(initial.iter().any(|status| status.local_path == visible));
        assert!(
            initial
                .iter()
                .all(|status| status.local_path != ignored_dir_file)
        );
        assert!(
            initial
                .iter()
                .all(|status| status.local_path != ignored_glob_file)
        );

        let new_file = root.join("docs/new.md");
        tokio::fs::write(&new_file, b"new")
            .await
            .expect("new file should be written");
        agent
            .sync_once()
            .await
            .expect("sync should discover new file");
        let after_sync = agent.status(None).await.expect("status should reload");

        // A hosted directory root is persistent, so later non-ignored children become hosted.
        assert!(
            after_sync
                .iter()
                .any(|status| status.local_path == new_file)
        );
    }

    #[tokio::test]
    async fn list_tracked_reports_gitignored_paths_only_when_requested() {
        let temp = TempDir::new().expect("temp dir should exist");
        let metadata = Arc::new(InMemoryDriveMetadataStore::new());
        let objects = Arc::new(InMemoryDriveObjectStore::new());
        let agent = agent(&temp, "a", metadata, objects);
        let root = temp.path().join("workspace");
        let visible = root.join("docs/a.md");
        let ignored = root.join("docs/debug.log");
        tokio::fs::create_dir_all(visible.parent().expect("visible parent"))
            .await
            .expect("visible parent should be created");
        tokio::fs::write(root.join(".gitignore"), b"*.log\n")
            .await
            .expect("gitignore should be written");
        tokio::fs::write(&visible, b"visible")
            .await
            .expect("visible file should be written");
        tokio::fs::write(&ignored, b"ignored")
            .await
            .expect("ignored file should be written");
        agent
            .add_root("workspace", root.to_str().expect("utf8 path"))
            .await
            .expect("root should add");
        agent
            .host_path(root.to_str().expect("utf8 path"), None, None)
            .await
            .expect("directory should host");

        let listed = agent
            .list_tracked(None, ListTrackedOptions::default())
            .await
            .expect("tracked list should load");
        // Default listing must match sync behavior and hide gitignored files.
        assert!(
            listed
                .iter()
                .all(|item| item.local_path.as_ref() != Some(&ignored))
        );

        let listed_with_ignored = agent
            .list_tracked(
                None,
                ListTrackedOptions {
                    include_ignored: true,
                    ..ListTrackedOptions::default()
                },
            )
            .await
            .expect("tracked list with ignored should load");

        assert!(listed_with_ignored.iter().any(|item| {
            item.local_path.as_ref() == Some(&ignored)
                && item.status == TrackedItemStatus::Ignored
                && item.source == TrackedItemSource::Gitignore
        }));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn hosted_directory_skips_unreadable_paths() {
        let temp = TempDir::new().expect("temp dir should exist");
        let metadata = Arc::new(InMemoryDriveMetadataStore::new());
        let objects = Arc::new(InMemoryDriveObjectStore::new());
        let agent = agent(&temp, "a", metadata, objects);
        let root = temp.path().join("workspace");
        let visible = root.join("docs/a.md");
        let unreadable = root.join("docs/private.md");
        let blocked_dir = root.join("Library/Blocked");
        tokio::fs::create_dir_all(visible.parent().expect("visible parent"))
            .await
            .expect("visible parent should be created");
        tokio::fs::create_dir_all(&blocked_dir)
            .await
            .expect("blocked dir should be created");
        tokio::fs::write(&visible, b"visible")
            .await
            .expect("visible file should be written");
        tokio::fs::write(&unreadable, b"private")
            .await
            .expect("unreadable file should be written");
        tokio::fs::write(blocked_dir.join("secret.md"), b"secret")
            .await
            .expect("blocked file should be written");
        std::fs::set_permissions(
            &unreadable,
            std::os::unix::fs::PermissionsExt::from_mode(0o000),
        )
        .expect("unreadable file permissions should change");
        std::fs::set_permissions(
            &blocked_dir,
            std::os::unix::fs::PermissionsExt::from_mode(0o000),
        )
        .expect("blocked dir permissions should change");
        agent
            .add_root("workspace", root.to_str().expect("utf8 path"))
            .await
            .expect("root should add");

        let statuses = agent
            .host_path(root.to_str().expect("utf8 path"), None, None)
            .await
            .expect("directory should host readable descendants");

        std::fs::set_permissions(
            &unreadable,
            std::os::unix::fs::PermissionsExt::from_mode(0o600),
        )
        .expect("unreadable file permissions should restore");
        std::fs::set_permissions(
            &blocked_dir,
            std::os::unix::fs::PermissionsExt::from_mode(0o700),
        )
        .expect("blocked dir permissions should restore");
        // Protected home-directory descendants must not abort the whole directory host.
        assert!(statuses.iter().any(|status| status.local_path == visible));
    }

    #[tokio::test]
    async fn unhost_child_under_hosted_root_creates_shared_ignore_rule() {
        let temp = TempDir::new().expect("temp dir should exist");
        let metadata = Arc::new(InMemoryDriveMetadataStore::new());
        let objects = Arc::new(InMemoryDriveObjectStore::new());
        let agent = agent(&temp, "a", metadata, objects);
        let root = temp.path().join("workspace");
        let visible = root.join("docs/a.md");
        let ignored_dir = root.join("private");
        let ignored_file = ignored_dir.join("secret.md");
        tokio::fs::create_dir_all(visible.parent().expect("visible parent"))
            .await
            .expect("visible parent should be created");
        tokio::fs::create_dir_all(&ignored_dir)
            .await
            .expect("ignored dir should be created");
        tokio::fs::write(&visible, b"visible")
            .await
            .expect("visible file should be written");
        tokio::fs::write(&ignored_file, b"secret")
            .await
            .expect("ignored file should be written");
        agent
            .add_root("workspace", root.to_str().expect("utf8 path"))
            .await
            .expect("root should add");
        agent
            .host_path(root.to_str().expect("utf8 path"), None, None)
            .await
            .expect("directory should host");

        let removed = agent
            .unhost_path(ignored_dir.to_str().expect("utf8 path"))
            .await
            .expect("child should unhost");

        assert!(removed > 0);
        let listed = agent
            .list_tracked(None, ListTrackedOptions::default())
            .await
            .expect("tracked list should load");
        // A DB ignore rule must prevent the child from being rediscovered.
        assert!(
            listed
                .iter()
                .all(|item| item.local_path.as_ref() != Some(&ignored_file))
        );

        let ignored = agent
            .list_tracked(
                None,
                ListTrackedOptions {
                    include_ignored: true,
                    ..ListTrackedOptions::default()
                },
            )
            .await
            .expect("ignored list should load");

        assert!(ignored.iter().any(|item| {
            item.canonical_path == "workspace/private"
                && item.status == TrackedItemStatus::Ignored
                && item.source == TrackedItemSource::DbIgnore
        }));
    }

    #[tokio::test]
    async fn list_tracked_uses_home_canonical_path_across_devices() {
        let temp = TempDir::new().expect("temp dir should exist");
        let metadata = Arc::new(InMemoryDriveMetadataStore::new());
        let objects = Arc::new(InMemoryDriveObjectStore::new());
        let agent_a = agent(&temp, "a", Arc::clone(&metadata), Arc::clone(&objects));
        let agent_b = agent(&temp, "b", metadata, objects);
        let home_a = temp.path().join("home-a");
        let home_b = temp.path().join("home-b");
        let file_a = home_a.join(".agents/skills/demo/SKILL.md");
        let file_b = home_b.join(".agents/skills/demo/SKILL.md");
        tokio::fs::create_dir_all(file_a.parent().expect("file a parent"))
            .await
            .expect("file a parent should be created");
        tokio::fs::create_dir_all(file_b.parent().expect("file b parent"))
            .await
            .expect("file b parent should be created");
        tokio::fs::write(&file_a, b"skill")
            .await
            .expect("file a should be written");
        tokio::fs::write(&file_b, b"skill")
            .await
            .expect("file b should be written");
        agent_a
            .add_root("home", home_a.to_str().expect("utf8 path"))
            .await
            .expect("home a should add");
        agent_b
            .add_root("home", home_b.to_str().expect("utf8 path"))
            .await
            .expect("home b should add");
        agent_a
            .host_path(file_a.to_str().expect("utf8 path"), None, None)
            .await
            .expect("file a should host");
        agent_b
            .host_path(file_b.to_str().expect("utf8 path"), None, None)
            .await
            .expect("file b should host");

        let listed_a = agent_a
            .list_tracked(None, ListTrackedOptions::default())
            .await
            .expect("agent a list should load");
        let listed_b = agent_b
            .list_tracked(None, ListTrackedOptions::default())
            .await
            .expect("agent b list should load");

        assert_eq!(listed_a[0].canonical_path, listed_b[0].canonical_path);
    }

    #[test]
    fn display_path_supports_absolute_macos_root() {
        let mut registry = RootRegistry::default();
        registry
            .add_root(RootAlias::parse("macos").expect("alias should parse"), "/")
            .expect("macos root should add");
        let key = EntryKey::new(
            "main",
            RootAlias::parse("macos").expect("alias should parse"),
            RelativePath::parse("Library/Application Support/demo").expect("path should parse"),
        );

        assert_eq!(
            display_path_for_key(&key, &registry),
            "/Library/Application Support/demo"
        );
    }

    #[tokio::test]
    async fn sync_once_creates_conflict_copy_for_stale_local_change() {
        let temp = TempDir::new().expect("temp dir should exist");
        let metadata = Arc::new(InMemoryDriveMetadataStore::new());
        let objects = Arc::new(InMemoryDriveObjectStore::new());
        let agent_a = agent(&temp, "a", Arc::clone(&metadata), Arc::clone(&objects));
        let agent_b = agent(&temp, "b", metadata, objects);
        let root_a = temp.path().join("a-root");
        let root_b = temp.path().join("b-root");
        let file_a = root_a.join("same.txt");
        let file_b = root_b.join("same.txt");
        tokio::fs::create_dir_all(&root_a).await.expect("root a");
        tokio::fs::create_dir_all(&root_b).await.expect("root b");
        tokio::fs::write(&file_a, b"base").await.expect("file a");
        tokio::fs::write(&file_b, b"base").await.expect("file b");
        agent_a
            .add_root("workspace", root_a.to_str().expect("utf8 path"))
            .await
            .expect("root a should add");
        agent_b
            .add_root("workspace", root_b.to_str().expect("utf8 path"))
            .await
            .expect("root b should add");
        agent_a
            .host_path(file_a.to_str().expect("utf8 path"), None, None)
            .await
            .expect("a should host");
        agent_b
            .host_path(file_b.to_str().expect("utf8 path"), None, None)
            .await
            .expect("b should host");

        tokio::fs::write(&file_a, b"from-a").await.expect("edit a");
        agent_a.sync_once().await.expect("a sync");
        tokio::fs::write(&file_b, b"from-b").await.expect("edit b");
        agent_b.sync_once().await.expect("b sync");
        let conflicts = agent_b.conflicts().await.expect("conflicts should load");

        assert_eq!(conflicts.len(), 1);
    }

    #[tokio::test]
    async fn pull_remote_materializes_entries_under_device_root() {
        let temp = TempDir::new().expect("temp dir should exist");
        let metadata = Arc::new(InMemoryDriveMetadataStore::new());
        let objects = Arc::new(InMemoryDriveObjectStore::new());
        let agent_a = agent(&temp, "a", Arc::clone(&metadata), Arc::clone(&objects));
        let agent_b = agent(&temp, "b", metadata, objects);
        let root_a = temp.path().join("a-root");
        let root_b = temp.path().join("b-root");
        let file_a = root_a.join("skills/demo/SKILL.md");
        tokio::fs::create_dir_all(file_a.parent().expect("file should have parent"))
            .await
            .expect("parent should be created");
        tokio::fs::write(&file_a, b"skill from a")
            .await
            .expect("source file should be written");
        agent_a
            .add_root("workspace", root_a.to_str().expect("utf8 path"))
            .await
            .expect("root a should add");
        agent_b
            .add_root("workspace", root_b.to_str().expect("utf8 path"))
            .await
            .expect("root b should add");
        agent_a
            .host_path(file_a.to_str().expect("utf8 path"), None, None)
            .await
            .expect("source file should host");

        let pulled = agent_b
            .pull_remote(None, PullRemoteOptions::default())
            .await
            .expect("remote should pull");

        let file_b = root_b.join("skills/demo/SKILL.md");
        assert_eq!(pulled[0].status, PullRemoteStatus::Pulled);
        assert_eq!(
            tokio::fs::read_to_string(&file_b)
                .await
                .expect("pulled file should exist"),
            "skill from a"
        );
        assert!(
            agent_b
                .status(None)
                .await
                .expect("status should load")
                .iter()
                .any(|status| status.local_path == file_b)
        );
    }
}
