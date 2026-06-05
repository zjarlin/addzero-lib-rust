use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AzGitError>;

#[derive(Debug, Error)]
pub enum AzGitError {
    #[error("config directory is unavailable")]
    ConfigDirUnavailable,
    #[error("failed to read config {path}: {source}")]
    ReadConfig {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to write config {path}: {source}")]
    WriteConfig {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse config {path}: {source}")]
    ParseConfig {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("failed to encode config: {0}")]
    EncodeConfig(#[from] serde_json::Error),
    #[error("failed to run {program}: {source}")]
    Command {
        program: String,
        source: std::io::Error,
    },
}
