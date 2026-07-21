//! 独立 AIO 网盘的核心路径、版本和冲突规则。
//!
//! 本 crate 刻意将本地绝对路径排除在远程标识之外。
//! 远程文件通过 `owner_drive + root_alias + relative_path` 进行标识，
//! 而各设备在本地存储各自的绝对路径映射。

use anyhow::{Context, Result, bail};
use az_str::sanitize::sanitize_ascii_label_or;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::env;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

/// Cross-device logical root name, for example `home` or `workspace`.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Display,
)]
pub struct RootAlias(String);

impl RootAlias {
    /// Default root alias mapped to `$HOME` or `%USERPROFILE%`.
    pub const HOME: &'static str = "home";

    /// Parses a stable root alias.
    ///
    /// # Errors
    /// Returns an error when the alias is empty or contains characters
    /// outside `[A-Za-z0-9_-]`.
    pub fn parse(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref().trim();
        if value.is_empty() {
            bail!("root alias cannot be empty");
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            bail!("invalid root alias `{value}`");
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the alias as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for RootAlias {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::parse(value)
    }
}

/// Normalized POSIX-style path below a logical root.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Display,
)]
pub struct RelativePath(String);

impl RelativePath {
    /// Parses a remote relative path.
    ///
    /// Empty input represents the root itself. Non-empty paths reject absolute
    /// paths, `..`, empty segments, Windows drive prefixes, and backtracking.
    ///
    /// # Errors
    /// Returns an error when the path is unsafe
    /// or not canonical.
    pub fn parse(value: impl AsRef<str>) -> Result<Self> {
        let raw = value.as_ref().replace('\\', "/");
        let trimmed = raw.trim_matches('/');
        if trimmed.is_empty() {
            return Ok(Self(String::new()));
        }
        if raw.starts_with('/') || looks_like_windows_drive(&raw) {
            bail!("invalid relative path `{raw}`");
        }

        let mut segments = Vec::new();
        for segment in trimmed.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                bail!("invalid relative path `{raw}`");
            }
            segments.push(segment);
        }

        Ok(Self(segments.join("/")))
    }

    /// Builds a relative path from local path components.
    ///
    /// # Errors
    /// Returns an error when the local path contains unsupported
    /// components such as prefixes, roots, or parent traversal.
    pub fn from_local_path(path: impl AsRef<Path>) -> Result<Self> {
        let mut segments = Vec::new();
        for component in path.as_ref().components() {
            match component {
                Component::Normal(value) => segments.push(value.to_string_lossy().to_string()),
                Component::CurDir => {}
                Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                    bail!("invalid relative path `{}`", path.as_ref().display());
                }
            }
        }
        Self::parse(segments.join("/"))
    }

    /// Returns `true` when the path represents a logical root.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the normalized path.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts the relative path back to platform local path segments.
    #[must_use]
    pub fn to_local_path(&self) -> PathBuf {
        self.0.split('/').filter(|part| !part.is_empty()).collect()
    }
}

impl TryFrom<&str> for RelativePath {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::parse(value)
    }
}

/// Stable remote identity for a file or directory.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EntryKey {
    /// Owner Drive namespace.
    pub space_id: String,
    /// Cross-device root alias.
    pub root_alias: RootAlias,
    /// Path relative to the logical root.
    pub relative_path: RelativePath,
}

impl EntryKey {
    /// Creates a new entry key.
    #[must_use]
    pub fn new(
        space_id: impl Into<String>,
        root_alias: RootAlias,
        relative_path: RelativePath,
    ) -> Self {
        Self {
            space_id: space_id.into(),
            root_alias,
            relative_path,
        }
    }

    /// Returns the canonical remote path `owner_drive/root/relative`.
    #[must_use]
    pub fn remote_path(&self) -> String {
        if self.relative_path.is_root() {
            format!("{}/{}", self.space_id, self.root_alias)
        } else {
            format!(
                "{}/{}/{}",
                self.space_id, self.root_alias, self.relative_path
            )
        }
    }
}

/// A device-local logical root mapping.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LogicalRoot {
    /// Cross-device logical alias.
    pub alias: RootAlias,
    /// Device-local absolute root path.
    pub local_path: PathBuf,
}

/// Result of mapping a device-local path to remote identity.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HostPathMapping {
    /// Device-local normalized absolute path.
    pub local_abs_path: PathBuf,
    /// Logical root selected for the path.
    pub root_alias: RootAlias,
    /// Path below the logical root.
    pub relative_path: RelativePath,
}

