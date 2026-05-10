//! Interactive setup and Docker detection for the headless drive app.

use anyhow::{Context, Result, anyhow};
use az_drive_store::{PgDriveMetadataStore, S3DriveObjectStore};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::time::Duration;

/// Docker detection result used by setup and REPL diagnostics.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct DockerDetection {
    /// Whether the `docker ps` command completed successfully.
    pub docker_available: bool,
    /// Detection error when Docker is unavailable.
    pub error: Option<String>,
    /// PostgreSQL-like containers with a published `5432/tcp` port.
    pub postgres: Vec<PostgresDockerCandidate>,
    /// MinIO containers with a published `9000/tcp` port.
    pub minio: Vec<MinioDockerCandidate>,
}

/// PostgreSQL container candidate.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PostgresDockerCandidate {
    /// Docker container id.
    pub id: String,
    /// Docker container name.
    pub name: String,
    /// Docker image.
    pub image: String,
    /// Host port mapped to container `5432/tcp`.
    pub host_port: u16,
    /// Detected PostgreSQL username from container env.
    pub detected_user: Option<String>,
    /// Detected PostgreSQL database from container env.
    pub detected_database: Option<String>,
    /// Whether a password-like env var was detected.
    pub password_detected: bool,
    #[serde(skip_serializing)]
    detected_password: Option<String>,
}

/// MinIO container candidate.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MinioDockerCandidate {
    /// Docker container id.
    pub id: String,
    /// Docker container name.
    pub name: String,
    /// Docker image.
    pub image: String,
    /// S3 endpoint inferred from published `9000/tcp`.
    pub endpoint: String,
    /// Detected access key from container env.
    pub detected_access_key: Option<String>,
    /// Whether a secret key env var was detected.
    pub secret_detected: bool,
    #[serde(skip_serializing)]
    detected_secret_key: Option<String>,
}

/// Complete drive setup values persisted to the env file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveSetupConfig {
    /// PostgreSQL URL used by drive metadata.
    pub database_url: String,
    /// MinIO/S3-compatible endpoint.
    pub minio_endpoint: String,
    /// MinIO access key.
    pub minio_access_key: String,
    /// MinIO secret key.
    pub minio_secret_key: String,
    /// MinIO region.
    pub minio_region: String,
    /// Object bucket for drive content.
    pub bucket: String,
}

/// User-safe config view that does not expose secrets.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DriveConfigView {
    /// Canonical config path used by setup.
    pub write_path: Option<PathBuf>,
    /// Env files read by the drive app, in priority order.
    pub read_paths: Vec<PathBuf>,
    /// PostgreSQL URL with password masked.
    pub database_url: Option<String>,
    /// MinIO endpoint.
    pub minio_endpoint: Option<String>,
    /// MinIO access key.
    pub minio_access_key: Option<String>,
    /// Whether a MinIO secret key is configured.
    pub minio_secret_configured: bool,
    /// MinIO region.
    pub minio_region: Option<String>,
    /// Object bucket.
    pub bucket: String,
}

/// Connection test result for a single backend.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EndpointTest {
    /// Whether the relevant config values were present.
    pub configured: bool,
    /// Whether validation succeeded.
    pub ok: bool,
    /// User-facing diagnostic with the concrete failure reason.
    pub message: String,
}

/// Full validation result for current setup.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DriveConfigTestResult {
    /// PostgreSQL metadata validation.
    pub postgres: EndpointTest,
    /// MinIO object-store validation.
    pub minio: EndpointTest,
}

/// Result returned after an interactive setup writes config.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DriveSetupResult {
    /// Config file path that was written.
    pub config_path: PathBuf,
    /// PostgreSQL URL with password masked.
    pub database_url: String,
    /// MinIO endpoint.
    pub minio_endpoint: String,
    /// MinIO access key.
    pub minio_access_key: String,
    /// MinIO region.
    pub minio_region: String,
    /// Object bucket.
    pub bucket: String,
    /// Validation result after writing.
    pub test: DriveConfigTestResult,
}

