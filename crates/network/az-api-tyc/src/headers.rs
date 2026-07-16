//! Request header construction for Tianyancha API calls.

use std::collections::BTreeMap;

use anyhow::Context;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

/// Environment variable read by [`TycCredentials::from_env`] for the `Authorization` header.
pub const TYC_AUTHORIZATION_ENV: &str = "TYC_AUTHORIZATION";

/// Environment variable read by [`TycCredentials::from_env`] for the `X-AUTH-TOKEN` header.
pub const TYC_X_AUTH_TOKEN_ENV: &str = "TYC_X_AUTH_TOKEN";

/// Explicit credentials required by the Tianyancha mini-program API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TycCredentials {
    /// Value sent as the `Authorization` header.
    pub authorization: String,
    /// Value sent as the `X-AUTH-TOKEN` header.
    pub x_auth_token: String,
}

impl TycCredentials {
    /// Builds credentials from explicit header values.
    pub fn new(
        authorization: impl Into<String>,
        x_auth_token: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let credentials = Self {
            authorization: authorization.into(),
            x_auth_token: x_auth_token.into(),
        };
        credentials.validate()?;
        Ok(credentials)
    }

    /// Reads credentials from `TYC_AUTHORIZATION` and `TYC_X_AUTH_TOKEN`.
    pub fn from_env() -> anyhow::Result<Self> {
        let authorization = std::env::var(TYC_AUTHORIZATION_ENV)
            .with_context(|| format!("missing environment variable `{TYC_AUTHORIZATION_ENV}`"))?;
        let x_auth_token = std::env::var(TYC_X_AUTH_TOKEN_ENV)
            .with_context(|| format!("missing environment variable `{TYC_X_AUTH_TOKEN_ENV}`"))?;
        Self::new(authorization, x_auth_token)
    }

    /// Checks that both required header values are present.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.authorization.trim().is_empty() {
            anyhow::bail!("invalid Tianyancha credentials: authorization cannot be blank");
        }
        if self.x_auth_token.trim().is_empty() {
            anyhow::bail!("invalid Tianyancha credentials: x_auth_token cannot be blank");
        }
        Ok(())
    }
}

/// Returns the browser/mini-program style headers required by the upstream API.
pub fn tyc_headers(credentials: &TycCredentials) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "Authorization".to_owned(),
            credentials.authorization.clone(),
        ),
        ("host".to_owned(), "api9.tianyancha.com".to_owned()),
        ("Content-Type".to_owned(), "application/json".to_owned()),
        ("X-AUTH-TOKEN".to_owned(), credentials.x_auth_token.clone()),
        ("Accept".to_owned(), "*/*".to_owned()),
        ("version".to_owned(), "TYC-XCX-WX".to_owned()),
        (
            "User-Agent".to_owned(),
            "Mozilla/5.0 (iPhone; CPU iPhone OS 12_1_4 like Mac OS X) \
             AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/16D57 \
             MicroMessenger/7.0.5(0x17000523) NetType/WIFI Language/zh_CN"
                .to_owned(),
        ),
        ("Accept-Language".to_owned(), "zh-cn".to_owned()),
    ])
}

pub(crate) fn to_header_map(headers: &BTreeMap<String, String>) -> anyhow::Result<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .with_context(|| format!("invalid Tianyancha header name `{name}`"))?;
        let header_value = HeaderValue::from_str(value)
            .with_context(|| format!("invalid Tianyancha header value for `{name}`"))?;
        header_map.insert(header_name, header_value);
    }
    Ok(header_map)
}