/// Registry that maps logical root aliases to device-local paths.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RootRegistry {
    roots: BTreeMap<RootAlias, PathBuf>,
}

impl RootRegistry {
    /// Builds a registry containing the default `home` root.
    ///
    /// # Errors
    /// Returns an error when neither `$HOME` nor
    /// `%USERPROFILE%` is available.
    pub fn default_for_device() -> Result<Self> {
        let home = home_dir().context("home directory could not be resolved")?;
        let mut registry = Self::default();
        registry.add_root(RootAlias::parse(RootAlias::HOME)?, home)?;
        Ok(registry)
    }

    /// Adds or replaces a logical root.
    ///
    /// # Errors
    /// Returns an error when `path` cannot be normalized.
    pub fn add_root(&mut self, alias: RootAlias, path: impl AsRef<Path>) -> Result<()> {
        let path = normalize_absolute_path(path.as_ref())?;
        self.roots.insert(alias, path);
        Ok(())
    }

    /// Returns configured roots sorted by alias.
    #[must_use]
    pub fn list_roots(&self) -> Vec<LogicalRoot> {
        self.roots
            .iter()
            .map(|(alias, path)| LogicalRoot {
                alias: alias.clone(),
                local_path: path.clone(),
            })
            .collect()
    }

    /// Resolves a local path to `root_alias + relative_path`.
    ///
    /// When no preferred alias is provided, the longest matching root wins.
    ///
    /// # Errors
    /// Returns an error when the path is outside all configured roots,
    /// or outside the requested root.
    pub fn resolve_host_path(
        &self,
        path: impl AsRef<Path>,
        preferred_alias: Option<&RootAlias>,
    ) -> Result<HostPathMapping> {
        let local_abs_path = normalize_absolute_path(path.as_ref())?;

        let selected = match preferred_alias {
            Some(alias) => {
                let Some(root) = self.roots.get(alias) else {
                    bail!(
                        "path is not under any configured root: {}",
                        local_abs_path.display()
                    );
                };
                if !local_abs_path.starts_with(root) {
                    bail!(
                        "path `{}` is outside root `{}`",
                        local_abs_path.display(),
                        root.display()
                    );
                }
                Some((alias.clone(), root.clone()))
            }
            None => self
                .roots
                .iter()
                .filter(|(_, root)| local_abs_path.starts_with(root))
                .max_by_key(|(_, root)| root.components().count())
                .map(|(alias, root)| (alias.clone(), root.clone())),
        };

        let Some((root_alias, root_path)) = selected else {
            bail!(
                "path is not under any configured root: {}",
                local_abs_path.display()
            );
        };

        let relative_local = local_abs_path.strip_prefix(&root_path).with_context(|| {
            format!(
                "path `{}` is outside root `{}`",
                local_abs_path.display(),
                root_path.display()
            )
        })?;
        let relative_path = RelativePath::from_local_path(relative_local)?;

        Ok(HostPathMapping {
            local_abs_path,
            root_alias,
            relative_path,
        })
    }
}

/// Expands user-facing path expressions accepted by the CLI.
///
/// `~`, `$HOME`, `${HOME}`, and `%USERPROFILE%` are expanded locally only.
/// The expanded value must never be stored as remote identity.
#[must_use]
pub fn expand_path_expression(value: impl AsRef<str>) -> PathBuf {
    let value = value.as_ref();
    let home = home_dir();
    if value == "~" {
        return home.unwrap_or_else(|| PathBuf::from(value));
    }
    if let Some(rest) = value.strip_prefix("~/")
        && let Some(home) = home
    {
        return home.join(rest);
    }
    if value == "$HOME" || value == "${HOME}" || value == "%USERPROFILE%" {
        return home.unwrap_or_else(|| PathBuf::from(value));
    }
    for prefix in ["$HOME/", "${HOME}/", "%USERPROFILE%/"] {
        if let Some(rest) = value.strip_prefix(prefix)
            && let Some(home) = home_dir()
        {
            return home.join(rest);
        }
    }
    PathBuf::from(value)
}

/// Normalizes a path lexically and makes it absolute without requiring it to exist.
///
/// # Errors
/// Returns an error when the process current directory is
/// unavailable while resolving a relative path.
pub fn normalize_absolute_path(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .context("current directory could not be resolved")?
            .join(path)
    };
    Ok(lexical_normalize(&path))
}

/// Version metadata used by sync decisions.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntryVersion {
    /// Metadata entry id.
    pub entry_id: Uuid,
    /// Monotonic per-entry version.
    pub version: u64,
    /// Hex SHA-256 content hash.
    pub content_hash: String,
    /// Device that wrote this version.
    pub device_id: String,
    /// Write timestamp.
    pub modified_at: DateTime<Utc>,
}