#[derive(Debug, Deserialize)]
struct DockerPsRow {
    #[serde(default, rename = "ID")]
    id: String,
    #[serde(default, rename = "Image")]
    image: String,
    #[serde(default, rename = "Names")]
    names: String,
    #[serde(default, rename = "Ports")]
    ports: String,
    #[serde(default, rename = "Command")]
    command: String,
}

#[derive(Default)]
struct DatabaseUrlParts {
    username: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
}

/// Detects local PostgreSQL/pgvector and MinIO containers from Docker.
#[must_use]
pub fn detect_docker() -> DockerDetection {
    let output = match ProcessCommand::new("docker")
        .args(["ps", "--format", "{{json .}}"])
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return DockerDetection {
                docker_available: false,
                error: Some(error.to_string()),
                ..DockerDetection::default()
            };
        }
    };

    if !output.status.success() {
        return DockerDetection {
            docker_available: false,
            error: Some(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
            ..DockerDetection::default()
        };
    }

    let mut detection = DockerDetection {
        docker_available: true,
        ..DockerDetection::default()
    };
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Ok(row) = serde_json::from_str::<DockerPsRow>(line) else {
            continue;
        };
        let env = inspect_container_env(&row.id);
        if is_postgres_container(&row)
            && let Some(host_port) = host_port_for_container_port(&row.ports, 5432)
        {
            detection.postgres.push(PostgresDockerCandidate {
                id: row.id.clone(),
                name: row.names.clone(),
                image: row.image.clone(),
                host_port,
                detected_user: env.get("POSTGRES_USER").cloned(),
                detected_database: env.get("POSTGRES_DB").cloned(),
                password_detected: env
                    .get("POSTGRES_PASSWORD")
                    .is_some_and(|value| !value.trim().is_empty()),
                detected_password: env.get("POSTGRES_PASSWORD").cloned(),
            });
        }
        if is_minio_container(&row)
            && let Some(host_port) = host_port_for_container_port(&row.ports, 9000)
        {
            detection.minio.push(MinioDockerCandidate {
                id: row.id,
                name: row.names,
                image: row.image,
                endpoint: format!("http://127.0.0.1:{host_port}"),
                detected_access_key: env
                    .get("MINIO_ROOT_USER")
                    .or_else(|| env.get("MINIO_ACCESS_KEY"))
                    .cloned(),
                secret_detected: env
                    .get("MINIO_ROOT_PASSWORD")
                    .or_else(|| env.get("MINIO_SECRET_KEY"))
                    .is_some_and(|value| !value.trim().is_empty()),
                detected_secret_key: env
                    .get("MINIO_ROOT_PASSWORD")
                    .or_else(|| env.get("MINIO_SECRET_KEY"))
                    .cloned(),
            });
        }
    }
    detection
}

/// Formats Docker detection as concise human-readable text.
#[must_use]
pub fn format_detection(detection: &DockerDetection) -> String {
    if !detection.docker_available {
        return format!(
            "Docker 检测不可用: {}",
            detection
                .error
                .as_deref()
                .filter(|value| !value.is_empty())
                .unwrap_or("docker ps 执行失败")
        );
    }

    let mut lines = vec!["Docker 检测结果:".to_owned()];
    if detection.postgres.is_empty() {
        lines.push("  PostgreSQL: 未检测到已发布 5432/tcp 的 postgres/pgvector 容器".to_owned());
    } else {
        for candidate in &detection.postgres {
            lines.push(format!(
                "  PostgreSQL: {} ({}) -> 127.0.0.1:{}{}{}",
                candidate.name,
                candidate.image,
                candidate.host_port,
                candidate
                    .detected_user
                    .as_deref()
                    .map(|user| format!(", user={user}"))
                    .unwrap_or_default(),
                candidate
                    .detected_database
                    .as_deref()
                    .map(|database| format!(", db={database}"))
                    .unwrap_or_default()
            ));
        }
    }

    if detection.minio.is_empty() {
        lines.push("  MinIO: 未检测到已发布 9000/tcp 的 minio 容器".to_owned());
    } else {
        for candidate in &detection.minio {
            lines.push(format!(
                "  MinIO: {} ({}) -> {}{}",
                candidate.name,
                candidate.image,
                candidate.endpoint,
                candidate
                    .detected_access_key
                    .as_deref()
                    .map(|access_key| format!(", access_key={access_key}"))
                    .unwrap_or_default()
            ));
        }
    }
    lines.join("\n")
}

