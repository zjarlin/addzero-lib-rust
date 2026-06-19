#![forbid(unsafe_code)]

//! Standalone headless drive app support utilities.

use anyhow::Context;
use az_drive_agent::{
    agent::{DriveAgent, DriveAgentConfig, HostedStatus},
    local_state::{LocalState, LocalStateStore},
};
use az_drive_store::api::{
    DEFAULT_AUTO_GIT_POOL_PREFIX, DEFAULT_BLOB_SHARD_PREFIX, DEFAULT_GIT_POOL_LIMIT_BYTES,
    DEFAULT_MAX_BLOB_SHARD_SIZE_BYTES, DriveMetadataStore, DriveObjectStore, DriveSyncCoordinator,
    GitDbObjectStore, GitDbObjectStoreConfig, GitPoolConfig, GitPoolDriveStore, GitPoolRepoConfig,
};
use az_str::sanitize::sanitize_file_stem;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use toml_edit::{DocumentMut, value};

automod::dir!(pub "src");

/// Shared drive store handles used by CLI and embedded AIO commands.
pub type DriveStores = (
    Arc<dyn DriveMetadataStore>,
    Arc<dyn DriveObjectStore>,
    Arc<dyn DriveSyncCoordinator>,
);

/// Default Git pool root used by the source-of-truth backend.
#[must_use]
pub fn default_git_pool_root() -> PathBuf {
    config_value("AIO_DRIVE_GIT_POOL_ROOT")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".aio/drive-git-pool")))
        .unwrap_or_else(|| PathBuf::from(".aio/drive-git-pool"))
}

/// Default root for GitDB-sharded object storage.
#[must_use]
pub fn default_gitdb_object_root() -> PathBuf {
    config_value("AIO_DRIVE_GITDB_OBJECT_ROOT")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME").map(|home| PathBuf::from(home).join(".aio/drive-gitdb-objects"))
        })
        .unwrap_or_else(|| PathBuf::from(".aio/drive-gitdb-objects"))
}

/// Default server bind address for the standalone WebDAV service.
#[must_use]
pub fn default_bind_addr() -> String {
    config_value("AZ_DRIVE_BIND").unwrap_or_else(|| "127.0.0.1:8788".to_owned())
}

/// Default Drive owner used by the CLI and daemon.
#[must_use]
pub fn default_owner_drive_id() -> String {
    read_auth_file()
        .and_then(|auth| auth.default_owner_drive_id())
        .unwrap_or_else(|| "main".to_owned())
}

/// Compatibility alias for older internal callers.
#[must_use]
pub fn default_space_id() -> String {
    default_owner_drive_id()
}

/// Stable per-user Drive owner id used by API-key authorization.
#[must_use]
pub fn owner_drive_id_for_username(username: &str) -> String {
    let safe = sanitize_file_stem(username.trim());
    if safe.is_empty() {
        "main".to_owned()
    } else {
        format!("user-{safe}")
    }
}

/// Compatibility alias for older internal callers.
#[must_use]
pub fn drive_space_id_for_username(username: &str) -> String {
    owner_drive_id_for_username(username)
}

/// Additional owner Drives visible for read-side fusion.
#[must_use]
pub fn default_fused_space_ids(primary_owner_drive_id: &str) -> Vec<String> {
    let mut drives = Vec::new();
    if let Some(auth) = read_auth_file() {
        for key in auth.trusted_api_keys {
            push_unique_drive(&mut drives, key.owner_drive_id);
        }
    }
    for owner in git_pool_mounted_owner_drive_ids() {
        push_unique_drive(&mut drives, owner);
    }
    drives.retain(|drive| drive != primary_owner_drive_id);
    drives
}

/// Owner Drives that should be materialized automatically during bidirectional sync.
#[must_use]
pub fn default_auto_materialize_space_ids(primary_owner_drive_id: &str) -> Vec<String> {
    let mut drives = Vec::new();
    if let Some(auth) = read_auth_file() {
        for key in auth.trusted_api_keys {
            push_unique_drive(&mut drives, key.owner_drive_id);
        }
    }
    for owner in git_pool_mounted_owner_drive_ids() {
        push_unique_drive(&mut drives, owner);
    }
    drives.retain(|drive| drive != primary_owner_drive_id);
    drives
}

