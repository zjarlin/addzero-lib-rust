use std::env;

use anyhow::{bail, Result};

pub const DATABASE_URL_ENV: &str = "AZ_AIO_LOWCODE_DATABASE_URL";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LowcodeConfig {
    pub database_url: String,
}

pub fn resolve_lowcode_config() -> Result<LowcodeConfig> {
    let database_url = env::var(DATABASE_URL_ENV)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "missing lowcode database url: set {DATABASE_URL_ENV} environment variable"
            )
        })?;
    Ok(LowcodeConfig { database_url })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_pin_database_config_key() {
        assert_eq!(DATABASE_URL_ENV, "AZ_AIO_LOWCODE_DATABASE_URL");
    }
}