/// Loads current drive config from env and config files.
#[must_use]
pub fn current_config_view() -> DriveConfigView {
    DriveConfigView {
        write_path: crate::drive_env_write_path(),
        read_paths: crate::drive_env_paths(),
        database_url: current_database_url().map(|value| mask_database_url(&value)),
        minio_endpoint: config_value_any(&["AZ_DRIVE_MINIO_ENDPOINT", "AIO_MINIO_ENDPOINT"]),
        minio_access_key: config_value_any(&["AZ_DRIVE_MINIO_ACCESS_KEY", "AIO_MINIO_ACCESS_KEY"]),
        minio_secret_configured: current_minio_secret_key().is_some(),
        minio_region: config_value_any(&["AZ_DRIVE_MINIO_REGION", "AIO_MINIO_REGION"]),
        bucket: crate::default_bucket(),
    }
}

/// Returns the currently configured drive PostgreSQL URL.
#[must_use]
pub fn current_database_url() -> Option<String> {
    config_value_any(&[
        "AZ_DRIVE_DATABASE_URL",
        "MSC_AIO_DATABASE_URL",
        "DATABASE_URL",
    ])
}

/// Returns the currently configured MinIO endpoint.
#[must_use]
pub fn current_minio_endpoint() -> Option<String> {
    config_value_any(&["AZ_DRIVE_MINIO_ENDPOINT", "AIO_MINIO_ENDPOINT"])
}

/// Returns the currently configured MinIO access key.
#[must_use]
pub fn current_minio_access_key() -> Option<String> {
    config_value_any(&["AZ_DRIVE_MINIO_ACCESS_KEY", "AIO_MINIO_ACCESS_KEY"])
}

/// Returns the currently configured MinIO secret key.
#[must_use]
pub fn current_minio_secret_key() -> Option<String> {
    config_value_any(&["AZ_DRIVE_MINIO_SECRET_KEY", "AIO_MINIO_SECRET_KEY"])
}

/// Returns the currently configured MinIO region.
#[must_use]
pub fn current_minio_region() -> String {
    config_value_any(&["AZ_DRIVE_MINIO_REGION", "AIO_MINIO_REGION"])
        .unwrap_or_else(|| "us-east-1".to_owned())
}

/// Tests the current env-backed config.
pub async fn test_current_config() -> DriveConfigTestResult {
    let config = DriveSetupConfig {
        database_url: current_database_url().unwrap_or_default(),
        minio_endpoint: current_minio_endpoint().unwrap_or_default(),
        minio_access_key: current_minio_access_key().unwrap_or_default(),
        minio_secret_key: current_minio_secret_key().unwrap_or_default(),
        minio_region: current_minio_region(),
        bucket: crate::default_bucket(),
    };
    test_config(&config).await
}

