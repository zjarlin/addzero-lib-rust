//! DI 容器：配置提供者。
//!
//! 命名空间约定：
//!   - `az-aio` - 项目业务配置（端口、功能开关、database、bucket 等）
//!   - `macmini-server` - 共享中间件配置（PG host/port/user/password、S3 endpoint/credentials）
//!
//! 读取顺序：项目命名空间优先，缺失时回退到共享命名空间，最终回退到环境变量。

use rudi::Singleton;
use std::path::PathBuf;

/// 应用配置接口。
pub trait AppConfig {
    /// 返回服务端口。
    fn port(&self) -> u16;
    /// 返回数据库连接 URL。
    fn database_url(&self) -> Option<String>;
}

/// Config-center 连接所需的环境变量。
pub struct ConfigCenterEnv {
    pub base_url: String,
    pub username: String,
    pub password: String,
}

impl ConfigCenterEnv {
    pub fn from_env() -> Self {
        Self {
            base_url: env_or_local("CONFIG_CENTER_BASE_URL").unwrap_or_default(),
            username: env_or_local("CONFIG_CENTER_USERNAME").unwrap_or_default(),
            password: env_or_local("CONFIG_CENTER_PASSWORD").unwrap_or_default(),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.base_url.is_empty() && !self.username.is_empty()
    }
}

/// 基于 config-center 的配置实现。
#[derive(Clone)]
#[Singleton]
pub struct ConfigCenterConfig;

impl AppConfig for ConfigCenterConfig {
    fn port(&self) -> u16 {
        let env = ConfigCenterEnv::from_env();
        if env.is_configured()
            && let Some(client) = login_center(&env)
            && let Some(port) = read_text_as_u16(&client, "web.port")
        {
            return port;
        }
        env_or_local("AZ_AIO_WEB_PORT")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(8080)
    }

    fn database_url(&self) -> Option<String> {
        let env = ConfigCenterEnv::from_env();
        if env.is_configured()
            && let Some(client) = login_center(&env)
        {
            // 项目命名空间 database.url 优先（完整 JDBC/URL）
            if let Ok(Some(val)) = client.get_text("database.url") {
                return Some(val);
            }
            // 回退：项目 database + 共享中间件 host/port/user/password → 拼接 PG URL
            if let Some(url) = compose_pg_url(&client) {
                return Some(url);
            }
        }
        env_or_local("AZ_AIO_DATABASE_URL")
    }
}

fn login_center(env: &ConfigCenterEnv) -> Option<az_config_center_client::ConfigCenterClient> {
    let client = az_config_center_client::ConfigCenterClient::new(&env.base_url).ok()?;
    let client = client.login(&env.username, &env.password).ok()?;
    let result = client.checkout_namespace("az-aio");
    result.ok()
}

fn read_text_as_u16(
    client: &az_config_center_client::ConfigCenterClient,
    key: &str,
) -> Option<u16> {
    match client.get_text(key) {
        Ok(Some(v)) => v.trim().parse().ok(),
        _ => None,
    }
}

/// 项目命名空间的 `database` + 共享命名空间的 PG 连接参数 → 完整 PostgreSQL URL。
///
/// 返回格式：`postgresql://user:password@host:port/database`
/// - `database` 从项目命名空间读取，缺失则用 `az_aio` 兜底
/// - host/port/user/password 从 `macmini-server` 共享命名空间读取
fn compose_pg_url(client: &az_config_center_client::ConfigCenterClient) -> Option<String> {
    let database = client
        .get_text("database")
        .ok()
        .flatten()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "az_aio".to_string());
    let shared = client.checkout_namespace("macmini-server").ok()?;
    let host = shared.get_text("postgres.host").ok().flatten()?;
    let port = shared
        .get_text("postgres.port")
        .ok()
        .flatten()
        .unwrap_or_else(|| "5432".to_string());
    let user = shared
        .get_text("postgres.user")
        .ok()
        .flatten()
        .unwrap_or_else(|| "postgres".to_string());
    let password = shared
        .get_text("postgres.password")
        .ok()
        .flatten()
        .unwrap_or_default();
    Some(format!(
        "postgresql://{user}:{password}@{host}:{port}/{database}"
    ))
}

fn env_or_local(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| local_env_value(key))
}

fn local_env_value(key: &str) -> Option<String> {
    local_env_paths().into_iter().find_map(|path| {
        let content = std::fs::read_to_string(path).ok()?;
        parse_env_value(&content, key)
    })
}

fn local_env_paths() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dot_config_path = home
        .join(".config")
        .join("addzero")
        .join("az-aio")
        .join("az-aio.env");
    let native_config_path = dirs::config_dir()
        .unwrap_or_else(|| home.join(".config"))
        .join("addzero")
        .join("az-aio")
        .join("az-aio.env");
    if native_config_path == dot_config_path {
        vec![dot_config_path]
    } else {
        vec![dot_config_path, native_config_path]
    }
}

fn parse_env_value(content: &str, key: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let (raw_key, raw_value) = line.split_once('=')?;
        let raw_key = raw_key.trim().strip_prefix("export ").unwrap_or(raw_key.trim());
        if raw_key != key {
            return None;
        }
        Some(unquote_env_value(raw_value.trim()).to_string())
    })
}

fn unquote_env_value(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_other_env_keys() {

    }

    #[test]
    fn local_env_paths_prefer_dot_config_on_macos() {
    }
}
