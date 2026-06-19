use anyhow::bail;
use std::time::Duration;

const DEFAULT_DUCKMAIL_BASE_URL: &str = "https://api.duckmail.sbs";
const DEFAULT_USER_AGENT: &str = "az-codex-auth-support/0.1";
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

/// DuckMail API 请求配置。
///
/// `auth_token` 接受 DuckMail bearer token 或 `dk_` API key。
/// 两者都会按 DuckMail 文档行为通过 `Authorization: Bearer ...` 发送。
#[derive(Clone, derive_more::Debug, Eq, PartialEq)]
pub struct DuckMailConfig {
    /// DuckMail API 基础 URL。
    pub base_url: String,
    /// DuckMail bearer token 或 `dk_` API key。
    #[debug(skip)]
    pub auth_token: Option<String>,
    /// HTTP User-Agent。
    pub user_agent: Option<String>,
    /// TCP 连接超时。
    pub connect_timeout: Duration,
    /// 完整请求超时。
    pub request_timeout: Duration,
}

impl Default for DuckMailConfig {
    fn default() -> Self {
        DuckMailConfig {
    base_url: DEFAULT_DUCKMAIL_BASE_URL.to_owned(),
    auth_token: None,
    user_agent: Some(DEFAULT_USER_AGENT.to_owned()),
    connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
    request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
}
    }
}

impl DuckMailConfig {
    /// 使用指定 API 基础 URL 创建 DuckMail 配置。
    pub fn builder(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            ..Self::default()
        }
    }

    /// 返回默认 DuckMail 公共 API 基础 URL。
    pub fn default_base_url() -> &'static str {
        DEFAULT_DUCKMAIL_BASE_URL
    }

    /// 设置 DuckMail bearer token 或 `dk_` API key。
    pub fn auth_token(mut self, value: impl Into<String>) -> Self {
        self.auth_token = Some(value.into());
        self
    }

    /// 设置底层 HTTP 客户端发送的 User-Agent。
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    /// 移除自定义 User-Agent。
    pub fn without_user_agent(mut self) -> Self {
        self.user_agent = None;
        self
    }

    /// 设置 TCP 连接超时。
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// 设置完整请求超时。
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// 构造网络客户端前校验配置。
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.base_url.trim().is_empty() {
            bail!("invalid config: base_url cannot be blank");
        }
        if self.connect_timeout.is_zero() {
            bail!("invalid config: connect_timeout must be greater than zero");
        }
        if self.request_timeout.is_zero() {
            bail!("invalid config: request_timeout must be greater than zero");
        }
        Ok(())
    }

    /// 完成构建器校验并返回最终配置。
    pub fn build(self) -> anyhow::Result<Self> {
        self.validate()?;
        Ok(self)
    }
}

/// 将生成的认证 JSON 文件上传到 CLIProxyAPI 管理端点的配置。
#[derive(Clone, derive_more::Debug, Eq, PartialEq)]
pub struct CpaUploadConfig {
    /// CLIProxyAPI 兼容上传端点 URL。
    pub upload_url: String,
    /// 管理端 bearer token。
    #[debug(skip)]
    pub bearer_token: Option<String>,
    /// HTTP User-Agent。
    pub user_agent: Option<String>,
    /// TCP 连接超时。
    pub connect_timeout: Duration,
    /// 完整请求超时。
    pub request_timeout: Duration,
}

impl CpaUploadConfig {
    /// 为 CLIProxyAPI 兼容管理端点创建上传配置。
    pub fn builder(upload_url: impl Into<String>) -> Self {
        Self {
            upload_url: upload_url.into(),
            bearer_token: None,
            user_agent: Some(DEFAULT_USER_AGENT.to_owned()),
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
        }
    }

    /// 设置管理 API bearer token。
    pub fn bearer_token(mut self, value: impl Into<String>) -> Self {
        self.bearer_token = Some(value.into());
        self
    }

    /// 设置底层 HTTP 客户端发送的 User-Agent。
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    /// 设置 TCP 连接超时。
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// 设置完整请求超时。
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// 构造网络客户端前校验配置。
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.upload_url.trim().is_empty() {
            bail!("invalid config: upload_url cannot be blank");
        }
        if self.connect_timeout.is_zero() {
            bail!("invalid config: connect_timeout must be greater than zero");
        }
        if self.request_timeout.is_zero() {
            bail!("invalid config: request_timeout must be greater than zero");
        }
        Ok(())
    }

    /// 完成构建器校验并返回最终配置。
    pub fn build(self) -> anyhow::Result<Self> {
        self.validate()?;
        Ok(self)
    }
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
