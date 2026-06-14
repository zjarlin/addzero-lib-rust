//! DI 容器：环境变量驱动的配置提供者。
//!
//! 从 config-center 或环境变量读取运行时配置。
//! 失败时回退到 `AZ_AIO_WEB_PORT` 环境变量，最终默认 8080。

use shaku::{Component, Interface};

/// 应用配置接口。
pub trait AppConfig: Interface {
    /// 返回服务端口。
    fn port(&self) -> u16;
}

/// 基于 config-center 的配置实现。
#[derive(Component)]
#[shaku(interface = AppConfig)]
pub struct ConfigCenterConfig;

impl AppConfig for ConfigCenterConfig {
    fn port(&self) -> u16 {
        let base_url = std::env::var("AZ_CONFIG_CENTER_BASE_URL").unwrap_or_default();
        let username = std::env::var("AZ_CONFIG_CENTER_USERNAME").unwrap_or_default();
        let password = std::env::var("AZ_CONFIG_CENTER_PASSWORD").unwrap_or_default();

        if !base_url.is_empty() && !username.is_empty() {
            if let Some(port) = read_port_from_center(&base_url, &username, &password) {
                return port;
            }
        }

        std::env::var("AZ_AIO_WEB_PORT")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(8080)
    }
}

fn read_port_from_center(base_url: &str, username: &str, password: &str) -> Option<u16> {
    let client = az_config_center_client::ConfigCenterClient::new(base_url).ok()?;
    let client = client.login(username, password).ok()?;
    let client = client.checkout_namespace("az-aio.dev").ok()?;
    let value: String = client.get_text("web.port").ok()??;
    value.trim().parse().ok()
}
