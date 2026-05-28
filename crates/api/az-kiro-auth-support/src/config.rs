use crate::{KiroAuthSupportError, KiroAuthSupportResult};
use az_derive_aliases::{apply, impl_default, plain_default_clone_debug, plain_eq};
use std::time::Duration;

const DEFAULT_OIDC_BASE_URL: &str = "https://oidc.us-east-1.amazonaws.com";
const DEFAULT_CLIENT_NAME: &str = "Kiro Manual Auth";
const DEFAULT_USER_AGENT: &str = "az-kiro-auth-support";
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;
const DEFAULT_POLL_INTERVAL_SECS: u64 = 2;
const DEFAULT_POLL_TIMEOUT_SECS: u64 = 300;

/// Kiro AWS Builder ID 设备流程的 HTTP 与轮询配置。
#[apply(plain_eq)]
pub struct KiroOidcConfig {
    /// AWS IAM Identity Center OIDC API 基础 URL。
    pub base_url: String,
    /// 发送给 `/client/register` 的 OIDC 客户端名称。
    pub client_name: String,
    /// TCP 连接超时。
    pub connect_timeout: Duration,
    /// 单次请求总超时。
    pub request_timeout: Duration,
    /// 服务端返回更大间隔前使用的轮询间隔。
    pub poll_interval: Duration,
    /// 阻塞式轮询辅助函数的最大等待时间。
    pub poll_timeout: Duration,
    /// 底层 HTTP 客户端发送的可选 User-Agent。
    pub user_agent: Option<String>,
}

impl_default!(KiroOidcConfig => KiroOidcConfig {
    base_url: DEFAULT_OIDC_BASE_URL.to_owned(),
    client_name: DEFAULT_CLIENT_NAME.to_owned(),
    connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
    request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
    poll_interval: Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS),
    poll_timeout: Duration::from_secs(DEFAULT_POLL_TIMEOUT_SECS),
    user_agent: Some(DEFAULT_USER_AGENT.to_owned()),
});

impl KiroOidcConfig {
    /// 使用默认 AWS OIDC 端点创建构建器。
    #[must_use]
    pub fn builder() -> KiroOidcConfigBuilder {
        KiroOidcConfigBuilder::default()
    }

    /// 返回默认 OIDC API 基础 URL。
    #[must_use]
    pub const fn default_base_url() -> &'static str {
        DEFAULT_OIDC_BASE_URL
    }

    /// 构造网络客户端前校验配置。
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

/// [`KiroOidcConfig`] 的链式构建器。
#[apply(plain_default_clone_debug)]
pub struct KiroOidcConfigBuilder {
    config: KiroOidcConfig,
}

impl KiroOidcConfigBuilder {
    /// 设置 OIDC API 基础 URL。
    #[must_use]
    pub fn base_url(mut self, value: impl Into<String>) -> Self {
        self.config.base_url = value.into();
        self
    }

    /// 设置注册时发送的 OIDC 客户端名称。
    #[must_use]
    pub fn client_name(mut self, value: impl Into<String>) -> Self {
        self.config.client_name = value.into();
        self
    }

    /// 设置 TCP 连接超时。
    #[must_use]
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.config.connect_timeout = value;
        self
    }

    /// 设置单次请求总超时。
    #[must_use]
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.config.request_timeout = value;
        self
    }

    /// 设置初始轮询间隔。
    #[must_use]
    pub fn poll_interval(mut self, value: Duration) -> Self {
        self.config.poll_interval = value;
        self
    }

    /// 设置阻塞式轮询超时。
    #[must_use]
    pub fn poll_timeout(mut self, value: Duration) -> Self {
        self.config.poll_timeout = value;
        self
    }

    /// 设置底层 HTTP 客户端发送的 User-Agent。
    #[must_use]
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.config.user_agent = Some(value.into());
        self
    }

    /// 移除显式 User-Agent。
    #[must_use]
    pub fn clear_user_agent(mut self) -> Self {
        self.config.user_agent = None;
        self
    }

    /// 完成构建器校验并返回最终配置。
    pub fn build(self) -> KiroAuthSupportResult<KiroOidcConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}
