use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use sqlx::{
    Connection, Executor,
    postgres::PgConnection,
    sqlite::{SqliteConnectOptions, SqliteConnection},
};

use super::platform_config::{MinioConfigUpdateDto, PlatformConfigDto, PostgresConfigUpdateDto};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapStatusDto {
    pub desktop_mode: bool,
    pub setup_required: bool,
    pub database_configured: bool,
    pub database_reachable: bool,
    pub config_path: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapDatabaseSetupDto {
    pub database_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapDatabaseSaveResultDto {
    pub database_configured: bool,
    pub database_reachable: bool,
    pub config_path: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapPlatformSetupDto {
    pub postgres: Option<PostgresConfigUpdateDto>,
    pub minio: Option<MinioConfigUpdateDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapPlatformSaveResultDto {
    pub config: PlatformConfigDto,
    pub setup_required: bool,
    pub message: String,
}

pub async fn bootstrap_status_on_server() -> Result<BootstrapStatusDto, String> {
    let config_path = local_config_path()?;
    let config_path_display = config_path.display().to_string();
    let Some(database_url) = local_database_url(&config_path) else {
        return Ok(BootstrapStatusDto {
            desktop_mode: true,
            setup_required: true,
            database_configured: false,
            database_reachable: false,
            config_path: config_path_display,
            message: "首次启动先生成 ~/.config/aio/aio.env。可以接 PostgreSQL / MinIO，也可以先使用本机 SQLite。"
                .to_string(),
        });
    };

    match ping_database(&database_url).await {
        Ok(()) => Ok(BootstrapStatusDto {
            desktop_mode: true,
            setup_required: false,
            database_configured: true,
            database_reachable: true,
            config_path: config_path_display,
            message: ready_message(&database_url),
        }),
        Err(err) => Ok(BootstrapStatusDto {
            desktop_mode: true,
            setup_required: true,
            database_configured: true,
            database_reachable: false,
            config_path: config_path_display,
            message: format!(
                "当前{}配置不可用：{err}",
                database_kind_label(&database_url)
            ),
        }),
    }
}

pub async fn save_database_url_on_server(
    input: BootstrapDatabaseSetupDto,
) -> Result<BootstrapDatabaseSaveResultDto, String> {
    let database_url = input.database_url.trim();
    if database_url.is_empty() {
        return Err("PostgreSQL 地址不能为空。".to_string());
    }
    if !(database_url.starts_with("postgres://") || database_url.starts_with("postgresql://")) {
        return Err("PostgreSQL 地址必须以 postgres:// 或 postgresql:// 开头。".to_string());
    }

    ping_database(database_url)
        .await
        .map_err(|err| format!("PostgreSQL 连接测试失败：{err}"))?;

    let path = write_local_database_url(database_url)?;
    Ok(BootstrapDatabaseSaveResultDto {
        database_configured: true,
        database_reachable: true,
        config_path: path.display().to_string(),
        message: "PostgreSQL 地址已保存到 ~/.config/aio/aio.env，重新进入登录即可使用。"
            .to_string(),
    })
}

pub async fn save_platform_setup_on_server(
    input: BootstrapPlatformSetupDto,
) -> Result<BootstrapPlatformSaveResultDto, String> {
    let mut messages = Vec::new();
    if let Some(postgres) = input.postgres {
        super::platform_config::save_postgres_config_on_server(postgres).await?;
        messages.push("PostgreSQL 已保存");
    }
    if let Some(minio) = input.minio {
        super::platform_config::save_minio_config_on_server(minio).await?;
        messages.push("MinIO 已保存");
    }
    if messages.is_empty() {
        return Err("至少需要保存 PostgreSQL 或 MinIO 中的一项配置。".to_string());
    }

    let config = super::platform_config::load_platform_config_on_server().await?;
    Ok(BootstrapPlatformSaveResultDto {
        setup_required: !config.postgres.reachable,
        config,
        message: format!(
            "{}。配置已写入 ~/.config/aio/aio.env。",
            messages.join("，")
        ),
    })
}

pub async fn save_local_sqlite_on_server() -> Result<BootstrapDatabaseSaveResultDto, String> {
    let sqlite_path = local_sqlite_database_path()?;
    if let Some(parent) = sqlite_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建 SQLite 目录失败：{err}"))?;
    }

    let database_url = sqlite_database_url(&sqlite_path);
    ping_database(&database_url)
        .await
        .map_err(|err| format!("初始化本机 SQLite 失败：{err}"))?;

    let path = write_local_database_url(&database_url)?;
    Ok(BootstrapDatabaseSaveResultDto {
        database_configured: true,
        database_reachable: true,
        config_path: path.display().to_string(),
        message: format!(
            "已切换到本机内嵌 SQLite，数据文件位于 {}。",
            sqlite_path.display()
        ),
    })
}

fn local_database_url(config_path: &Path) -> Option<String> {
    let content = fs::read_to_string(config_path).ok()?;
    env_value(&content, "MSC_AIO_DATABASE_URL")
        .or_else(|| env_value(&content, "DATABASE_URL"))
        .filter(|value| !value.trim().is_empty())
}

fn env_value(content: &str, expected_key: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }
        let (key, value) = trimmed.split_once('=')?;
        if key.trim() == expected_key {
            Some(value.trim().to_string())
        } else {
            None
        }
    })
}