/// Builds a drive agent using the same configuration sources as the CLI.
///
/// # Errors
/// Returns an error when local state or Git Pool initialization fails.
pub async fn build_agent() -> anyhow::Result<DriveAgent> {
    build_agent_and_migrate()
        .await
        .map(|(agent, _migrated)| agent)
}

/// Runs the legacy `main` to current owner Drive migration for the active login.
///
/// # Errors
/// Returns an error when the agent cannot be built or migration fails.
pub async fn migrate_legacy_main_for_current_owner() -> anyhow::Result<u64> {
    build_agent_and_migrate()
        .await
        .map(|(_agent, migrated)| migrated)
}

async fn build_agent_and_migrate() -> anyhow::Result<(DriveAgent, u64)> {
    let (metadata, objects, sync) = build_stores().await?;
    let state_store = LocalStateStore::new(LocalStateStore::default_path());
    let state = state_store.load_or_init().await?;
    let primary_owner_drive_id = default_owner_drive_id();
    let config = DriveAgentConfig::new(
        primary_owner_drive_id.clone(),
        state.device_id,
        state.device_name,
    )
    .with_fused_space_ids(default_fused_space_ids(&primary_owner_drive_id))
    .with_auto_materialize_space_ids(default_auto_materialize_space_ids(&primary_owner_drive_id));
    let agent = DriveAgent::new_with_sync(metadata, objects, sync, state_store, config);
    let migrated = if primary_owner_drive_id != "main" {
        agent
            .migrate_legacy_owner_drive("main", &primary_owner_drive_id)
            .await
            .context("failed to migrate legacy main drive namespace")?
    } else {
        0
    };
    Ok((agent, migrated))
}

/// Builds configured metadata and object stores.
///
/// # Errors
/// Returns an error when Drive metadata or object storage cannot be initialized.
pub async fn build_stores() -> anyhow::Result<DriveStores> {
    build_git_pool_stores()
}

fn build_git_pool_stores() -> anyhow::Result<DriveStores> {
    let config = current_git_pool_config()?;
    let store = Arc::new(GitPoolDriveStore::open(config)?);
    let metadata: Arc<dyn DriveMetadataStore> = store.clone();
    let objects: Arc<dyn DriveObjectStore> = match current_object_backend_config()? {
        DriveObjectBackendConfig::GitPool => store.clone(),
        DriveObjectBackendConfig::GitDb(config) => Arc::new(GitDbObjectStore::open(config)?),
    };
    let sync: Arc<dyn DriveSyncCoordinator> = store;
    Ok((metadata, objects, sync))
}

fn git_pool_mounted_owner_drive_ids() -> Vec<String> {
    let Ok(config) = current_git_pool_config() else {
        return Vec::new();
    };
    let Ok(store) = GitPoolDriveStore::open(config) else {
        return Vec::new();
    };
    store
        .list_mounts()
        .map(|mounts| {
            mounts
                .into_iter()
                .map(|mount| mount.owner_drive_id)
                .collect()
        })
        .unwrap_or_default()
}

