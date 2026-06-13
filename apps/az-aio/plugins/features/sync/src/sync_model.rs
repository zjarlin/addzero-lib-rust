#![cfg(not(target_arch = "wasm32"))]

use std::{
    env,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

pub const DEFAULT_SYNC_ROOT_RELATIVE: &str = "az-sync";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncDeviceInfo {
    pub device_name: String,
    pub home_dir: PathBuf,
    pub os: String,
    pub arch: String,
}

impl SyncDeviceInfo {
    pub fn detect() -> Self {
        Self {
            device_name: detect_device_name(),
            home_dir: detect_home_dir().unwrap_or_else(|| PathBuf::from(".")),
            os: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
        }
    }

    pub fn new(device_name: impl Into<String>, home_dir: impl Into<PathBuf>) -> Self {
        Self {
            device_name: device_name.into(),
            home_dir: home_dir.into(),
            os: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
        }
    }

    pub fn default_sync_root(&self) -> PathBuf {
        self.home_dir.join(DEFAULT_SYNC_ROOT_RELATIVE)
    }

    pub fn home_relative_path(&self, path: impl AsRef<Path>) -> Result<String> {
        let path = path.as_ref();
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.home_dir.join(path)
        };
        if let Ok(relative) = absolute.strip_prefix(&self.home_dir) {
            return Ok(path_to_slash_string(relative));
        }

        let canonical_absolute =
            canonicalize_existing_or_parent(&absolute).unwrap_or_else(|| absolute.clone());
        let canonical_home = canonicalize_existing_or_parent(&self.home_dir)
            .unwrap_or_else(|| self.home_dir.clone());
        let Ok(relative) = canonical_absolute.strip_prefix(&canonical_home) else {
            bail!(
                "path `{absolute:?}` is outside home directory `{:?}`",
                self.home_dir
            );
        };
        let relative = relative.to_path_buf();
        Ok(path_to_slash_string(&relative))
    }

    pub fn local_path_for_home_relative(&self, relative_path: &str) -> Result<PathBuf> {
        let relative = normalize_home_relative_path(relative_path)?;
        Ok(self.home_dir.join(relative))
    }

    pub fn peer_id_for_path(&self, relative_path: &str) -> u64 {
        stable_hash64(&format!("{}:{relative_path}", self.device_name))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncRoot {
    pub alias: String,
    pub local_path: PathBuf,
    pub relative_path: String,
    pub space_id: String,
}

impl SyncRoot {
    pub fn default_for_device(device: &SyncDeviceInfo) -> Self {
        Self {
            alias: "default".to_string(),
            local_path: device.default_sync_root(),
            relative_path: DEFAULT_SYNC_ROOT_RELATIVE.to_string(),
            space_id: "main".to_string(),
        }
    }

    pub fn from_home_relative(
        device: &SyncDeviceInfo,
        alias: impl Into<String>,
        relative_path: &str,
        space_id: impl Into<String>,
    ) -> Result<Self> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        Ok(Self {
            alias: alias.into(),
            local_path: device.local_path_for_home_relative(&relative_path)?,
            relative_path,
            space_id: space_id.into(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncFileStatus {
    Synced,
    Syncing,
    Error,
    Shared,
    Deleted,
}

impl Default for SyncFileStatus {
    fn default() -> Self {
        Self::Synced
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncDocumentRecord {
    pub relative_path: String,
    pub local_path: PathBuf,
    pub device_name: String,
    pub home_dir: PathBuf,
    pub crdt_snapshot: Vec<u8>,
    pub crdt_version: Vec<u8>,
    pub content_hash: String,
    pub status: SyncFileStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncBlobKind {
    Snapshot,
    Update,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncCrdtEnvelope {
    pub relative_path: String,
    pub source_device: String,
    pub base_version: Option<Vec<u8>>,
    pub version: Vec<u8>,
    pub kind: SyncBlobKind,
    pub blob: Vec<u8>,
    pub content_hash: String,
}

pub fn normalize_home_relative_path(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("invalid sync relative path `{value}`");
    }

    let without_home = trimmed
        .strip_prefix("~/")
        .or_else(|| trimmed.strip_prefix("~\\"))
        .unwrap_or(trimmed);
    let normalized = without_home
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string();
    if normalized.is_empty()
        || normalized == "."
        || normalized == ".."
        || normalized.starts_with("../")
        || normalized.ends_with("/..")
        || normalized.contains("/../")
    {
        bail!("invalid sync relative path `{value}`");
    }
    Ok(normalized)
}

pub fn content_hash(text: &str) -> String {
    format!("fnv1a64:{:016x}", stable_hash64(text))
}

pub fn stable_hash64(value: &str) -> u64 {
    let mut hasher = Fnv1a64::default();
    value.hash(&mut hasher);
    hasher.finish()
}

fn detect_device_name() -> String {
    env::var("AZ_SYNC_DEVICE_NAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| env::var("COMPUTERNAME").ok())
        .or_else(|| env::var("HOSTNAME").ok())
        .or_else(read_unix_hostname)
        .unwrap_or_else(|| "local-device".to_string())
}

fn detect_home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
}

#[cfg(unix)]
fn read_unix_hostname() -> Option<String> {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(not(unix))]
fn read_unix_hostname() -> Option<String> {
    None
}

fn path_to_slash_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn canonicalize_existing_or_parent(path: &Path) -> Option<PathBuf> {
    if let Ok(canonical) = path.canonicalize() {
        return Some(canonical);
    }
    let parent = path.parent()?.canonicalize().ok()?;
    Some(parent.join(path.file_name()?))
}

#[derive(Default)]
struct Fnv1a64(u64);

impl Hasher for Fnv1a64 {
    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 {
            0xcbf2_9ce4_8422_2325
        } else {
            self.0
        };
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        self.0 = hash;
    }

    fn finish(&self) -> u64 {
        if self.0 == 0 {
            0xcbf2_9ce4_8422_2325
        } else {
            self.0
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{SyncDeviceInfo, normalize_home_relative_path};

    #[test]
    fn device_maps_absolute_paths_to_home_relative() -> Result<(), Box<dyn std::error::Error>> {
        let device = SyncDeviceInfo::new("mac-a", PathBuf::from("/tmp/mac-a"));

        assert_eq!(
            device.home_relative_path("/tmp/mac-a/az-sync/a.txt")?,
            "az-sync/a.txt"
        );
        assert_eq!(
            device.local_path_for_home_relative("az-sync/a.txt")?,
            PathBuf::from("/tmp/mac-a/az-sync/a.txt")
        );
        Ok(())
    }

    #[test]
    fn relative_paths_are_home_scoped() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(normalize_home_relative_path("~/az-sync")?, "az-sync");
        assert_eq!(
            normalize_home_relative_path("./az-sync/a.txt")?,
            "az-sync/a.txt"
        );
        assert!(normalize_home_relative_path("../secret").is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn home_relative_path_accepts_canonicalized_home_aliases()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let real_home = temp_dir.path().join("real-home");
        let alias_home = temp_dir.path().join("alias-home");
        std::fs::create_dir_all(real_home.join("az-sync"))?;
        std::os::unix::fs::symlink(&real_home, &alias_home)?;
        let device = SyncDeviceInfo::new("mac-a", alias_home);

        assert_eq!(
            device.home_relative_path(real_home.join("az-sync/a.txt"))?,
            "az-sync/a.txt"
        );
        Ok(())
    }
}
