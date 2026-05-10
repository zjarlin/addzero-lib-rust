#![forbid(unsafe_code)]

//! Standalone headless drive app support utilities.

use anyhow::Context;
use az_drive_agent::{DriveAgent, DriveAgentConfig, LocalStateStore};
use az_drive_store::{
    DriveMetadataStore, DriveObjectStore, InMemoryDriveMetadataStore, InMemoryDriveObjectStore,
    PgDriveMetadataStore, S3DriveObjectStore,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub mod cli;
pub mod setup;

#[cfg(target_os = "macos")]
pub mod macos_actions;

/// Shared drive store handles used by CLI and embedded AIO commands.
pub type DriveStores = (Arc<dyn DriveMetadataStore>, Arc<dyn DriveObjectStore>);

/// Default server bind address for the standalone WebDAV service.
#[must_use]
pub fn default_bind_addr() -> String {
    config_value("AZ_DRIVE_BIND").unwrap_or_else(|| "127.0.0.1:8788".to_owned())
}

/// Default drive space used by the CLI and daemon.
#[must_use]
pub fn default_space_id() -> String {
    config_value("AZ_DRIVE_SPACE")
        .or_else(|| auth_username().map(|username| drive_space_id_for_username(&username)))
        .unwrap_or_else(|| "main".to_owned())
}

/// Stable per-user drive space used by API-key fusion.
#[must_use]
pub fn drive_space_id_for_username(username: &str) -> String {
    let safe = username
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if safe.is_empty() {
        "main".to_owned()
    } else {
        format!("user-{safe}")
    }
}

/// Additional spaces visible for read-side fusion.
#[must_use]
pub fn default_fused_space_ids(primary_space: &str) -> Vec<String> {
    let mut spaces = Vec::new();
    if primary_space != "main" {
        spaces.push("main".to_owned());
    }
    if let Some(auth) = read_auth_file() {
        for key in auth.trusted_api_keys {
            push_unique_space(&mut spaces, key.owner_space_id);
        }
    }
    spaces.retain(|space| space != primary_space);
    spaces
}

/// Default object bucket for drive bytes.
#[must_use]
pub fn default_bucket() -> String {
    config_value("AZ_DRIVE_BUCKET")
        .or_else(|| config_value("AIO_DRIVE_BUCKET"))
        .unwrap_or_else(|| "aio-drive".to_owned())
}

/// Builds a drive agent using the same configuration sources as the CLI.
///
/// # Errors
/// Returns an error when local state, PostgreSQL, migrations, or object store
/// initialization fails.
pub async fn build_agent() -> anyhow::Result<DriveAgent> {
    let (metadata, objects) = build_stores().await?;
    let state_store = LocalStateStore::new(LocalStateStore::default_path());
    let state = state_store.load_or_init().await?;
    let primary_space = default_space_id();
    let config = DriveAgentConfig::new(primary_space.clone(), state.device_id, state.device_name)
        .with_fused_space_ids(default_fused_space_ids(&primary_space));
    Ok(DriveAgent::new(metadata, objects, state_store, config))
}

/// Builds configured metadata and object stores.
///
/// # Errors
/// Returns an error when configured PostgreSQL or S3-compatible storage cannot
/// be initialized.
pub async fn build_stores() -> anyhow::Result<DriveStores> {
    let metadata: Arc<dyn DriveMetadataStore> = if let Some(database_url) = database_url() {
        let store = PgDriveMetadataStore::connect(&database_url)
            .await
            .context("failed to connect drive postgres metadata store")?;
        store
            .run_migrations()
            .await
            .context("failed to run drive postgres migrations")?;
        Arc::new(store)
    } else {
        eprintln!(
            "AZ_DRIVE_DATABASE_URL/MSC_AIO_DATABASE_URL/DATABASE_URL not set; using non-persistent metadata store"
        );
        Arc::new(InMemoryDriveMetadataStore::new())
    };

    let objects: Arc<dyn DriveObjectStore> = if let Some(config) = s3_config() {
        let bucket = default_bucket();
        let store = tokio::task::spawn_blocking(move || {
            let client = az_rustfs::create_storage_client(config);
            S3DriveObjectStore::new(client, bucket)
        })
        .await
        .context("S3 object store initialization task failed")?
        .context("failed to initialize S3 object store")?;
        Arc::new(store)
    } else {
        eprintln!("AZ_DRIVE_MINIO_*/AIO_MINIO_* not set; using non-persistent object store");
        Arc::new(InMemoryDriveObjectStore::new())
    };

    Ok((metadata, objects))
}

/// Returns a configuration value from process env or drive env files.
#[must_use]
pub fn config_value(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| drive_env_values().remove(key))
}

/// Returns the configured PostgreSQL URL, if present.
#[must_use]
pub fn database_url() -> Option<String> {
    setup::current_database_url().filter(|value| !value.trim().is_empty())
}

/// Returns the configured S3-compatible object-store client config, if present.
#[must_use]
pub fn s3_config() -> Option<az_rustfs::S3ClientConfig> {
    let endpoint = setup::current_minio_endpoint()?;
    let access_key = setup::current_minio_access_key()?;
    let secret_key = setup::current_minio_secret_key()?;
    let region = setup::current_minio_region();
    Some(
        az_rustfs::S3ClientConfig::new(endpoint, access_key, secret_key)
            .with_region(region)
            .with_path_style_access(true),
    )
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

#[derive(Deserialize)]
struct AuthFile {
    username: String,
    #[serde(default)]
    trusted_api_keys: Vec<TrustedApiKeyFile>,
}

#[derive(Deserialize)]
struct TrustedApiKeyFile {
    owner_space_id: String,
}

fn auth_username() -> Option<String> {
    read_auth_file()
        .map(|auth| auth.username)
        .filter(|username| !username.trim().is_empty())
}

fn read_auth_file() -> Option<AuthFile> {
    let path = aio_config_dir()?.join("auth.json");
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn push_unique_space(spaces: &mut Vec<String>, space: String) {
    if !space.trim().is_empty() && !spaces.contains(&space) {
        spaces.push(space);
    }
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