/// Initializes the Git Pool backend and persists it as the default.
///
/// # Errors
/// Returns an error when config cannot be written or the Git control repo cannot
/// be initialized.
pub fn init_git_pool_backend(
    control_remote: Option<&str>,
    auto_pool_root: Option<PathBuf>,
    auto_pool_prefix: Option<&str>,
    object_backend: Option<&str>,
    gitdb_object_root: Option<PathBuf>,
    gitdb_object_shard_prefix: Option<&str>,
    gitdb_object_max_shard_size_bytes: Option<u64>,
) -> anyhow::Result<serde_json::Value> {
    let mut config = current_drive_config()?;
    config.backend = "git_pool".to_owned();
    config
        .git_pool_root
        .get_or_insert_with(default_git_pool_root);
    config.control_remote = control_remote
        .map(str::to_owned)
        .or_else(|| config.control_remote.clone());
    config.auto_pool_root = auto_pool_root.or(config.auto_pool_root);
    if let Some(prefix) = auto_pool_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        config.auto_pool_prefix = prefix.to_owned();
    }
    if let Some(backend) = object_backend
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        config.object_backend = normalize_object_backend_name(backend)?;
    }
    config.gitdb_object_root = gitdb_object_root.or(config.gitdb_object_root);
    if let Some(prefix) = gitdb_object_shard_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        config.gitdb_object_shard_prefix = prefix.to_owned();
    }
    if let Some(limit) = gitdb_object_max_shard_size_bytes {
        config.gitdb_object_max_shard_size_bytes = limit;
    }
    save_drive_config(&config)?;
    drive_backend_status()
}

/// Adds a writable Git pool repo.
///
/// # Errors
/// Returns an error when Git setup or config writes fail.
pub fn add_git_pool(
    name: &str,
    remote_url: &str,
    max_size_bytes: Option<u64>,
) -> anyhow::Result<GitPoolRepoConfig> {
    let store = GitPoolDriveStore::open(current_git_pool_config()?)?;
    store.init_pool(name, remote_url, max_size_bytes)
}

/// Mounts another owner Git pool as a read-side fused source.
///
/// # Errors
/// Returns an error when Git setup or config writes fail.
pub fn mount_git_pool(
    name: &str,
    remote_url: &str,
    owner: &str,
    readonly: bool,
) -> anyhow::Result<serde_json::Value> {
    let store = GitPoolDriveStore::open(current_git_pool_config()?)?;
    let mount = store.mount_pool(name, remote_url, owner, readonly)?;
    Ok(serde_json::to_value(mount)?)
}

/// Unmounts a fused Git pool.
///
/// # Errors
/// Returns an error when config writes fail.
pub fn unmount_git_pool(name: &str) -> anyhow::Result<()> {
    let store = GitPoolDriveStore::open(current_git_pool_config()?)?;
    store.unmount_pool(name)
}

/// Returns Git Pool backend status.
///
/// # Errors
/// Returns an error when store metadata cannot be read.
pub fn git_pool_backend_status() -> anyhow::Result<serde_json::Value> {
    let store = GitPoolDriveStore::open(current_git_pool_config()?)?;
    store.backend_status()
}

/// Returns current Drive backend status.
///
/// # Errors
/// Returns an error when Git Pool metadata cannot be read.
pub fn drive_backend_status() -> anyhow::Result<serde_json::Value> {
    let mut status = git_pool_backend_status()?;
    match current_object_backend_config()? {
        DriveObjectBackendConfig::GitPool => {
            status["object_backend"] = serde_json::json!("git_pool");
        }
        DriveObjectBackendConfig::GitDb(config) => {
            let store = GitDbObjectStore::open(config)?;
            status["object_backend"] = serde_json::json!("gitdb");
            status["object_store"] = store.backend_status()?;
        }
    }
    Ok(status)
}

/// Persists the drive backend selector.
///
/// # Errors
/// Returns an error when config cannot be written or the requested backend is unsupported.
pub fn use_drive_backend(backend: &str) -> anyhow::Result<()> {
    let mut config = current_drive_config()?;
    config.backend = normalize_backend_name(backend)?;
    config
        .git_pool_root
        .get_or_insert_with(default_git_pool_root);
    save_drive_config(&config)
}

/// Imports currently hosted local paths into the Git Pool backend.
///
/// # Errors
/// Returns an error when hosted state cannot be loaded or any file cannot be
/// hosted into the Git Pool store.
pub async fn migrate_local_state_to_git_pool() -> anyhow::Result<Vec<HostedStatus>> {
    use_drive_backend("git_pool")?;
    let state_store = LocalStateStore::new(LocalStateStore::default_path());
    let state = state_store.load_or_init().await?;
    let agent = build_agent().await?;
    migrate_state_paths(&agent, &state).await
}