async fn ping_database(database_url: &str) -> Result<(), String> {
    if is_postgres_url(database_url) {
        let connect_future = PgConnection::connect(database_url);
        let mut connection = tokio::time::timeout(Duration::from_secs(5), connect_future)
            .await
            .map_err(|_| "连接超时".to_string())?
            .map_err(|err| err.to_string())?;

        tokio::time::timeout(Duration::from_secs(5), connection.execute("SELECT 1"))
            .await
            .map_err(|_| "健康检查超时".to_string())?
            .map_err(|err| err.to_string())?;

        return connection.close().await.map_err(|err| err.to_string());
    }

    if is_sqlite_url(database_url) {
        ensure_sqlite_parent_dir(database_url)?;

        let connect_options = SqliteConnectOptions::from_str(database_url)
            .map_err(|err| format!("解析 SQLite 地址失败：{err}"))?
            .create_if_missing(true)
            .foreign_keys(true);
        let connect_future = SqliteConnection::connect_with(&connect_options);
        let mut connection = tokio::time::timeout(Duration::from_secs(5), connect_future)
            .await
            .map_err(|_| "连接超时".to_string())?
            .map_err(|err| err.to_string())?;

        tokio::time::timeout(Duration::from_secs(5), connection.execute("SELECT 1"))
            .await
            .map_err(|_| "健康检查超时".to_string())?
            .map_err(|err| err.to_string())?;

        return connection.close().await.map_err(|err| err.to_string());
    }

    Err("仅支持 postgres://、postgresql:// 或 sqlite: 数据库地址。".to_string())
}

fn write_local_database_url(database_url: &str) -> Result<PathBuf, String> {
    let path = local_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建配置目录失败：{err}"))?;
    }

    let existing = fs::read_to_string(&path).unwrap_or_default();
    let content = upsert_env_key(&existing, "MSC_AIO_DATABASE_URL", database_url);
    fs::write(&path, content).map_err(|err| format!("写入配置文件失败：{err}"))?;
    Ok(path)
}

fn local_config_path() -> Result<PathBuf, String> {
    az_persistence::local_env_path().ok_or_else(|| "无法定位 ~/.config/aio/aio.env".to_string())
}

fn local_sqlite_database_path() -> Result<PathBuf, String> {
    let config_path = local_config_path()?;
    let Some(parent) = config_path.parent() else {
        return Err("无法定位 ~/.config/aio 目录".to_string());
    };
    Ok(parent.join("aio.sqlite3"))
}

fn ready_message(database_url: &str) -> String {
    if is_sqlite_url(database_url) {
        "本机内嵌 SQLite 已就绪，桌面端可直接进入工作台。".to_string()
    } else {
        "PostgreSQL 已就绪，桌面端可直接进入工作台。".to_string()
    }
}

fn database_kind_label(database_url: &str) -> &'static str {
    if is_sqlite_url(database_url) {
        "本机 SQLite"
    } else {
        "PostgreSQL"
    }
}

fn sqlite_database_url(path: &Path) -> String {
    format!("sqlite://{}", path.display())
}

fn ensure_sqlite_parent_dir(database_url: &str) -> Result<(), String> {
    let Some(path) = sqlite_file_path(database_url) else {
        return Ok(());
    };
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent).map_err(|err| format!("创建 SQLite 目录失败：{err}"))
}

fn sqlite_file_path(database_url: &str) -> Option<PathBuf> {
    if database_url == "sqlite::memory:" {
        return None;
    }

    database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn is_postgres_url(database_url: &str) -> bool {
    database_url.starts_with("postgres://") || database_url.starts_with("postgresql://")
}

fn is_sqlite_url(database_url: &str) -> bool {
    database_url.starts_with("sqlite:")
}

fn upsert_env_key(content: &str, key: &str, value: &str) -> String {
    let mut lines = Vec::new();
    let mut replaced = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            lines.push(line.to_string());
            continue;
        }

        let Some((current_key, _)) = line.split_once('=') else {
            lines.push(line.to_string());
            continue;
        };

        if current_key.trim() == key {
            if !replaced {
                lines.push(format!("{key}={value}"));
                replaced = true;
            }
            continue;
        }

        lines.push(line.to_string());
    }

    if !replaced {
        if !lines.is_empty() && !lines.last().is_some_and(|line| line.is_empty()) {
            lines.push(String::new());
        }
        lines.push(format!("{key}={value}"));
    }

    let mut result = lines.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result
}