/// Runs the interactive first-time setup flow and writes the config file.
pub async fn interactive_setup() -> Result<DriveSetupResult> {
    let detection = detect_docker();
    println!("{}", format_detection(&detection));
    println!();

    let current = current_config_view();
    let current_database_url = current_database_url();
    let current_parts = current_database_url
        .as_deref()
        .map(parse_database_url)
        .unwrap_or_default();
    let postgres = detection.postgres.first();
    let minio = detection.minio.first();

    let pg_host = if postgres.is_some() {
        "127.0.0.1"
    } else {
        current_parts
            .host
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or("127.0.0.1")
    };
    let pg_port = postgres
        .map(|candidate| candidate.host_port)
        .or(current_parts.port)
        .unwrap_or(5432);
    let pg_user = prompt_with_default(
        "PostgreSQL 用户名",
        postgres
            .and_then(|candidate| candidate.detected_user.as_deref())
            .or(current_parts.username.as_deref())
            .unwrap_or("postgres"),
    )?;
    let pg_password = prompt_secret_with_default(
        "PostgreSQL 密码",
        postgres
            .and_then(|candidate| candidate.detected_password.as_deref())
            .or(current_parts.password.as_deref()),
    )?;
    if pg_password.trim().is_empty() {
        return Err(anyhow!("PostgreSQL 密码不能为空"));
    }
    let pg_database = prompt_with_default(
        "PostgreSQL 数据库",
        postgres
            .and_then(|candidate| candidate.detected_database.as_deref())
            .or(current_parts.database.as_deref())
            .unwrap_or("postgres"),
    )?;
    let database_url = postgres_url(pg_host, pg_port, &pg_user, &pg_password, &pg_database);

    let minio_endpoint = prompt_with_default(
        "MinIO endpoint",
        current
            .minio_endpoint
            .as_deref()
            .or_else(|| minio.map(|candidate| candidate.endpoint.as_str()))
            .unwrap_or("http://127.0.0.1:9000"),
    )?;
    let minio_access_key = prompt_with_default(
        "MinIO access key",
        current
            .minio_access_key
            .as_deref()
            .or_else(|| minio.and_then(|candidate| candidate.detected_access_key.as_deref()))
            .unwrap_or("minioadmin"),
    )?;
    let current_minio_secret = current_minio_secret_key();
    let minio_secret_key = prompt_secret_with_default(
        "MinIO secret key",
        minio
            .and_then(|candidate| candidate.detected_secret_key.as_deref())
            .or(current_minio_secret.as_deref()),
    )?;
    if minio_secret_key.trim().is_empty() {
        return Err(anyhow!("MinIO secret key 不能为空"));
    }
    let minio_region = prompt_with_default(
        "MinIO region",
        current.minio_region.as_deref().unwrap_or("us-east-1"),
    )?;
    let bucket = prompt_with_default("Drive bucket", &current.bucket)?;

    let config = DriveSetupConfig {
        database_url,
        minio_endpoint,
        minio_access_key,
        minio_secret_key,
        minio_region,
        bucket,
    };

    println!();
    println!("正在验证 PostgreSQL / MinIO 连接...");
    let test = test_config(&config).await;
    if !test.postgres.ok || !test.minio.ok {
        return Err(anyhow!(
            "配置验证失败: PostgreSQL: {}; MinIO: {}",
            test.postgres.message,
            test.minio.message
        ));
    }

    let config_path = write_config(&config)?;
    Ok(DriveSetupResult {
        config_path,
        database_url: mask_database_url(&config.database_url),
        minio_endpoint: config.minio_endpoint,
        minio_access_key: config.minio_access_key,
        minio_region: config.minio_region,
        bucket: config.bucket,
        test,
    })
}

/// Tests a concrete config without exposing secrets.
pub async fn test_config(config: &DriveSetupConfig) -> DriveConfigTestResult {
    DriveConfigTestResult {
        postgres: test_postgres(&config.database_url).await,
        minio: test_minio(config).await,
    }
}