async fn migrate_state_paths(
    agent: &DriveAgent,
    state: &LocalState,
) -> anyhow::Result<Vec<HostedStatus>> {
    let mut statuses = Vec::new();
    for root in &state.hosted_roots {
        if root.local_path.exists() {
            statuses.extend(
                agent
                    .host_path(
                        &root.local_path.to_string_lossy(),
                        Some(&root.root_alias),
                        None,
                    )
                    .await?,
            );
        }
    }
    for hosted in &state.hosted {
        if hosted.local_path.exists() {
            statuses.extend(
                agent
                    .host_path(
                        &hosted.local_path.to_string_lossy(),
                        Some(&hosted.root_alias),
                        Some(&hosted.relative_path),
                    )
                    .await?,
            );
        }
    }
    Ok(statuses)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DriveTomlConfig {
    backend: String,
    git_pool_root: Option<PathBuf>,
    control_remote: Option<String>,
    default_pool_limit_bytes: u64,
    auto_pool_root: Option<PathBuf>,
    auto_pool_prefix: String,
    object_backend: DriveObjectBackend,
    gitdb_object_root: Option<PathBuf>,
    gitdb_object_shard_prefix: String,
    gitdb_object_max_shard_size_bytes: u64,
}

fn current_git_pool_config() -> anyhow::Result<GitPoolConfig> {
    let config = current_drive_config()?;
    Ok(GitPoolConfig {
        root: config.git_pool_root.unwrap_or_else(default_git_pool_root),
        owner_drive_id: default_owner_drive_id(),
        control_remote: config.control_remote,
        default_pool_limit_bytes: config.default_pool_limit_bytes,
        auto_pool_root: config.auto_pool_root,
        auto_pool_prefix: config.auto_pool_prefix,
    })
}

enum DriveObjectBackendConfig {
    GitPool,
    GitDb(GitDbObjectStoreConfig),
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Default,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[strum(serialize_all = "snake_case")]
enum DriveObjectBackend {
    #[default]
    GitPool,
    #[strum(serialize = "gitdb")]
    GitDb,
}

impl DriveObjectBackend {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

fn current_object_backend_config() -> anyhow::Result<DriveObjectBackendConfig> {
    let config = current_drive_config()?;
    match config.object_backend {
        DriveObjectBackend::GitDb => Ok(DriveObjectBackendConfig::GitDb(GitDbObjectStoreConfig {
            root: config
                .gitdb_object_root
                .unwrap_or_else(default_gitdb_object_root),
            max_shard_size_bytes: config.gitdb_object_max_shard_size_bytes,
            shard_prefix: config.gitdb_object_shard_prefix,
        })),
        DriveObjectBackend::GitPool => Ok(DriveObjectBackendConfig::GitPool),
    }
}

fn current_drive_config() -> anyhow::Result<DriveTomlConfig> {
    let Some(path) = drive_toml_path() else {
        return Ok(default_drive_toml_config());
    };
    if !path.exists() {
        return Ok(default_drive_toml_config());
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("读取 Drive 配置失败: {}", path.display()))?;
    let doc = raw
        .parse::<DocumentMut>()
        .with_context(|| format!("解析 Drive 配置失败: {}", path.display()))?;
    let mut config = default_drive_toml_config();
    if let Some(backend) = doc
        .get("backend")
        .and_then(toml_edit::Item::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        config.backend = normalize_persisted_backend_name(backend);
    }
    if let Some(root) = doc
        .get("git_pool_root")
        .and_then(toml_edit::Item::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        config.git_pool_root = Some(expand_home_path(root));
    }
    if let Some(remote) = doc
        .get("control_remote")
        .and_then(toml_edit::Item::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        config.control_remote = Some(remote.to_owned());
    }
    if let Some(limit) = doc
        .get("default_pool_limit_bytes")
        .and_then(toml_edit::Item::as_integer)
        .and_then(|value| u64::try_from(value).ok())
    {
        config.default_pool_limit_bytes = limit;
    }
    if let Some(root) = doc
        .get("auto_pool_root")
        .and_then(toml_edit::Item::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        config.auto_pool_root = Some(expand_home_path(root));
    }
    if let Some(prefix) = doc
        .get("auto_pool_prefix")
        .and_then(toml_edit::Item::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        config.auto_pool_prefix = prefix.to_owned();
    }
    if let Some(backend) = doc
        .get("object_backend")
        .and_then(toml_edit::Item::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        config.object_backend = normalize_persisted_object_backend_name(backend);
    }
    if let Some(root) = doc
        .get("gitdb_object_root")
        .and_then(toml_edit::Item::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        config.gitdb_object_root = Some(expand_home_path(root));
    }
    if let Some(prefix) = doc
        .get("gitdb_object_shard_prefix")
        .and_then(toml_edit::Item::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        config.gitdb_object_shard_prefix = prefix.to_owned();
    }
    if let Some(limit) = doc
        .get("gitdb_object_max_shard_size_bytes")
        .and_then(toml_edit::Item::as_integer)
        .and_then(|value| u64::try_from(value).ok())
    {
        config.gitdb_object_max_shard_size_bytes = limit;
    }
    Ok(config)
}

fn save_drive_config(config: &DriveTomlConfig) -> anyhow::Result<()> {
    let path = drive_toml_path().context("无法定位 ~/.config/aio/drive.toml")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建 Drive 配置目录失败: {}", parent.display()))?;
    }
    let mut doc = DocumentMut::new();
    doc["backend"] = value(config.backend.as_str());
    doc["git_pool_root"] = value(path_to_config_string(
        config
            .git_pool_root
            .clone()
            .unwrap_or_else(default_git_pool_root),
    ));
    if let Some(remote) = &config.control_remote {
        doc["control_remote"] = value(remote.as_str());
    }
    doc["default_pool_limit_bytes"] = value(config.default_pool_limit_bytes as i64);
    if let Some(root) = &config.auto_pool_root {
        doc["auto_pool_root"] = value(path_to_config_string(root.clone()));
    }
    if config.auto_pool_prefix != DEFAULT_AUTO_GIT_POOL_PREFIX {
        doc["auto_pool_prefix"] = value(config.auto_pool_prefix.as_str());
    }
    doc["object_backend"] = value(config.object_backend.code());
    if let Some(root) = &config.gitdb_object_root {
        doc["gitdb_object_root"] = value(path_to_config_string(root.clone()));
    }
    if config.gitdb_object_shard_prefix != DEFAULT_BLOB_SHARD_PREFIX {
        doc["gitdb_object_shard_prefix"] = value(config.gitdb_object_shard_prefix.as_str());
    }
    doc["gitdb_object_max_shard_size_bytes"] =
        value(config.gitdb_object_max_shard_size_bytes as i64);
    fs::write(&path, doc.to_string())
        .with_context(|| format!("写入 Drive 配置失败: {}", path.display()))
}

fn default_drive_toml_config() -> DriveTomlConfig {
    DriveTomlConfig {
        backend: "git_pool".to_owned(),
        git_pool_root: Some(default_git_pool_root()),
        control_remote: None,
        default_pool_limit_bytes: DEFAULT_GIT_POOL_LIMIT_BYTES,
        auto_pool_root: None,
        auto_pool_prefix: DEFAULT_AUTO_GIT_POOL_PREFIX.to_owned(),
        object_backend: DriveObjectBackend::GitPool,
        gitdb_object_root: Some(default_gitdb_object_root()),
        gitdb_object_shard_prefix: DEFAULT_BLOB_SHARD_PREFIX.to_owned(),
        gitdb_object_max_shard_size_bytes: DEFAULT_MAX_BLOB_SHARD_SIZE_BYTES,
    }
}

fn normalize_backend_name(value: &str) -> anyhow::Result<String> {
    match value.trim().replace('-', "_").as_str() {
        "git_pool" => Ok("git_pool".to_owned()),
        "pg_minio" | "pg" | "postgres_minio" => {
            anyhow::bail!("Drive 旧版 pg/minio 后端已删除，只支持 git_pool")
        }
        other => anyhow::bail!("不支持的 Drive backend: {other}"),
    }
}

fn normalize_persisted_backend_name(value: &str) -> String {
    match value.trim().replace('-', "_").as_str() {
        "pg_minio" | "pg" | "postgres_minio" => "git_pool".to_owned(),
        "" => "git_pool".to_owned(),
        other => other.to_owned(),
    }
}

fn normalize_object_backend_name(value: &str) -> anyhow::Result<DriveObjectBackend> {
    let normalized = value.trim().replace('-', "_");
    DriveObjectBackend::from_code(&normalized)
        .ok_or_else(|| anyhow::anyhow!("不支持的 Drive object backend: {}", normalized))
}

fn normalize_persisted_object_backend_name(value: &str) -> DriveObjectBackend {
    let normalized = value.trim().replace('-', "_");
    DriveObjectBackend::from_code_or_default(&normalized)
}

fn drive_toml_path() -> Option<PathBuf> {
    aio_config_dir().map(|dir| dir.join("drive.toml"))
}

fn path_to_config_string(path: PathBuf) -> String {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return path.display().to_string();
    };
    if let Ok(rest) = path.strip_prefix(&home) {
        if rest.as_os_str().is_empty() {
            return "~".to_owned();
        }
        return format!("~/{}", rest.display());
    }
    path.display().to_string()
}

fn expand_home_path(raw: &str) -> PathBuf {
    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        if raw == "~" {
            return home;
        }
        if let Some(rest) = raw.strip_prefix("~/") {
            return home.join(rest);
        }
        if let Some(rest) = raw.strip_prefix("$HOME/") {
            return home.join(rest);
        }
    }
    PathBuf::from(raw)
}

