//! 文件系统 I/O 工具库，提供「移动并符号链接回原位」以及路径确保操作。
//!
//! # 核心功能
//!
//! - [`mvln`] — 将文件/目录移动到新位置，并在原路径创建符号链接指向新位置。
//!   适用于将本地数据迁移到外部存储后保留原有访问路径的场景。
//! - [`undo_mvln`] — 撤销 `mvln` 操作：移除符号链接并将文件移回原位。
//! - [`PathExt`] trait — 为 [`std::path::Path`] 扩展三个常用方法：
//!   - [`ensure_file`](PathExt::ensure_file) — 确保路径存在且为文件（不存在则自动创建）。
//!   - [`ensure_dir`](PathExt::ensure_dir) — 确保路径存在且为目录（不存在则自动创建）。
//!   - [`remove_if_exists`](PathExt::remove_if_exists) — 安全删除路径（不存在时静默通过）。
//! - [`MoveLink`] — builder 风格的 `mvln` 包装器，适合链式调用。
//!
//! # 错误处理
//!
//! 所有公开函数返回 `anyhow::Result<T>`，文件系统失败会保留底层 `std::io::Error` 并附带路径上下文。
//!
//! # 平台说明
//!
//! 符号链接操作仅在 Unix 平台可用；非 Unix 平台调用时返回错误。
//!
//! # 典型用法
//!
//! ```rust,no_run
//! use az_io::api::mvln;
//! use std::path::Path;
//! # fn main() -> anyhow::Result<()> {
//!
//! // 将 data.db 移动到 /mnt/external/data.db，并在原位保留符号链接
//! let new_path = mvln("data.db", "/mnt/external")?;
//! # Ok(())
//! # }
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// Extension methods for ensuring and removing filesystem paths.
pub trait PathExt {
    /// Ensure the path exists and is a file, creating parent directories and the file when needed.
    fn ensure_file(&self) -> Result<()>;

    /// Ensure the path exists and is a directory, creating it when needed.
    fn ensure_dir(&self) -> Result<()>;

    /// Remove the file, directory, or symlink when it exists.
    fn remove_if_exists(&self) -> Result<()>;
}

impl PathExt for Path {
    fn ensure_file(&self) -> Result<()> {
        if self.exists() {
            return if self.is_file() {
                Ok(())
            } else {
                bail!("path has unexpected file type: {}", self.display());
            };
        }

        if let Some(parent) = self
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .with_context(|| format!("filesystem operation failed for {}", parent.display()))?;
        }
        fs::File::create(self)
            .map(drop)
            .with_context(|| format!("filesystem operation failed for {}", self.display()))
    }

    fn ensure_dir(&self) -> Result<()> {
        if self.exists() {
            return if self.is_dir() {
                Ok(())
            } else {
                bail!("path has unexpected file type: {}", self.display());
            };
        }

        fs::create_dir_all(self)
            .with_context(|| format!("filesystem operation failed for {}", self.display()))
    }

    fn remove_if_exists(&self) -> Result<()> {
        let metadata = match fs::symlink_metadata(self) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                let message = format!("filesystem operation failed for {}", self.display());
                let error = anyhow::Error::new(error).context(message);

                return Err(error);
            }
        };

        let file_type = metadata.file_type();
        if file_type.is_dir() {
            fs::remove_dir_all(self)
                .with_context(|| format!("filesystem operation failed for {}", self.display()))
        } else {
            fs::remove_file(self)
                .with_context(|| format!("filesystem operation failed for {}", self.display()))
        }
    }
}

impl PathExt for PathBuf {
    fn ensure_file(&self) -> Result<()> {
        self.as_path().ensure_file()
    }

    fn ensure_dir(&self) -> Result<()> {
        self.as_path().ensure_dir()
    }

    fn remove_if_exists(&self) -> Result<()> {
        self.as_path().remove_if_exists()
    }
}

