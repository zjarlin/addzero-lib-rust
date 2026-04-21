use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IoError {
    #[error("path does not exist: {0}")]
    MissingPath(PathBuf),
    #[error("move target was not configured")]
    MissingTarget,
    #[error("source path does not have a final path segment: {0}")]
    MissingFileName(PathBuf),
    #[error("path is not a symlink: {0}")]
    NotSymlink(PathBuf),
    #[error("symlink target does not exist: {0}")]
    BrokenSymlink(PathBuf),
    #[error("expected {expected} at {path}")]
    UnexpectedFileType {
        path: PathBuf,
        expected: &'static str,
    },
    #[error("symlink operations are supported only on unix targets")]
    UnsupportedSymlink,
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveLink {
    source: PathBuf,
    target: Option<PathBuf>,
}

impl MoveLink {
    pub fn new(source: impl Into<PathBuf>) -> Self {
        Self {
            source: source.into(),
            target: None,
        }
    }

    pub fn to(mut self, target: impl Into<PathBuf>) -> Self {
        self.target = Some(target.into());
        self
    }

    pub fn move_and_link(self) -> Result<PathBuf, IoError> {
        let target = self.target.ok_or(IoError::MissingTarget)?;
        mvln(self.source, target)
    }
}

pub trait PathExt {
    fn ensure_file(&self) -> Result<(), IoError>;
    fn ensure_dir(&self) -> Result<(), IoError>;
    fn remove_if_exists(&self) -> Result<(), IoError>;
}

impl PathExt for Path {
    fn ensure_file(&self) -> Result<(), IoError> {
        if self.exists() {
            if self.is_file() {
                return Ok(());
            }

            return Err(IoError::UnexpectedFileType {
                path: self.to_path_buf(),
                expected: "a file",
            });
        }

        if let Some(parent) = self.parent() {
            fs::create_dir_all(parent).map_err(|source| IoError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        fs::File::create(self).map_err(|source| IoError::Io {
            path: self.to_path_buf(),
            source,
        })?;
        Ok(())
    }

    fn ensure_dir(&self) -> Result<(), IoError> {
        if self.exists() {
            if self.is_dir() {
                return Ok(());
            }

            return Err(IoError::UnexpectedFileType {
                path: self.to_path_buf(),
                expected: "a directory",
            });
        }

        fs::create_dir_all(self).map_err(|source| IoError::Io {
            path: self.to_path_buf(),
            source,
        })
    }

    fn remove_if_exists(&self) -> Result<(), IoError> {
        match fs::symlink_metadata(self) {
            Ok(metadata) => {
                let file_type = metadata.file_type();
                if file_type.is_dir() {
                    fs::remove_dir_all(self).map_err(|source| IoError::Io {
                        path: self.to_path_buf(),
                        source,
                    })?;
                } else {
                    fs::remove_file(self).map_err(|source| IoError::Io {
                        path: self.to_path_buf(),
                        source,
                    })?;
                }
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(IoError::Io {
                path: self.to_path_buf(),
                source,
            }),
        }
    }
}

pub fn mvln(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<PathBuf, IoError> {
    let source = source.as_ref().to_path_buf();
    let target = target.as_ref().to_path_buf();

    if !source.exists() {
        return Err(IoError::MissingPath(source));
    }

    if source == target {
        return Ok(source);
    }

    let final_target = resolve_final_target(&source, &target)?;

    if let Some(parent) = final_target.parent() {
        fs::create_dir_all(parent).map_err(|source_error| IoError::Io {
            path: parent.to_path_buf(),
            source: source_error,
        })?;
    }

    fs::rename(&source, &final_target).map_err(|source_error| IoError::Io {
        path: source.clone(),
        source: source_error,
    })?;

    create_symlink(&final_target, &source)?;

    Ok(final_target)
}

pub fn undo_mvln(path: impl AsRef<Path>) -> Result<PathBuf, IoError> {
    let symlink_path = path.as_ref().to_path_buf();

    let metadata = fs::symlink_metadata(&symlink_path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            IoError::MissingPath(symlink_path.clone())
        } else {
            IoError::Io {
                path: symlink_path.clone(),
                source,
            }
        }
    })?;

    if !metadata.file_type().is_symlink() {
        return Err(IoError::NotSymlink(symlink_path));
    }

    let raw_target = fs::read_link(&symlink_path).map_err(|source| IoError::Io {
        path: symlink_path.clone(),
        source,
    })?;
    let resolved_target = if raw_target.is_absolute() {
        raw_target
    } else {
        symlink_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(raw_target)
            .normalize()
    };

    if !resolved_target.exists() {
        return Err(IoError::BrokenSymlink(resolved_target));
    }

    fs::remove_file(&symlink_path).map_err(|source| IoError::Io {
        path: symlink_path.clone(),
        source,
    })?;

    fs::rename(&resolved_target, &symlink_path).map_err(|source| IoError::Io {
        path: symlink_path.clone(),
        source,
    })?;

    Ok(symlink_path)
}

fn resolve_final_target(source: &Path, target: &Path) -> Result<PathBuf, IoError> {
    let Some(source_name) = source.file_name() else {
        return Err(IoError::MissingFileName(source.to_path_buf()));
    };

    let final_target = match target.file_name() {
        Some(target_name) if target_name == source_name => target.to_path_buf(),
        _ => target.join(source_name),
    };

    Ok(final_target)
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> Result<(), IoError> {
    std::os::unix::fs::symlink(target, link).map_err(|source| IoError::Io {
        path: link.to_path_buf(),
        source,
    })
}

#[cfg(not(unix))]
fn create_symlink(_target: &Path, _link: &Path) -> Result<(), IoError> {
    Err(IoError::UnsupportedSymlink)
}

trait NormalizePath {
    fn normalize(&self) -> PathBuf;
}

impl NormalizePath for PathBuf {
    fn normalize(&self) -> PathBuf {
        let mut normalized = PathBuf::new();
        for component in self.components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    normalized.pop();
                }
                _ => normalized.push(component.as_os_str()),
            }
        }
        normalized
    }
}
