use crate::util::default_user_agent;
use crate::{CreatesError, CreatesResult};
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiConfig {
    pub base_url: String,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub user_agent: Option<String>,
    pub default_headers: BTreeMap<String, String>,
}

impl ApiConfig {
    pub fn builder(base_url: impl Into<String>) -> ApiConfigBuilder {
        ApiConfigBuilder {
            base_url: base_url.into(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(20),
            user_agent: Some(default_user_agent()),
            default_headers: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> CreatesResult<()> {
        if self.base_url.trim().is_empty() {
            return Err(CreatesError::InvalidConfig(
                "base_url cannot be blank".to_owned(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(CreatesError::InvalidConfig(
                "connect_timeout cannot be zero".to_owned(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(CreatesError::InvalidConfig(
                "request_timeout cannot be zero".to_owned(),
            ));
        }
        Ok(())
    }

    pub(crate) fn into_music_config(self) -> addzero_music::ApiConfig {
        addzero_music::ApiConfig {
            base_url: self.base_url,
            connect_timeout: self.connect_timeout,
            request_timeout: self.request_timeout,
            user_agent: self.user_agent,
            default_headers: self.default_headers,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiConfigBuilder {
    base_url: String,
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
    default_headers: BTreeMap<String, String>,
}

impl ApiConfigBuilder {
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    pub fn clear_user_agent(mut self) -> Self {
        self.user_agent = None;
        self
    }

    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(name.into(), value.into());
        self
    }

    pub fn build(self) -> CreatesResult<ApiConfig> {
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
