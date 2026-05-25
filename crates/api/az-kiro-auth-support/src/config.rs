use crate::{KiroAuthSupportError, KiroAuthSupportResult};
use az_derive_aliases::{apply, plain_clone_debug, plain_eq};
use std::time::Duration;

const DEFAULT_OIDC_BASE_URL: &str = "https://oidc.us-east-1.amazonaws.com";
const DEFAULT_CLIENT_NAME: &str = "Kiro Manual Auth";
const DEFAULT_USER_AGENT: &str = "az-kiro-auth-support";
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;
const DEFAULT_POLL_INTERVAL_SECS: u64 = 2;
const DEFAULT_POLL_TIMEOUT_SECS: u64 = 300;

/// HTTP and polling configuration for Kiro's AWS Builder ID device flow.
#[apply(plain_eq)]
pub struct KiroOidcConfig {
    /// AWS IAM Identity Center OIDC API base URL.
    pub base_url: String,
    /// OIDC client name sent to `/client/register`.
    pub client_name: String,
    /// TCP connect timeout.
    pub connect_timeout: Duration,
    /// Whole request timeout.
    pub request_timeout: Duration,
    /// Poll interval used before the server returns a larger interval.
    pub poll_interval: Duration,
    /// Maximum time to wait in blocking polling helpers.
    pub poll_timeout: Duration,
    /// Optional user agent sent by the underlying HTTP client.
    pub user_agent: Option<String>,
}

impl Default for KiroOidcConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_OIDC_BASE_URL.to_owned(),
            client_name: DEFAULT_CLIENT_NAME.to_owned(),
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
            poll_interval: Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS),
            poll_timeout: Duration::from_secs(DEFAULT_POLL_TIMEOUT_SECS),
            user_agent: Some(DEFAULT_USER_AGENT.to_owned()),
        }
    }
}

impl KiroOidcConfig {
    /// Starts a builder with the default AWS OIDC endpoint.
    #[must_use]
    pub fn builder() -> KiroOidcConfigBuilder {
        KiroOidcConfigBuilder::default()
    }

    /// Returns the default OIDC API base URL.
    #[must_use]
    pub const fn default_base_url() -> &'static str {
        DEFAULT_OIDC_BASE_URL
    }

    /// Validates the configuration before constructing a network client.
    pub fn validate(&self) -> KiroAuthSupportResult<()> {
        if self.base_url.trim().is_empty() {
            return Err(KiroAuthSupportError::InvalidConfig(
                "base_url cannot be blank".to_owned(),
            ));
        }
        if self.client_name.trim().is_empty() {
            return Err(KiroAuthSupportError::InvalidConfig(
                "client_name cannot be blank".to_owned(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(KiroAuthSupportError::InvalidConfig(
                "connect_timeout must be greater than zero".to_owned(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(KiroAuthSupportError::InvalidConfig(
                "request_timeout must be greater than zero".to_owned(),
            ));
        }
        if self.poll_interval.is_zero() {
            return Err(KiroAuthSupportError::InvalidConfig(
                "poll_interval must be greater than zero".to_owned(),
            ));
        }
        if self.poll_timeout.is_zero() {
            return Err(KiroAuthSupportError::InvalidConfig(
                "poll_timeout must be greater than zero".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Builder for [`KiroOidcConfig`].
#[apply(plain_clone_debug)]
pub struct KiroOidcConfigBuilder {
    config: KiroOidcConfig,
}

impl Default for KiroOidcConfigBuilder {
    fn default() -> Self {
        Self {
            config: KiroOidcConfig::default(),
        }
    }
}

impl KiroOidcConfigBuilder {
    /// Sets the OIDC API base URL.
    #[must_use]
    pub fn base_url(mut self, value: impl Into<String>) -> Self {
        self.config.base_url = value.into();
        self
    }

    /// Sets the OIDC client name sent during registration.
    #[must_use]
    pub fn client_name(mut self, value: impl Into<String>) -> Self {
        self.config.client_name = value.into();
        self
    }

    /// Sets the TCP connect timeout.
    #[must_use]
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.config.connect_timeout = value;
        self
    }

    /// Sets the whole request timeout.
    #[must_use]
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.config.request_timeout = value;
        self
    }

    /// Sets the initial poll interval.
    #[must_use]
    pub fn poll_interval(mut self, value: Duration) -> Self {
        self.config.poll_interval = value;
        self
    }

    /// Sets the blocking poll timeout.
    #[must_use]
    pub fn poll_timeout(mut self, value: Duration) -> Self {
        self.config.poll_timeout = value;
        self
    }

    /// Sets the user agent sent by the underlying HTTP client.
    #[must_use]
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.config.user_agent = Some(value.into());
        self
    }

    /// Removes the explicit user agent.
    #[must_use]
    pub fn clear_user_agent(mut self) -> Self {
        self.config.user_agent = None;
        self
    }

    /// Completes builder validation and returns the final config.
    pub fn build(self) -> KiroAuthSupportResult<KiroOidcConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}
