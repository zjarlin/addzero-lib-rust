use crate::{TempMailError, TempMailResult};
use az_derive_aliases::{apply, plain_clone_debug, plain_eq};
use std::collections::BTreeMap;
use std::time::Duration;

/// HTTP configuration for a Cloudflare Temp Email worker deployment.
#[apply(plain_eq)]
pub struct ApiConfig {
    /// Base URL of the deployed worker, for example `https://mail.example.com`.
    pub base_url: String,
    /// TCP connect timeout.
    pub connect_timeout: Duration,
    /// Whole request timeout.
    pub request_timeout: Duration,
    /// Optional user agent sent by the client.
    pub user_agent: Option<String>,
    /// Headers sent on every request.
    pub default_headers: BTreeMap<String, String>,
}

impl ApiConfig {
    /// Creates a builder with conservative defaults.
    pub fn builder(base_url: impl Into<String>) -> ApiConfigBuilder {
        ApiConfigBuilder {
            base_url: base_url.into(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(20),
            user_agent: Some(default_user_agent()),
            default_headers: BTreeMap::new(),
        }
    }

    /// Validates timeout and base URL fields before I/O starts.
    pub fn validate(&self) -> TempMailResult<()> {
        if self.base_url.trim().is_empty() {
            return Err(TempMailError::InvalidConfig(
                "base_url cannot be blank".to_owned(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(TempMailError::InvalidConfig(
                "connect_timeout cannot be zero".to_owned(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(TempMailError::InvalidConfig(
                "request_timeout cannot be zero".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Builder for [`ApiConfig`].
#[apply(plain_clone_debug)]
pub struct ApiConfigBuilder {
    base_url: String,
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
    default_headers: BTreeMap<String, String>,
}

impl ApiConfigBuilder {
    /// Sets the TCP connect timeout.
    #[must_use]
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// Sets the whole request timeout.
    #[must_use]
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// Sets the user agent header.
    #[must_use]
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    /// Disables the explicit user agent header.
    #[must_use]
    pub fn clear_user_agent(mut self) -> Self {
        self.user_agent = None;
        self
    }

    /// Adds a default HTTP header.
    #[must_use]
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(name.into(), value.into());
        self
    }

    /// Builds and validates the final config.
    pub fn build(self) -> TempMailResult<ApiConfig> {
        let config = ApiConfig {
            base_url: self.base_url,
            connect_timeout: self.connect_timeout,
            request_timeout: self.request_timeout,
            user_agent: self.user_agent,
            default_headers: self.default_headers,
        };
        config.validate()?;
        Ok(config)
    }
}

fn default_user_agent() -> String {
    format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}
