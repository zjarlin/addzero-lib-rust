use std::{
    fs,
    path::PathBuf,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use sqlx::{Connection, Executor, postgres::PgConnection};

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

pub async fn bootstrap_status_on_server() -> Result<BootstrapStatusDto, String> {
    let config_path = local_config_path()?;
    let config_path_display = config_path.display().to_string();
    let Some(database_url) = addzero_persistence::database_url() else {
        return Ok(BootstrapStatusDto {
            desktop_mode: true,
            setup_required: true,
            database_configured: false,
            database_reachable: false,
            config_path: config_path_display,
            message: "首次启动需要先配置 PostgreSQL 地址。".to_string(),
        });
    };

    match ping_database(&database_url).await {
        Ok(()) => Ok(BootstrapStatusDto {
            desktop_mode: true,
            setup_required: false,
            database_configured: true,
            database_reachable: true,
            config_path: config_path_display,
            message: "PostgreSQL 已就绪，桌面端可直接进入工作台。".to_string(),
        }),
        Err(err) => Ok(BootstrapStatusDto {
            desktop_mode: true,
            setup_required: true,
            database_configured: true,
            database_reachable: false,
            config_path: config_path_display,
            message: format!("当前 PostgreSQL 配置不可用：{err}"),
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
        message: "PostgreSQL 地址已保存到本机配置，重新进入登录即可使用。".to_string(),
    })
}

async fn ping_database(database_url: &str) -> Result<(), String> {
    let connect_future = PgConnection::connect(database_url);
    let mut connection = tokio::time::timeout(Duration::from_secs(5), connect_future)
        .await
        .map_err(|_| "连接超时".to_string())?
        .map_err(|err| err.to_string())?;

    tokio::time::timeout(
        Duration::from_secs(5),
        connection.execute("SELECT 1"),
    )
    .await
    .map_err(|_| "健康检查超时".to_string())?
    .map_err(|err| err.to_string())?;

    connection.close().await.map_err(|err| err.to_string())
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
    addzero_persistence::local_env_path()
        .ok_or_else(|| "无法定位 ~/.config/aio/aio.env".to_string())
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