/// Writes drive setup values into the canonical AIO env file.
pub fn write_config(config: &DriveSetupConfig) -> Result<PathBuf> {
    let path = crate::drive_env_write_path().ok_or_else(|| {
        anyhow!("无法定位配置文件路径；请设置 AZ_DRIVE_ENV 或 HOME/XDG_CONFIG_HOME")
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir {}", parent.display()))?;
    }
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let content = upsert_env_keys(
        &existing,
        &[
            ("AZ_DRIVE_DATABASE_URL", config.database_url.as_str()),
            ("AZ_DRIVE_MINIO_ENDPOINT", config.minio_endpoint.as_str()),
            (
                "AZ_DRIVE_MINIO_ACCESS_KEY",
                config.minio_access_key.as_str(),
            ),
            (
                "AZ_DRIVE_MINIO_SECRET_KEY",
                config.minio_secret_key.as_str(),
            ),
            ("AZ_DRIVE_MINIO_REGION", config.minio_region.as_str()),
            ("AZ_DRIVE_BUCKET", config.bucket.as_str()),
            ("MSC_AIO_DATABASE_URL", config.database_url.as_str()),
            ("AIO_MINIO_ENDPOINT", config.minio_endpoint.as_str()),
            ("AIO_MINIO_ACCESS_KEY", config.minio_access_key.as_str()),
            ("AIO_MINIO_SECRET_KEY", config.minio_secret_key.as_str()),
            ("AIO_MINIO_REGION", config.minio_region.as_str()),
            ("AIO_DRIVE_BUCKET", config.bucket.as_str()),
        ],
    );
    fs::write(&path, content)
        .with_context(|| format!("failed to write config file {}", path.display()))?;
    Ok(path)
}

fn config_value_any(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| crate::config_value(key))
}

async fn test_postgres(database_url: &str) -> EndpointTest {
    if database_url.trim().is_empty() {
        return EndpointTest {
            configured: false,
            ok: false,
            message: "未配置 PostgreSQL URL".to_owned(),
        };
    }
    let result = tokio::time::timeout(Duration::from_secs(10), async {
        let store = PgDriveMetadataStore::connect(database_url).await?;
        store.run_migrations().await
    })
    .await;

    match result {
        Ok(Ok(())) => EndpointTest {
            configured: true,
            ok: true,
            message: "PostgreSQL 可用，migration 已执行".to_owned(),
        },
        Ok(Err(error)) => EndpointTest {
            configured: true,
            ok: false,
            message: error.to_string(),
        },
        Err(_) => EndpointTest {
            configured: true,
            ok: false,
            message: "PostgreSQL 连接或 migration 超时".to_owned(),
        },
    }
}

async fn test_minio(config: &DriveSetupConfig) -> EndpointTest {
    if config.minio_endpoint.trim().is_empty()
        || config.minio_access_key.trim().is_empty()
        || config.minio_secret_key.trim().is_empty()
    {
        return EndpointTest {
            configured: false,
            ok: false,
            message: "未完整配置 MinIO endpoint/access key/secret key".to_owned(),
        };
    }

    let endpoint = config.minio_endpoint.clone();
    let access_key = config.minio_access_key.clone();
    let secret_key = config.minio_secret_key.clone();
    let region = config.minio_region.clone();
    let bucket = config.bucket.clone();
    let result = tokio::time::timeout(Duration::from_secs(10), async move {
        tokio::task::spawn_blocking(move || {
            let client = az_rustfs::create_storage_client(
                az_rustfs::S3ClientConfig::new(endpoint, access_key, secret_key)
                    .with_region(region)
                    .with_path_style_access(true),
            );
            S3DriveObjectStore::new(client, bucket)
        })
        .await
        .map_err(|error| anyhow!("MinIO 初始化任务失败: {error}"))?
        .map(|_| ())
        .map_err(|error| anyhow!(error))
    })
    .await;

    match result {
        Ok(Ok(())) => EndpointTest {
            configured: true,
            ok: true,
            message: "MinIO 可用，bucket 已确认".to_owned(),
        },
        Ok(Err(error)) => EndpointTest {
            configured: true,
            ok: false,
            message: error.to_string(),
        },
        Err(_) => EndpointTest {
            configured: true,
            ok: false,
            message: "MinIO 连接或 bucket 初始化超时".to_owned(),
        },
    }
}