/// Returns a configuration value from process env or drive env files.
#[must_use]
pub fn config_value(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| drive_env_values().remove(key))
}

/// Returns candidate config file locations used by the CLI and Finder actions.
#[must_use]
pub fn drive_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(dir) = aio_config_dir() {
        paths.push(dir.join("drive.toml"));
        paths.push(dir.join("auth.json"));
    }
    paths
}

/// Returns the AIO config directory used by the headless drive.
#[must_use]
pub fn aio_config_dir() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|config| config.join("aio"))
}

#[derive(Debug, serde::Deserialize)]
struct AuthFile {
    username: String,
    #[serde(default)]
    drive_api_key: Option<DriveApiKeyFile>,
    #[serde(default)]
    trusted_api_keys: Vec<TrustedApiKeyFile>,
}

impl AuthFile {
    fn default_owner_drive_id(&self) -> Option<String> {
        self.drive_api_key
            .as_ref()
            .map(|key| key.owner_drive_id.clone())
            .filter(|drive| !drive.trim().is_empty())
            .or_else(|| {
                (!self.username.trim().is_empty())
                    .then(|| owner_drive_id_for_username(&self.username))
            })
    }
}

#[derive(Debug, serde::Deserialize)]
struct DriveApiKeyFile {
    #[serde(alias = "owner_space_id")]
    owner_drive_id: String,
}

#[derive(Debug, serde::Deserialize)]
struct TrustedApiKeyFile {
    #[serde(alias = "owner_space_id")]
    owner_drive_id: String,
}

fn read_auth_file() -> Option<AuthFile> {
    let path = aio_config_dir()?.join("auth.json");
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn push_unique_drive(drives: &mut Vec<String>, drive: String) {
    if !drive.trim().is_empty() && !drives.contains(&drive) {
        drives.push(drive);
    }
}

fn drive_env_values() -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for path in legacy_drive_env_paths() {
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

fn legacy_drive_env_paths() -> Vec<PathBuf> {
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
