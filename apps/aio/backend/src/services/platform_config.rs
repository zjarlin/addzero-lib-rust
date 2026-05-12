use std::{collections::BTreeMap, fs, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use sqlx::{Connection, Executor, postgres::PgConnection};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformConfigDto {
    pub config_path: String,
    pub postgres: PostgresConfigDto,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostgresConfigDto {
    pub database_url: String,
    pub configured: bool,
    pub reachable: bool,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostgresConfigUpdateDto {
    pub database_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformConfigSaveResultDto {
    pub config: PlatformConfigDto,
    pub message: String,
}

const POSTGRES_KEYS: &[&str] = &["MSC_AIO_DATABASE_URL", "DATABASE_URL"];
const LEGACY_OBJECT_STORAGE_KEYS: &[&str] = &[
    "AIO_MINIO_ENDPOINT",
    "AIO_MINIO_ACCESS_KEY",
    "AIO_MINIO_SECRET_KEY",
    "AIO_MINIO_REGION",
    "AIO_MINIO_BUCKET",
    "AIO_MINIO_SHARE_SECRET",
    "AIO_MINIO_SHARE_EXPIRES_SECONDS",
    "MINIO_URL_ENCRYPTION_SECRET",
];

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

    let mut updates = vec![("MSC_AIO_DATABASE_URL", Some(database_url))];
    updates.extend(
        LEGACY_OBJECT_STORAGE_KEYS
            .iter()
            .copied()
            .map(|key| (key, None)),
    );
    write_local_env(&updates)?;
    let config = load_platform_config_on_server().await?;
    Ok(PlatformConfigSaveResultDto {
        config,
        message: "PostgreSQL 配置已写入 ~/.config/aio/aio.env。重启后所有 PG 能力会使用新连接。"
            .to_string(),
    })
}

pub fn cleanup_legacy_object_storage_config_on_server() -> Result<PathBuf, String> {
    let updates = LEGACY_OBJECT_STORAGE_KEYS
        .iter()
        .copied()
        .map(|key| (key, None))
        .collect::<Vec<_>>();
    write_local_env(&updates)
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

    Ok(PlatformConfigDto {
        config_path: path.display().to_string(),
        postgres,
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
        } else {
            content = remove_env_key(&content, key);
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

fn local_config_path() -> Result<PathBuf, String> {
    az_persistence::local_env_path().ok_or_else(|| "无法定位 ~/.config/aio/aio.env".to_string())
}

fn remove_env_key(content: &str, key: &str) -> String {
    let lines = content
        .lines()
        .filter(|line| {
            line.split_once('=')
                .map(|(current_key, _)| current_key.trim() != key)
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    if lines.is_empty() {
        String::new()
    } else {
        let mut output = lines.join("\n");
        output.push('\n');
        output
    }
}

fn is_postgres_url(database_url: &str) -> bool {
    database_url.starts_with("postgres://") || database_url.starts_with("postgresql://")
}
