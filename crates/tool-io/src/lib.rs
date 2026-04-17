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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn ensure_file_and_directory_behave_explicitly() {
        let temp = TempDir::new().expect("temp dir should be created");
        let file_path = temp.path().join("nested/output.txt");
        let dir_path = temp.path().join("logs");

        file_path
            .as_path()
            .ensure_file()
            .expect("file should be created");
        dir_path
            .as_path()
            .ensure_dir()
            .expect("dir should be created");

        assert!(file_path.is_file());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn remove_if_exists_handles_files_and_directories() {
        let temp = TempDir::new().expect("temp dir should be created");
        let file_path = temp.path().join("artifact.txt");
        let dir_path = temp.path().join("build/cache");
        file_path
            .as_path()
            .ensure_file()
            .expect("file should be created");
        dir_path
            .as_path()
            .ensure_dir()
            .expect("dir should be created");

        file_path
            .as_path()
            .remove_if_exists()
            .expect("file should be removed");
        temp.path()
            .join("build")
            .as_path()
            .remove_if_exists()
            .expect("dir should be removed");

        assert!(!file_path.exists());
        assert!(!temp.path().join("build").exists());
    }

    #[test]
    fn mvln_returns_noop_when_paths_match() {
        let temp = TempDir::new().expect("temp dir should be created");
        let path = temp.path().join("same.txt");
        fs::write(&path, "hello").expect("file should be written");

        let result = mvln(&path, &path).expect("same path should be a noop");

        assert_eq!(result, path);
    }

    #[cfg(unix)]
    #[test]
    fn mvln_moves_file_and_undo_restores_it() {
        let temp = TempDir::new().expect("temp dir should be created");
        let source = temp.path().join("report.txt");
        fs::write(&source, "hello world").expect("file should be written");

        let moved = MoveLink::new(&source)
            .to(temp.path().join("archive"))
            .move_and_link()
            .expect("move-and-link should succeed");

        assert_eq!(moved, temp.path().join("archive/report.txt"));
        assert!(
            fs::symlink_metadata(&source)
                .expect("metadata should exist")
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            fs::read_to_string(&source).expect("link should resolve"),
            "hello world"
        );

        let restored = undo_mvln(&source).expect("undo should succeed");

        assert_eq!(restored, source);
        assert!(source.is_file());
        assert_eq!(
            fs::read_to_string(&source).expect("file should be restored"),
            "hello world"
        );
        assert!(!moved.exists());
    }

    #[cfg(unix)]
    #[test]
    fn mvln_moves_directory_and_undo_restores_it() {
        let temp = TempDir::new().expect("temp dir should be created");
        let source = temp.path().join("docs");
        source
            .as_path()
            .ensure_dir()
            .expect("dir should be created");
        fs::write(source.join("guide.md"), "# guide").expect("nested file should be written");

        let moved = mvln(&source, temp.path().join("backup")).expect("directory move should work");

        assert_eq!(moved, temp.path().join("backup/docs"));
        assert!(
            fs::symlink_metadata(&source)
                .expect("metadata should exist")
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            fs::read_to_string(source.join("guide.md")).expect("link should resolve"),
            "# guide"
        );

        undo_mvln(&source).expect("undo should restore directory");

        assert!(source.is_dir());
        assert_eq!(
            fs::read_to_string(source.join("guide.md")).expect("directory should be restored"),
            "# guide"
        );
        assert!(!moved.exists());
    }

    #[cfg(unix)]
    #[test]
    fn undo_mvln_reports_non_symlink_and_broken_symlink_errors() {
        let temp = TempDir::new().expect("temp dir should be created");
        let regular_file = temp.path().join("plain.txt");
        fs::write(&regular_file, "hello").expect("file should be written");

        let regular_error = undo_mvln(&regular_file).expect_err("regular file should fail");
        assert!(matches!(regular_error, IoError::NotSymlink(_)));

        let broken_link = temp.path().join("dangling.txt");
        std::os::unix::fs::symlink(temp.path().join("missing.txt"), &broken_link)
            .expect("broken symlink should be created");

        let broken_error = undo_mvln(&broken_link).expect_err("broken symlink should fail");
        assert!(matches!(broken_error, IoError::BrokenSymlink(_)));
    }
}
