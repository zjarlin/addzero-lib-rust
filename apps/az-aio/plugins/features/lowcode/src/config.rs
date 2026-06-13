#![cfg(not(target_arch = "wasm32"))]

use std::env;

use az_config_center_client::ConfigCenterClient;

use crate::error::{LowcodeError, LowcodeResult};

pub const LOWCODE_CONFIG_NAMESPACE: &str = "az-aio.dev";
pub const DATABASE_URL_CONFIG_KEY: &str = "lowcode.database_url";
pub const DATABASE_URL_ENV: &str = "AZ_AIO_LOWCODE_DATABASE_URL";
pub const CONFIG_CENTER_BASE_URL_ENV: &str = "AZ_CONFIG_CENTER_BASE_URL";
pub const CONFIG_CENTER_USERNAME_ENV: &str = "AZ_CONFIG_CENTER_USERNAME";
pub const CONFIG_CENTER_PASSWORD_ENV: &str = "AZ_CONFIG_CENTER_PASSWORD";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LowcodeConfig {
    pub database_url: String,
    pub source: LowcodeConfigSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LowcodeConfigSource {
    ConfigCenter,
    Environment,
}

pub fn resolve_lowcode_config() -> LowcodeResult<LowcodeConfig> {
    if let Some(config) = read_config_center_database_url()? {
        return Ok(config);
    }
    env_database_url()
        .map(|database_url| LowcodeConfig {
            database_url,
            source: LowcodeConfigSource::Environment,
        })
        .ok_or(LowcodeError::MissingDatabaseUrl)
}

fn read_config_center_database_url() -> LowcodeResult<Option<LowcodeConfig>> {
    let Some(base_url) = env_value(CONFIG_CENTER_BASE_URL_ENV) else {
        return Ok(None);
    };
    let Some(username) = env_value(CONFIG_CENTER_USERNAME_ENV) else {
        return Ok(None);
    };
    let Some(password) = env_value(CONFIG_CENTER_PASSWORD_ENV) else {
        return Ok(None);
    };
    let client = ConfigCenterClient::new(base_url)?
        .login(username, password)?
        .checkout_namespace(LOWCODE_CONFIG_NAMESPACE)?;
    Ok(client
        .get_secret(DATABASE_URL_CONFIG_KEY)?
        .and_then(normalize_optional)
        .map(|database_url| LowcodeConfig {
            database_url,
            source: LowcodeConfigSource::ConfigCenter,
        }))
}

fn env_database_url() -> Option<String> {
    env_value(DATABASE_URL_ENV)
}

fn env_value(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_optional(value: String) -> Option<String> {
    let value = value.trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_pin_database_config_to_az_aio_dev_namespace() {
        assert_eq!(LOWCODE_CONFIG_NAMESPACE, "az-aio.dev");
        assert_eq!(DATABASE_URL_CONFIG_KEY, "lowcode.database_url");
    }
}
