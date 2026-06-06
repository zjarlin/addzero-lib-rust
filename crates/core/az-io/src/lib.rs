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
//! 所有公开函数返回 [`Result<T, IoError>`]，[`IoError`] 使用 `thiserror` 派生，
//! 提供结构化的错误变体：路径缺失、目标缺失、文件类型不符、符号链接相关错误等。
//!
//! # 平台说明
//!
//! 符号链接操作仅在 Unix 平台可用；非 Unix 平台调用时返回
//! [`IoError::UnsupportedSymlink`]。
//!
//! # 典型用法
//!
//! ```rust,no_run
//! use az_io::mvln;
//! use std::path::Path;
//! # fn main() -> az_io::IoResult<()> {
//!
//! // 将 data.db 移动到 /mnt/external/data.db，并在原位保留符号链接
//! let new_path = mvln("data.db", "/mnt/external")?;
//! # Ok(())
//! # }
//! ```

use std::fs;
use std::path::{Path, PathBuf};

/// Result alias for filesystem helper operations.
pub type IoResult<T> = Result<T, IoError>;

/// Error type returned by filesystem helper operations.
#[derive(Debug, thiserror::Error)]
pub enum IoError {
    /// The source path does not exist.
    #[error("source path does not exist: {0}")]
    SourceMissing(PathBuf),
    /// The symlink target does not exist.
    #[error("symlink target does not exist: {0}")]
    TargetMissing(PathBuf),
    /// The path exists but has an unexpected file type.
    #[error("path has unexpected file type: {0}")]
    InvalidFileType(PathBuf),
    /// The path is expected to be a symbolic link.
    #[error("path is not a symlink: {0}")]
    NotSymlink(PathBuf),
    /// The symlink target cannot be used to restore the original path.
    #[error("symlink target is missing or invalid: {0}")]
    BrokenSymlink(PathBuf),
    /// Symbolic links are not supported by this platform build.
    #[error("symbolic links are not supported on this platform")]
    UnsupportedSymlink,
    /// A filesystem operation failed.
    #[error("filesystem operation failed for {path}: {source}")]
    Fs {
        /// Path involved in the failed operation.
        path: PathBuf,
        /// Original I/O error.
        #[source]
        source: std::io::Error,
    },
}

impl IoError {
    fn fs(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Fs {
            path: path.into(),
            source,
        }
    }
}

/// Extension methods for ensuring and removing filesystem paths.
pub trait PathExt {
    /// Ensure the path exists and is a file, creating parent directories and the file when needed.
    fn ensure_file(&self) -> IoResult<()>;

    /// Ensure the path exists and is a directory, creating it when needed.
    fn ensure_dir(&self) -> IoResult<()>;

    /// Remove the file, directory, or symlink when it exists.
    fn remove_if_exists(&self) -> IoResult<()>;
}

impl PathExt for Path {
    fn ensure_file(&self) -> IoResult<()> {
        if self.exists() {
            return if self.is_file() {
                Ok(())
            } else {
                Err(IoError::InvalidFileType(self.to_path_buf()))
            };
        }

        if let Some(parent) = self
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|source| IoError::fs(parent, source))?;
        }
        fs::File::create(self)
            .map(drop)
            .map_err(|source| IoError::fs(self, source))
    }

    fn ensure_dir(&self) -> IoResult<()> {
        if self.exists() {
            return if self.is_dir() {
                Ok(())
            } else {
                Err(IoError::InvalidFileType(self.to_path_buf()))
            };
        }

        fs::create_dir_all(self).map_err(|source| IoError::fs(self, source))
    }

    fn remove_if_exists(&self) -> IoResult<()> {
        let metadata = match fs::symlink_metadata(self) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(IoError::fs(self, error)),
        };

        let file_type = metadata.file_type();
        if file_type.is_dir() {
            fs::remove_dir_all(self).map_err(|source| IoError::fs(self, source))
        } else {
            fs::remove_file(self).map_err(|source| IoError::fs(self, source))
        }
    }
}

impl PathExt for PathBuf {
    fn ensure_file(&self) -> IoResult<()> {
        self.as_path().ensure_file()
    }

    fn ensure_dir(&self) -> IoResult<()> {
        self.as_path().ensure_dir()
    }

    fn remove_if_exists(&self) -> IoResult<()> {
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
    pub fn move_and_link(self) -> IoResult<PathBuf> {
        let target = self
            .target_dir
            .ok_or_else(|| IoError::TargetMissing(PathBuf::new()))?;
        mvln(self.source, target)
    }
}

/// Move a file or directory and create a symlink at the original path.
pub fn mvln(source: impl AsRef<Path>, target: impl AsRef<Path>) -> IoResult<PathBuf> {
    let source = source.as_ref();
    let target = target.as_ref();

    if source == target {
        return Ok(source.to_path_buf());
    }

    let metadata = fs::symlink_metadata(source).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            IoError::SourceMissing(source.to_path_buf())
        } else {
            IoError::fs(source, error)
        }
    })?;

    let destination = destination_path(source, target, &metadata)?;
    if let Some(parent) = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        parent.ensure_dir()?;
    }

    fs::rename(source, &destination).map_err(|source_error| IoError::fs(source, source_error))?;

    let link_target = fs::canonicalize(&destination)
        .map_err(|source_error| IoError::fs(&destination, source_error))?;

    create_symlink(&link_target, source).inspect_err(|_| {
        let _ = fs::rename(&destination, source);
    })?;

    Ok(destination)
}

/// Undo a previous [`mvln`] operation.
pub fn undo_mvln(link_path: impl AsRef<Path>) -> IoResult<PathBuf> {
    let link_path = link_path.as_ref();
    let metadata = fs::symlink_metadata(link_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            IoError::SourceMissing(link_path.to_path_buf())
        } else {
            IoError::fs(link_path, error)
        }
    })?;

    if !metadata.file_type().is_symlink() {
        return Err(IoError::NotSymlink(link_path.to_path_buf()));
    }

    let target = fs::read_link(link_path).map_err(|error| IoError::fs(link_path, error))?;
    let target = if target.is_absolute() {
        target
    } else {
        link_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(target)
    };

    if !target.exists() {
        return Err(IoError::BrokenSymlink(link_path.to_path_buf()));
    }

    fs::remove_file(link_path).map_err(|error| IoError::fs(link_path, error))?;
    fs::rename(&target, link_path).map_err(|error| IoError::fs(&target, error))?;
    Ok(link_path.to_path_buf())
}

fn destination_path(source: &Path, target: &Path, metadata: &fs::Metadata) -> IoResult<PathBuf> {
    if target.exists() {
        if target.is_dir() {
            return Ok(target.join(
                source
                    .file_name()
                    .ok_or_else(|| IoError::InvalidFileType(source.to_path_buf()))?,
            ));
        }
        return Err(IoError::InvalidFileType(target.to_path_buf()));
    }

    if metadata.file_type().is_dir() {
        Ok(target.join(
            source
                .file_name()
                .ok_or_else(|| IoError::InvalidFileType(source.to_path_buf()))?,
        ))
    } else if target.extension().is_some() {
        Ok(target.to_path_buf())
    } else {
        Ok(target.join(
            source
                .file_name()
                .ok_or_else(|| IoError::InvalidFileType(source.to_path_buf()))?,
        ))
    }
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> IoResult<()> {
    std::os::unix::fs::symlink(target, link).map_err(|error| IoError::fs(link, error))
}

#[cfg(not(unix))]
fn create_symlink(_target: &Path, _link: &Path) -> IoResult<()> {
    Err(IoError::UnsupportedSymlink)
}
