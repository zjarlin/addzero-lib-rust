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
    DriveConflict, DriveEntryKind, DriveMetadataStore, DriveObjectStore, DriveStoreError,
    DriveVersion,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;
use walkdir::WalkDir;

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
            device_id,
            device_name,
            poll_interval: Duration::from_secs(2),
        }
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

        let files = if metadata.is_dir() {
            collect_files(&requested)?
        } else {
            vec![requested.clone()]
        };

        let mut statuses = Vec::new();
        for file in files {
            let mut mapping = registry.resolve_host_path(&file, preferred_alias.as_ref())?;
            if let Some(remote_path) = remote_path
                && files_count_for_single_remote(metadata.is_dir())
            {
                mapping.relative_path = RelativePath::parse(remote_path)?;
            }
            let status = self.host_file(&mut state, mapping).await?;
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
        let before = state.hosted.len();
        state.hosted.retain(|hosted| {
            hosted.local_path != requested && !hosted.local_path.starts_with(&requested)
        });
        let removed = before.saturating_sub(state.hosted.len());
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

fn collect_files(root: &Path) -> DriveAgentResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|err| DriveAgentError::WalkDir(err.to_string()))?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn files_count_for_single_remote(is_dir: bool) -> bool {
    !is_dir
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
    use super::{DriveAgent, DriveAgentConfig, LocalStateStore};
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
}