fn inspect_container_env(container_id: &str) -> BTreeMap<String, String> {
    let output = ProcessCommand::new("docker")
        .args(["inspect", container_id, "--format", "{{json .Config.Env}}"])
        .output();
    let Ok(output) = output else {
        return BTreeMap::new();
    };
    if !output.status.success() {
        return BTreeMap::new();
    }
    let Ok(items) = serde_json::from_slice::<Vec<String>>(&output.stdout) else {
        return BTreeMap::new();
    };
    items
        .into_iter()
        .filter_map(|item| {
            let (key, value) = item.split_once('=')?;
            Some((key.to_owned(), value.to_owned()))
        })
        .collect()
}

fn is_postgres_container(row: &DockerPsRow) -> bool {
    let haystack = format!("{} {} {}", row.image, row.names, row.command).to_ascii_lowercase();
    haystack.contains("postgres") || haystack.contains("pgvector")
}

fn is_minio_container(row: &DockerPsRow) -> bool {
    let haystack = format!("{} {} {}", row.image, row.names, row.command).to_ascii_lowercase();
    haystack.contains("minio")
}

fn host_port_for_container_port(ports: &str, container_port: u16) -> Option<u16> {
    let suffix = format!("->{container_port}/");
    ports.split(',').find_map(|part| {
        let part = part.trim();
        if !part.contains(&suffix) {
            return None;
        }
        let (host, _) = part.split_once("->")?;
        let host = host.trim();
        let port = host.rsplit(':').next().unwrap_or(host).trim();
        port.parse::<u16>().ok()
    })
}

fn prompt_with_default(label: &str, default: &str) -> Result<String> {
    print!("{label} [{default}]: ");
    io::stdout().flush().context("failed to flush stdout")?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .context("failed to read setup input")?;
    let value = line.trim();
    if value.is_empty() {
        Ok(default.to_owned())
    } else {
        Ok(value.to_owned())
    }
}

fn prompt_secret_with_default(label: &str, default: Option<&str>) -> Result<String> {
    let prompt = if default.is_some_and(|value| !value.is_empty()) {
        format!("{label} [已检测/已配置，回车沿用]: ")
    } else {
        format!("{label}: ")
    };
    print!("{prompt}");
    io::stdout().flush().context("failed to flush stdout")?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .context("failed to read setup input")?;
    let value = line.trim();
    if value.is_empty() {
        Ok(default.unwrap_or_default().to_owned())
    } else {
        Ok(value.to_owned())
    }
}

fn postgres_url(host: &str, port: u16, user: &str, password: &str, database: &str) -> String {
    format!(
        "postgresql://{}:{}@{}:{}/{}",
        urlencoding::encode(user),
        urlencoding::encode(password),
        host,
        port,
        urlencoding::encode(database)
    )
}

fn parse_database_url(value: &str) -> DatabaseUrlParts {
    let Some((_, rest)) = value.split_once("://") else {
        return DatabaseUrlParts::default();
    };
    let Some((userinfo, host_path)) = rest.split_once('@') else {
        return DatabaseUrlParts::default();
    };
    let (username, password) = userinfo.split_once(':').unwrap_or((userinfo, ""));
    let (host_port, path) = host_path.split_once('/').unwrap_or((host_path, ""));
    let database = path.split('?').next().filter(|value| !value.is_empty());
    let (host, port) = parse_host_port(host_port);
    DatabaseUrlParts {
        username: decode_url_part(username),
        password: decode_url_part(password),
        host,
        port,
        database: database.and_then(decode_url_part),
    }
}

fn mask_database_url(value: &str) -> String {
    let Some((scheme, rest)) = value.split_once("://") else {
        return value.to_owned();
    };
    let Some((userinfo, host_path)) = rest.split_once('@') else {
        return value.to_owned();
    };
    let username = userinfo
        .split_once(':')
        .map(|(username, _)| username)
        .unwrap_or(userinfo);
    format!("{scheme}://{username}:******@{host_path}")
}

