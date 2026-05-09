use std::{collections::BTreeMap, fs, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use sqlx::{Connection, Executor, postgres::PgConnection};

#[cfg(not(target_arch = "wasm32"))]
use az_minio::{MinioConfig, create_client};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformConfigDto {
    pub config_path: String,
    pub postgres: PostgresConfigDto,
    pub minio: MinioConfigDto,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostgresConfigDto {
    pub database_url: String,
    pub configured: bool,
    pub reachable: bool,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinioConfigDto {
    pub endpoint: String,
    pub access_key: String,
    pub secret_configured: bool,
    pub region: String,
    pub bucket: String,
    pub configured: bool,
    pub reachable: bool,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostgresConfigUpdateDto {
    pub database_url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinioConfigUpdateDto {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: Option<String>,
    pub region: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformConfigSaveResultDto {
    pub config: PlatformConfigDto,
    pub message: String,
}

const POSTGRES_KEYS: &[&str] = &["MSC_AIO_DATABASE_URL", "DATABASE_URL"];
const MINIO_ENDPOINT_KEYS: &[&str] = &["AIO_MINIO_ENDPOINT"];
const MINIO_ACCESS_KEY_KEYS: &[&str] = &["AIO_MINIO_ACCESS_KEY"];
const MINIO_SECRET_KEY_KEYS: &[&str] = &["AIO_MINIO_SECRET_KEY"];
const MINIO_REGION_KEYS: &[&str] = &["AIO_MINIO_REGION"];
const AIO_BUCKET_NAME: &str = "aio";

pub async fn load_platform_config_on_server() -> Result<PlatformConfigDto, String> {
    let path = local_config_path()?;
    let values = env_values()?;
    build_platform_config(path, values).await
}

pub async fn save_postgres_config_on_server(
    input: PostgresConfigUpdateDto,
) -> Result<PlatformConfigSaveResultDto, String> {
    let database_url = input.database_url.trim();
    if database_url.is_empty() {
        return Err("PostgreSQL 地址不能为空。".to_string());
    }
    if !is_postgres_url(database_url) {
        return Err("PostgreSQL 地址必须以 postgres:// 或 postgresql:// 开头。".to_string());
    }
    ping_postgres(database_url)
        .await
        .map_err(|err| format!("PostgreSQL 连接测试失败：{err}"))?;

    write_local_env(&[("MSC_AIO_DATABASE_URL", Some(database_url))])?;
    let config = load_platform_config_on_server().await?;
    Ok(PlatformConfigSaveResultDto {
        config,
        message: "PostgreSQL 配置已写入 ~/.config/aio/aio.env。重启后所有 PG 能力会使用新连接。"
            .to_string(),
    })
}

pub async fn save_minio_config_on_server(
    input: MinioConfigUpdateDto,
) -> Result<PlatformConfigSaveResultDto, String> {
    let endpoint = normalized_required("MinIO endpoint", &input.endpoint)?;
    let access_key = normalized_required("MinIO access key", &input.access_key)?;
    let secret_key = input.secret_key.as_deref().unwrap_or("").trim().to_string();
    let existing_values = env_values()?;
    let secret_key = if secret_key.is_empty() {
        read_value_any(&existing_values, MINIO_SECRET_KEY_KEYS)
            .ok_or_else(|| "MinIO secret key 不能为空。".to_string())?
    } else {
        secret_key
    };
    let region = input
        .region
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("us-east-1")
        .to_string();

    ping_minio(&endpoint, &access_key, &secret_key, &region)
        .await
        .map_err(|err| format!("MinIO 连接测试失败：{err}"))?;

    write_local_env(&[
        ("AIO_MINIO_ENDPOINT", Some(endpoint.as_str())),
        ("AIO_MINIO_ACCESS_KEY", Some(access_key.as_str())),
        ("AIO_MINIO_SECRET_KEY", Some(secret_key.as_str())),
        ("AIO_MINIO_REGION", Some(region.as_str())),
        ("AIO_MINIO_BUCKET", Some(AIO_BUCKET_NAME)),
    ])?;
    let config = load_platform_config_on_server().await?;
    Ok(PlatformConfigSaveResultDto {
        config,
        message: "MinIO 配置已写入 ~/.config/aio/aio.env。重启后对象存储与插件包会使用新连接。"
            .to_string(),
    })
}

async fn build_platform_config(
    path: PathBuf,
    values: BTreeMap<String, String>,
) -> Result<PlatformConfigDto, String> {
    let database_url = read_value_any(&values, POSTGRES_KEYS).unwrap_or_default();
    let postgres = if database_url.trim().is_empty() {
        PostgresConfigDto {
            configured: false,
            reachable: false,
            message: "未配置 PostgreSQL。".to_string(),
            ..PostgresConfigDto::default()
        }
    } else {
        match ping_postgres(&database_url).await {
            Ok(()) => PostgresConfigDto {
                database_url,
                configured: true,
                reachable: true,
                message: "PostgreSQL 连接可用。".to_string(),
            },
            Err(err) => PostgresConfigDto {
                database_url,
                configured: true,
                reachable: false,
                message: format!("PostgreSQL 连接不可用：{err}"),
            },
        }
    };

    let endpoint = read_value_any(&values, MINIO_ENDPOINT_KEYS).unwrap_or_default();
    let access_key = read_value_any(&values, MINIO_ACCESS_KEY_KEYS).unwrap_or_default();
    let secret_key = read_value_any(&values, MINIO_SECRET_KEY_KEYS).unwrap_or_default();
    let region =
        read_value_any(&values, MINIO_REGION_KEYS).unwrap_or_else(|| "us-east-1".to_string());
    let minio_configured = !endpoint.trim().is_empty()
        && !access_key.trim().is_empty()
        && !secret_key.trim().is_empty();
    let minio = if minio_configured {
        match ping_minio(&endpoint, &access_key, &secret_key, &region).await {
            Ok(()) => MinioConfigDto {
                endpoint,
                access_key,
                secret_configured: true,
                region,
                bucket: AIO_BUCKET_NAME.to_string(),
                configured: true,
                reachable: true,
                message: "MinIO 连接可用。".to_string(),
            },
            Err(err) => MinioConfigDto {
                endpoint,
                access_key,
                secret_configured: true,
                region,
                bucket: AIO_BUCKET_NAME.to_string(),
                configured: true,
                reachable: false,
                message: format!("MinIO 连接不可用：{err}"),
            },
        }
    } else {
        MinioConfigDto {
            endpoint,
            access_key,
            secret_configured: false,
            region,
            bucket: AIO_BUCKET_NAME.to_string(),
            configured: false,
            reachable: false,
            message: "未配置 MinIO。".to_string(),
        }
    };

    Ok(PlatformConfigDto {
        config_path: path.display().to_string(),
        postgres,
        minio,
    })
}

async fn ping_postgres(database_url: &str) -> Result<(), String> {
    let connect_future = PgConnection::connect(database_url);
    let mut connection = tokio::time::timeout(Duration::from_secs(5), connect_future)
        .await
        .map_err(|_| "连接超时".to_string())?
        .map_err(|err| err.to_string())?;
    tokio::time::timeout(Duration::from_secs(5), connection.execute("SELECT 1"))
        .await
        .map_err(|_| "健康检查超时".to_string())?
        .map_err(|err| err.to_string())?;
    connection.close().await.map_err(|err| err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
async fn ping_minio(
    endpoint: &str,
    access_key: &str,
    secret_key: &str,
    region: &str,
) -> Result<(), String> {
    let endpoint = endpoint.to_string();
    let access_key = access_key.to_string();
    let secret_key = secret_key.to_string();
    let region = region.to_string();
    tokio::task::spawn_blocking(move || {
        ping_minio_blocking(&endpoint, &access_key, &secret_key, &region)
    })
    .await
    .map_err(|err| format!("MinIO 连接测试任务失败：{err}"))?
}

#[cfg(not(target_arch = "wasm32"))]
fn ping_minio_blocking(
    endpoint: &str,
    access_key: &str,
    secret_key: &str,
    region: &str,
) -> Result<(), String> {
    let config = MinioConfig::builder(
        endpoint.to_string(),
        access_key.to_string(),
        secret_key.to_string(),
    )
    .region(region.to_string())
    .build()
    .map_err(|err| err.to_string())?;
    let client = create_client(config).map_err(|err| err.to_string())?;
    client
        .ensure_bucket(AIO_BUCKET_NAME)
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn ping_minio(
    _endpoint: &str,
    _access_key: &str,
    _secret_key: &str,
    _region: &str,
) -> Result<(), String> {
    Err("MinIO 配置只能在本机后端验证。".to_string())
}

fn env_values() -> Result<BTreeMap<String, String>, String> {
    let mut values = BTreeMap::new();
    if let Some(path) = az_persistence::local_env_path() {
        values.extend(read_env_file(&path)?);
    }
    for (key, value) in std::env::vars() {
        if !value.trim().is_empty() {
            values.insert(key, value);
        }
    }
    Ok(values)
}

fn read_env_file(path: &PathBuf) -> Result<BTreeMap<String, String>, String> {
    let Ok(content) = fs::read_to_string(path) else {
        return Ok(BTreeMap::new());
    };
    Ok(parse_env_pairs(&content))
}

fn parse_env_pairs(content: &str) -> BTreeMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn write_local_env(updates: &[(&str, Option<&str>)]) -> Result<PathBuf, String> {
    let path = local_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建配置目录失败：{err}"))?;
    }

    let existing = fs::read_to_string(&path).unwrap_or_default();
    let mut content = existing;
    for (key, value) in updates {
        if let Some(value) = value {
            content = upsert_env_key(&content, key, value);
        }
    }
    fs::write(&path, content).map_err(|err| format!("写入配置文件失败：{err}"))?;
    Ok(path)
}

fn upsert_env_key(content: &str, key: &str, value: &str) -> String {
    let mut found = false;
    let mut lines = content
        .lines()
        .map(|line| {
            let Some((current_key, _)) = line.split_once('=') else {
                return line.to_string();
            };
            if current_key.trim() == key {
                found = true;
                format!("{key}={value}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();

    if !found {
        lines.push(format!("{key}={value}"));
    }
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

fn read_value_any(values: &BTreeMap<String, String>, names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| values.get(*name))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalized_required(label: &str, value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(format!("{label} 不能为空。"))
    } else {
        Ok(trimmed.to_string())
    }
}

fn local_config_path() -> Result<PathBuf, String> {
    az_persistence::local_env_path().ok_or_else(|| "无法定位 ~/.config/aio/aio.env".to_string())
}

fn is_postgres_url(database_url: &str) -> bool {
    database_url.starts_with("postgres://") || database_url.starts_with("postgresql://")
}
