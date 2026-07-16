//! HTTP configuration for the Tianyancha client.

use std::time::Duration;

use anyhow::bail;

/// Tianyancha HTTP client configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TycConfig {
    /// TCP/HTTPS connect timeout.
    pub connect_timeout: Duration,
    /// Total timeout for each request.
    pub request_timeout: Duration,
}

impl Default for TycConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(20),
        }
    }
}

impl TycConfig {
    /// Creates a builder initialized with conservative defaults.
    pub fn builder() -> TycConfigBuilder {
        TycConfigBuilder::default()
    }

    /// Validates local timeout constraints before the HTTP client is built.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.connect_timeout.is_zero() {
            bail!("invalid Tianyancha config: connect_timeout cannot be zero");
        }
        if self.request_timeout.is_zero() {
            bail!("invalid Tianyancha config: request_timeout cannot be zero");
        }
        Ok(())
    }
}

/// Builder for [`TycConfig`].
#[derive(Clone, Debug)]
pub struct TycConfigBuilder {
    connect_timeout: Duration,
    request_timeout: Duration,
}

impl Default for TycConfigBuilder {
    fn default() -> Self {
        let config = TycConfig::default();
        Self {
            connect_timeout: config.connect_timeout,
            request_timeout: config.request_timeout,
        }
    }
}

impl TycConfigBuilder {
    /// Sets the TCP/HTTPS connect timeout.
    #[must_use]
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// Sets the total timeout for each request.
    #[must_use]
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// Builds and validates the configuration.
    pub fn build(self) -> anyhow::Result<TycConfig> {
        let config = TycConfig {
            connect_timeout: self.connect_timeout,
            request_timeout: self.request_timeout,
        };
        config.validate()?;
        Ok(config)
    }
}