fn parse_host_port(host_port: &str) -> (Option<String>, Option<u16>) {
    if let Some((host, port)) = host_port.rsplit_once(':')
        && let Ok(port) = port.parse::<u16>()
    {
        return (Some(host.trim_matches(['[', ']']).to_owned()), Some(port));
    }
    if host_port.is_empty() {
        (None, None)
    } else {
        (Some(host_port.trim_matches(['[', ']']).to_owned()), None)
    }
}

fn decode_url_part(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }
    urlencoding::decode(value)
        .ok()
        .map(|value| value.into_owned())
}

fn upsert_env_keys(content: &str, entries: &[(&str, &str)]) -> String {
    let mut lines = Vec::new();
    let mut replaced = BTreeMap::new();
    for (key, _) in entries {
        replaced.insert((*key).to_owned(), false);
    }

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            lines.push(line.to_owned());
            continue;
        }

        let Some((key, _)) = line.split_once('=') else {
            lines.push(line.to_owned());
            continue;
        };
        let key = key.trim();
        if let Some((_, value)) = entries.iter().find(|(entry_key, _)| *entry_key == key) {
            if !replaced.get(key).copied().unwrap_or(false) {
                lines.push(format!("{key}={value}"));
                replaced.insert(key.to_owned(), true);
            }
            continue;
        }

        lines.push(line.to_owned());
    }

    for (key, value) in entries {
        if replaced.get(*key).copied().unwrap_or(false) {
            continue;
        }
        if !lines.is_empty() && !lines.last().is_some_and(String::is_empty) {
            lines.push(String::new());
        }
        lines.push(format!("{key}={value}"));
        replaced.insert((*key).to_owned(), true);
    }

    let mut result = lines.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{
        DockerPsRow, host_port_for_container_port, is_minio_container, is_postgres_container,
        mask_database_url, parse_database_url, upsert_env_keys,
    };

    #[test]
    fn host_port_parser_finds_ipv4_and_ipv6_publish_lines() {
        let ports = "0.0.0.0:9091->9000/tcp, [::]:9091->9000/tcp, 0.0.0.0:9090->9001/tcp";

        assert_eq!(host_port_for_container_port(ports, 9000), Some(9091));
    }

    #[test]
    fn docker_row_detection_matches_pgvector_and_minio() {
        let postgres = DockerPsRow {
            image: "pgvector/pgvector:pg17".to_owned(),
            names: "pgvector".to_owned(),
            ..row()
        };
        let minio = DockerPsRow {
            image: "minio/minio:latest".to_owned(),
            names: "minio".to_owned(),
            ..row()
        };

        assert!(is_postgres_container(&postgres));
        assert!(is_minio_container(&minio));
    }

    #[test]
    fn database_url_parser_decodes_userinfo_and_database() {
        let parts = parse_database_url("postgresql://user:p%40ss@127.0.0.1:15432/aio");

        assert_eq!(parts.username.as_deref(), Some("user"));
        assert_eq!(parts.password.as_deref(), Some("p@ss"));
        assert_eq!(parts.port, Some(15432));
        assert_eq!(parts.database.as_deref(), Some("aio"));
    }

    #[test]
    fn env_upsert_replaces_once_and_appends_missing_keys() {
        let content = "A=old\nB=keep\nA=duplicate\n";
        let next = upsert_env_keys(content, &[("A", "new"), ("C", "value")]);

        assert!(next.contains("A=new\n"));
        assert!(next.contains("B=keep\n"));
        assert!(!next.contains("A=duplicate"));
        assert!(next.contains("C=value\n"));
    }

    #[test]
    fn database_url_mask_keeps_host_and_hides_password() {
        let masked = mask_database_url("postgresql://user:p%40ss@127.0.0.1:15432/aio");

        assert_eq!(masked, "postgresql://user:******@127.0.0.1:15432/aio");
    }

    fn row() -> DockerPsRow {
        DockerPsRow {
            id: String::new(),
            image: String::new(),
            names: String::new(),
            ports: String::new(),
            command: String::new(),
        }
    }
}
