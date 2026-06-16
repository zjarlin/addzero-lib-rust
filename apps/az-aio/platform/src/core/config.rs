//! DI 容器：配置提供者。
//!
//! 命名空间约定：
//!   - `az-aio.dev` - 项目业务配置（端口、功能开关、database、bucket 等）
//!   - `macmini-server` - 共享中间件配置（PG host/port/user/password、S3 endpoint/credentials）
//!
//! 读取顺序：项目命名空间优先，缺失时回退到共享命名空间，最终回退到环境变量。

use rudi::Singleton;

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
            base_url: std::env::var("CONFIG_CENTER_BASE_URL").unwrap_or_default(),
            username: std::env::var("CONFIG_CENTER_USERNAME").unwrap_or_default(),
            password: std::env::var("CONFIG_CENTER_PASSWORD").unwrap_or_default(),
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
        if env.is_configured() {
            if let Some(client) = login_center(&env) {
                if let Some(port) = read_text_as_u16(&client, "web.port") {
                    return port;
                }
            }
        }
        std::env::var("AZ_AIO_WEB_PORT")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(8080)
    }

    fn database_url(&self) -> Option<String> {
        let env = ConfigCenterEnv::from_env();
        if env.is_configured() {
            if let Some(client) = login_center(&env) {
                // 项目命名空间 database.url 优先（完整 JDBC/URL）
                if let Ok(Some(val)) = client.get_text("database.url") {
                    return Some(val);
                }
                // 回退：项目 database + 共享中间件 host/port/user/password → 拼接 PG URL
                if let Some(url) = compose_pg_url(&client) {
                    return Some(url);
                }
            }
        }
        std::env::var("AZ_AIO_DATABASE_URL").ok()
    }
}

fn login_center(env: &ConfigCenterEnv) -> Option<az_config_center_client::ConfigCenterClient> {
    let client = az_config_center_client::ConfigCenterClient::new(&env.base_url).ok()?;
    let client = client.login(&env.username, &env.password).ok()?;
    let result = client.checkout_namespace("az-aio.dev");
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
/// - `database` 从项目命名空间读取，缺失则用 `cmp_aio` 兜底
/// - host/port/user/password 从 `macmini-server` 共享命名空间读取
fn compose_pg_url(client: &az_config_center_client::ConfigCenterClient) -> Option<String> {
    let database = client
        .get_text("database")
        .ok()
        .flatten()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "cmp_aio".to_string());
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
