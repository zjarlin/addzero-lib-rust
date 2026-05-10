#![forbid(unsafe_code)]

//! Standalone headless drive app support utilities.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

pub mod setup;

/// Default server bind address for the standalone WebDAV service.
#[must_use]
pub fn default_bind_addr() -> String {
    config_value("AZ_DRIVE_BIND").unwrap_or_else(|| "127.0.0.1:8788".to_owned())
}

/// Default drive space used by the CLI and daemon.
#[must_use]
pub fn default_space_id() -> String {
    config_value("AZ_DRIVE_SPACE").unwrap_or_else(|| "main".to_owned())
}

/// Default object bucket for drive bytes.
#[must_use]
pub fn default_bucket() -> String {
    config_value("AZ_DRIVE_BUCKET")
        .or_else(|| config_value("AIO_DRIVE_BUCKET"))
        .unwrap_or_else(|| "aio-drive".to_owned())
}

/// Returns a configuration value from process env or drive env files.
#[must_use]
pub fn config_value(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| drive_env_values().remove(key))
}

/// Returns candidate env file locations used by the CLI and Finder actions.
#[must_use]
pub fn drive_env_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(path) = env::var_os("AZ_DRIVE_ENV") {
        paths.push(PathBuf::from(path));
    }
    if let Some(dir) = aio_config_dir() {
        paths.push(dir.join("aio.env"));
        paths.push(dir.join("drive.env"));
    }
    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        paths.push(home.join(".config").join("az-drive").join("drive.env"));
    }
    paths
}

/// Returns the canonical env file path written by `setup`.
#[must_use]
pub fn drive_env_write_path() -> Option<PathBuf> {
    env::var_os("AZ_DRIVE_ENV")
        .map(PathBuf::from)
        .or_else(|| aio_config_dir().map(|dir| dir.join("aio.env")))
}

/// Returns the AIO config directory used by the headless drive.
#[must_use]
pub fn aio_config_dir() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|config| config.join("aio"))
}

fn drive_env_values() -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for path in drive_env_paths() {
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                values
                    .entry(key.trim().to_owned())
                    .or_insert(value.to_owned());
            }
        }
    }
    values
}
