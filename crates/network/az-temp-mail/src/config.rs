use anyhow::bail;
use az_derive_aliases::{apply, plain_clone_debug, plain_eq};
use std::collections::BTreeMap;
use std::time::Duration;

/// 临时邮箱 provider 的 HTTP 客户端配置。
#[apply(plain_eq)]
pub struct ApiConfig {
    /// 已部署服务的基础 URL，例如 `https://mail.example.com`。
    pub base_url: String,
    /// TCP 连接超时。
    pub connect_timeout: Duration,
    /// 单次请求总超时。
    pub request_timeout: Duration,
    /// 客户端发送的可选 User-Agent。
    pub user_agent: Option<String>,
    /// 每个请求都会携带的默认 header。
    pub default_headers: BTreeMap<String, String>,
}

impl ApiConfig {
    /// 使用保守默认值创建配置构建器。
    pub fn builder(base_url: impl Into<String>) -> ApiConfigBuilder {
        ApiConfigBuilder {
            base_url: base_url.into(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(20),
            user_agent: Some(default_user_agent()),
            default_headers: BTreeMap::new(),
        }
    }

    /// 在 IO 开始前校验超时和基础 URL 字段。
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.base_url.trim().is_empty() {
            bail!("invalid config: base_url cannot be blank");
        }
        if self.connect_timeout.is_zero() {
            bail!("invalid config: connect_timeout cannot be zero");
        }
        if self.request_timeout.is_zero() {
            bail!("invalid config: request_timeout cannot be zero");
        }
        Ok(())
    }
}

/// [`ApiConfig`] 的链式构建器。
#[apply(plain_clone_debug)]
pub struct ApiConfigBuilder {
    base_url: String,
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
    default_headers: BTreeMap<String, String>,
}

impl ApiConfigBuilder {
    /// 设置 TCP 连接超时。
    #[must_use]
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// 设置单次请求总超时。
    #[must_use]
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// 设置 User-Agent header。
    #[must_use]
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    /// 禁用显式 User-Agent header。
    #[must_use]
    pub fn clear_user_agent(mut self) -> Self {
        self.user_agent = None;
        self
    }

    /// 添加默认 HTTP header。
    #[must_use]
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(name.into(), value.into());
        self
    }

    /// 构建并校验最终配置。
    pub fn build(self) -> anyhow::Result<ApiConfig> {
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