/// Active lock snapshot for conflict decisions.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LockSnapshot {
    /// Lock owner device id.
    pub owner_device_id: String,
    /// Lock expiry.
    pub expires_at: DateTime<Utc>,
}

impl LockSnapshot {
    /// Returns true when the lock is still active.
    #[must_use]
    pub fn is_active_at(&self, now: DateTime<Utc>) -> bool {
        self.expires_at > now
    }
}

/// Decision for a local file change against the latest remote version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChangeDecision {
    /// Local content already matches remote content.
    NoopSameContent,
    /// Local content can become the next remote version.
    UploadNewVersion,
    /// Remote changed since the local base version.
    Conflict,
    /// Another device owns an active lock.
    LockedByOther {
        /// Lock owner device id.
        owner_device_id: String,
    },
}

/// Decides whether a local change can be uploaded, conflicts, or is blocked.
#[must_use]
pub fn decide_local_change(
    base_version: Option<u64>,
    remote_version: Option<u64>,
    local_hash: &str,
    remote_hash: Option<&str>,
    lock: Option<&LockSnapshot>,
    device_id: &str,
    now: DateTime<Utc>,
) -> ChangeDecision {
    if let Some(lock) = lock
        && lock.is_active_at(now)
        && lock.owner_device_id != device_id
    {
        return ChangeDecision::LockedByOther {
            owner_device_id: lock.owner_device_id.clone(),
        };
    }

    if remote_hash == Some(local_hash) {
        return ChangeDecision::NoopSameContent;
    }

    match (base_version, remote_version) {
        (_, None) => ChangeDecision::UploadNewVersion,
        (Some(base), Some(remote)) if base == remote => ChangeDecision::UploadNewVersion,
        _ => ChangeDecision::Conflict,
    }
}

/// Computes a stable SHA-256 content hash.
#[must_use]
pub fn content_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

/// Returns the object-store key for content-addressed bytes.
#[must_use]
pub fn object_key_for_hash(hash: &str) -> String {
    format!("objects/sha256/{hash}")
}

/// Builds a deterministic local conflict copy name.
#[must_use]
pub fn conflict_file_name(path: &Path, device_name: &str, timestamp: DateTime<Utc>) -> String {
    let file_name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_owned());
    let sanitized_device = sanitize_conflict_part(device_name);
    let stamp = timestamp.format("%Y%m%dT%H%M%SZ");

    let candidate = Path::new(&file_name);
    let stem = candidate
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| file_name.clone());
    let extension = candidate.extension().map(|value| value.to_string_lossy());

    match extension {
        Some(extension) if !extension.is_empty() => {
            format!("{stem}.conflict.{sanitized_device}.{stamp}.{extension}")
        }
        _ => format!("{stem}.conflict.{sanitized_device}.{stamp}"),
    }
}

/// Attempts a conservative automatic text merge.
///
/// This intentionally returns `None` for ambiguous edits; callers should then
/// create a conflict copy rather than block synchronization.
#[must_use]
pub fn try_safe_text_merge(base: &[u8], local: &[u8], remote: &[u8]) -> Option<Vec<u8>> {
    if local == remote {
        return Some(local.to_vec());
    }
    if local == base {
        return Some(remote.to_vec());
    }
    if remote == base {
        return Some(local.to_vec());
    }

    let base_text = std::str::from_utf8(base).ok()?;
    let local_text = std::str::from_utf8(local).ok()?;
    let remote_text = std::str::from_utf8(remote).ok()?;

    if local_text.starts_with(base_text) && remote_text.starts_with(base_text) {
        let local_suffix = &local_text[base_text.len()..];
        let remote_suffix = &remote_text[base_text.len()..];
        if !local_suffix.is_empty() && !remote_suffix.is_empty() && local_suffix != remote_suffix {
            let mut merged =
                String::with_capacity(base_text.len() + local_suffix.len() + remote_suffix.len());
            merged.push_str(base_text);
            merged.push_str(local_suffix);
            if !merged.ends_with('\n') && !remote_suffix.starts_with('\n') {
                merged.push('\n');
            }
            merged.push_str(remote_suffix);
            return Some(merged.into_bytes());
        }
    }

    None
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn looks_like_windows_drive(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(value) => normalized.push(value),
        }
    }
    normalized
}

fn sanitize_conflict_part(value: &str) -> String {
    sanitize_ascii_label_or(value, "-_", '-', "device")
}
