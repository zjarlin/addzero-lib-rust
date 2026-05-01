use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, DotfilesError>;

#[derive(Debug, Error)]
pub enum DotfilesError {
    #[error("{0}")]
    Message(String),
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("json error at {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error(
        "command `{command}` failed with exit code {code:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    )]
    Command {
        command: String,
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },
}

impl From<std::io::Error> for DotfilesError {
    fn from(source: std::io::Error) -> Self {
        Self::Io {
            path: PathBuf::from("<unknown>"),
            source,
        }
    }
}

impl From<serde_json::Error> for DotfilesError {
    fn from(source: serde_json::Error) -> Self {
        Self::Json {
            path: PathBuf::from("<memory>"),
            source,
        }
    }
}

pub fn io_error(path: impl Into<PathBuf>, source: std::io::Error) -> DotfilesError {
    DotfilesError::Io {
        path: path.into(),
        source,
    }
}

pub fn message(message: impl Into<String>) -> DotfilesError {
    DotfilesError::Message(message.into())
}