/// Builder-style wrapper around [`mvln`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveLink {
    source: PathBuf,
    target_dir: Option<PathBuf>,
}

impl MoveLink {
    /// Start a move-and-link operation from the given source path.
    pub fn new(source: impl AsRef<Path>) -> Self {
        Self {
            source: source.as_ref().to_path_buf(),
            target_dir: None,
        }
    }

    /// Set the destination directory or exact destination path.
    pub fn to(mut self, target: impl AsRef<Path>) -> Self {
        self.target_dir = Some(target.as_ref().to_path_buf());
        self
    }

    /// Execute the configured move-and-link operation.
    pub fn move_and_link(self) -> Result<PathBuf> {
        let Some(target) = self.target_dir else {
            bail!("symlink target does not exist: ");
        };
        mvln(self.source, target)
    }
}

/// Move a file or directory and create a symlink at the original path.
pub fn mvln(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<PathBuf> {
    let source = source.as_ref();
    let target = target.as_ref();

    if source == target {
        return Ok(source.to_path_buf());
    }

    let metadata = match fs::symlink_metadata(source) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bail!("source path does not exist: {}", source.display());
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("filesystem operation failed for {}", source.display()));
        }
    };

    let destination = destination_path(source, target, &metadata)?;
    if let Some(parent) = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        parent.ensure_dir()?;
    }

    fs::rename(source, &destination)
        .with_context(|| format!("filesystem operation failed for {}", source.display()))?;

    let link_target = fs::canonicalize(&destination)
        .with_context(|| format!("filesystem operation failed for {}", destination.display()))?;

    create_symlink(&link_target, source).inspect_err(|_| {
        let _ = fs::rename(&destination, source);
    })?;

    Ok(destination)
}

/// Undo a previous [`mvln`] operation.
pub fn undo_mvln(link_path: impl AsRef<Path>) -> Result<PathBuf> {
    let link_path = link_path.as_ref();
    let metadata = match fs::symlink_metadata(link_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bail!("source path does not exist: {}", link_path.display());
        }
        Err(error) => {
            let message = format!("filesystem operation failed for {}", link_path.display());
            let error = anyhow::Error::new(error).context(message);

            return Err(error);
        }
    };

    if !metadata.file_type().is_symlink() {
        bail!("path is not a symlink: {}", link_path.display());
    }

    let target = fs::read_link(link_path)
        .with_context(|| format!("filesystem operation failed for {}", link_path.display()))?;
    let target = if target.is_absolute() {
        target
    } else {
        link_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(target)
    };

    if !target.exists() {
        bail!(
            "symlink target is missing or invalid: {}",
            link_path.display()
        );
    }

    fs::remove_file(link_path)
        .with_context(|| format!("filesystem operation failed for {}", link_path.display()))?;
    fs::rename(&target, link_path)
        .with_context(|| format!("filesystem operation failed for {}", target.display()))?;
    Ok(link_path.to_path_buf())
}

fn destination_path(source: &Path, target: &Path, metadata: &fs::Metadata) -> Result<PathBuf> {
    if target.exists() {
        if target.is_dir() {
            let Some(file_name) = source.file_name() else {
                bail!("path has unexpected file type: {}", source.display());
            };
            return Ok(target.join(file_name));
        }
        bail!("path has unexpected file type: {}", target.display());
    }

    if metadata.file_type().is_dir() {
        let Some(file_name) = source.file_name() else {
            bail!("path has unexpected file type: {}", source.display());
        };
        Ok(target.join(file_name))
    } else if target.extension().is_some() {
        Ok(target.to_path_buf())
    } else {
        let Some(file_name) = source.file_name() else {
            bail!("path has unexpected file type: {}", source.display());
        };
        Ok(target.join(file_name))
    }
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link)
        .with_context(|| format!("filesystem operation failed for {}", link.display()))
}

#[cfg(not(unix))]
fn create_symlink(_target: &Path, _link: &Path) -> Result<()> {
    bail!("symbolic links are not supported on this platform");
}
