use crate::{CodexAuthSupportError, CodexAuthSupportResult};
use az_derive_aliases::{apply, plain_eq_redacted};
use std::time::Duration;

const DEFAULT_DUCKMAIL_BASE_URL: &str = "https://api.duckmail.sbs";
const DEFAULT_USER_AGENT: &str = "az-codex-auth-support/0.1";
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

/// Configuration for DuckMail API requests.
///
/// `auth_token` accepts either a DuckMail bearer token or a `dk_` API key.
/// Both are transmitted with `Authorization: Bearer ...`, matching DuckMail's
/// documented API behavior.
#[apply(plain_eq_redacted)]
pub struct DuckMailConfig {
    pub base_url: String,
    #[debug(skip)]
    pub auth_token: Option<String>,
    pub user_agent: Option<String>,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
}

impl Default for DuckMailConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_DUCKMAIL_BASE_URL.to_owned(),
            auth_token: None,
            user_agent: Some(DEFAULT_USER_AGENT.to_owned()),
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
        }
    }
}

impl DuckMailConfig {
    /// Starts a DuckMail configuration with the provided API base URL.
    pub fn builder(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            ..Self::default()
        }
    }

    /// Uses the default DuckMail public API base URL.
    pub fn default_base_url() -> &'static str {
        DEFAULT_DUCKMAIL_BASE_URL
    }

    /// Sets the DuckMail bearer token or `dk_` API key.
    pub fn auth_token(mut self, value: impl Into<String>) -> Self {
        self.auth_token = Some(value.into());
        self
    }

    /// Sets the user agent sent by the underlying HTTP client.
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    /// Removes the custom user agent.
    pub fn without_user_agent(mut self) -> Self {
        self.user_agent = None;
        self
    }

    /// Sets the TCP connect timeout.
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// Sets the full request timeout.
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// Validates the configuration before constructing a network client.
    pub fn validate(&self) -> CodexAuthSupportResult<()> {
        if self.base_url.trim().is_empty() {
            return Err(CodexAuthSupportError::InvalidConfig(
                "base_url cannot be blank".to_owned(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(CodexAuthSupportError::InvalidConfig(
                "connect_timeout must be greater than zero".to_owned(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(CodexAuthSupportError::InvalidConfig(
                "request_timeout must be greater than zero".to_owned(),
            ));
        }
        Ok(())
    }

    /// Completes builder validation and returns the final config.
    pub fn build(self) -> CodexAuthSupportResult<Self> {
        self.validate()?;
        Ok(self)
    }
}

/// Configuration for uploading generated auth JSON files to a CLIProxyAPI management endpoint.
#[apply(plain_eq_redacted)]
pub struct CpaUploadConfig {
    pub upload_url: String,
    #[debug(skip)]
    pub bearer_token: Option<String>,
    pub user_agent: Option<String>,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
}

#[cfg(test)]
mod tests {
    use super::{CpaUploadConfig, DuckMailConfig};

    #[test]
    fn duckmail_config_debug_skips_token() {
        let output = format!("{:?}", DuckMailConfig::default().auth_token("dk_test"));
        assert!(!output.contains("dk_test"));
        assert!(output.contains("base_url"));
    }

    #[test]
    fn cpa_upload_config_debug_skips_token() {
        let output = format!(
            "{:?}",
            CpaUploadConfig::builder("https://example.invalid").bearer_token("abc123")
        );
        assert!(!output.contains("abc123"));
        assert!(output.contains("upload_url"));
    }
}

impl CpaUploadConfig {
    /// Starts an upload configuration for a CLIProxyAPI-compatible management endpoint.
    pub fn builder(upload_url: impl Into<String>) -> Self {
        Self {
            upload_url: upload_url.into(),
            bearer_token: None,
            user_agent: Some(DEFAULT_USER_AGENT.to_owned()),
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
        }
    }

    /// Sets the management API bearer token.
    pub fn bearer_token(mut self, value: impl Into<String>) -> Self {
        self.bearer_token = Some(value.into());
        self
    }

    /// Sets the user agent sent by the underlying HTTP client.
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    /// Sets the TCP connect timeout.
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// Sets the full request timeout.
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// Validates the configuration before constructing a network client.
    pub fn validate(&self) -> CodexAuthSupportResult<()> {
        if self.upload_url.trim().is_empty() {
            return Err(CodexAuthSupportError::InvalidConfig(
                "upload_url cannot be blank".to_owned(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(CodexAuthSupportError::InvalidConfig(
                "connect_timeout must be greater than zero".to_owned(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(CodexAuthSupportError::InvalidConfig(
                "request_timeout must be greater than zero".to_owned(),
            ));
        }
        Ok(())
    }

    /// Completes builder validation and returns the final config.
    pub fn build(self) -> CodexAuthSupportResult<Self> {
        self.validate()?;
        Ok(self)
    }
}
