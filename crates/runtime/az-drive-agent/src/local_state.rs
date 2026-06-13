use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use az_derive_aliases::{apply, plain_clone_debug, serde_eq};
use chrono::{DateTime, Utc};
use fs2::FileExt;
use uuid::Uuid;

/// Local root persisted in the device-private state file.
#[apply(serde_eq)]
pub struct LocalRootState {
    /// Logical root alias.
    pub alias: String,
    /// Device-local path.
    pub path: PathBuf,
}

/// Local hosted path mapping persisted on each device.
#[apply(serde_eq)]
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
#[apply(serde_eq)]
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
#[apply(serde_eq)]
pub struct LocalConflictState {
    /// Server conflict id.
    pub id: Uuid,
    /// Local conflict copy path.
    pub conflict_path: PathBuf,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Device-private state. This file is not server truth.
#[apply(serde_eq)]
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

/// File-backed local state store.
#[apply(plain_clone_debug)]
pub struct LocalStateStore {
    path: PathBuf,
}

pub(crate) struct LocalStateWriteGuard {
    file: std::fs::File,
}

impl Drop for LocalStateWriteGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
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

    fn lock_path(&self) -> PathBuf {
        let name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .map_or_else(
                || "state.json.lock".to_owned(),
                |name| format!("{name}.lock"),
            );
        self.path.with_file_name(name)
    }

    pub(crate) fn acquire_write_lock(&self) -> Result<LocalStateWriteGuard> {
        let lock_path = self.lock_path();
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("io error at {}", parent.display()))?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .with_context(|| format!("io error at {}", lock_path.display()))?;
        file.lock_exclusive()
            .with_context(|| format!("io error at {}", lock_path.display()))?;
        Ok(LocalStateWriteGuard { file })
    }

    /// Loads state, creating a new in-memory document when the file is absent.
    ///
    /// # Errors
    /// Returns an error when file reading or JSON decoding fails.
    pub async fn load_or_init(&self) -> Result<LocalState> {
        match tokio::fs::read(&self.path).await {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .with_context(|| format!("local state json error at {}", self.path.display())),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                Ok(LocalState::new(default_device_name()))
            }
            Err(source) => {
                Err(source).with_context(|| format!("io error at {}", self.path.display()))
            }
        }
    }

    /// Saves state via side-file replacement so concurrent readers never see a
    /// truncated JSON document.
    ///
    /// # Errors
    /// Returns an error when parent creation, JSON encoding, or write
    /// fails.
    pub async fn save(&self, state: &LocalState) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("io error at {}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(state)
            .with_context(|| format!("local state json error at {}", self.path.display()))?;
        let tmp_path = self.path.with_file_name(format!(
            ".{}.{}.tmp",
            self.path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("state.json"),
            std::process::id()
        ));
        tokio::fs::write(&tmp_path, bytes)
            .await
            .with_context(|| format!("io error at {}", tmp_path.display()))?;
        tokio::fs::rename(&tmp_path, &self.path)
            .await
            .with_context(|| format!("io error at {}", self.path.display()))
    }
}

fn default_device_name() -> String {
    std::env::var("AZ_DRIVE_DEVICE_NAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "device".to_owned())
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration as StdDuration};

    use tempfile::TempDir;

    use super::LocalStateStore;

    #[test]
    fn local_state_write_lock_serializes_cross_process_writers() {
        let temp = TempDir::new().expect("temp dir should exist");
        let store = LocalStateStore::new(temp.path().join("state.json"));
        let guard = store
            .acquire_write_lock()
            .expect("first lock should succeed");
        let store_for_thread = store.clone();

        let worker = thread::spawn(move || {
            let _guard = store_for_thread
                .acquire_write_lock()
                .expect("second lock should succeed after release");
        });

        thread::sleep(StdDuration::from_millis(150));
        assert!(
            !worker.is_finished(),
            "the second writer must block while the first lock is held"
        );

        drop(guard);
        worker.join().expect("worker should finish");
    }
}
