use crate::input::{required_non_blank, trim_non_blank};
use crate::client::CloudflareTempMailApi;
use crate::config::ApiConfig;
use crate::model::{CreateMailboxRequest, NewAddressRequest};

/// 调用方提供的 Cloudflare 临时邮箱 worker 上下文，用于初始化客户端和默认邮箱创建请求。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudflareTempMailContext {
    /// 已部署 worker 的基础 URL，例如 `https://mail.example.com`。
    pub base_url: String,
    /// 可选的部署专属认证 header 值，会作为 `x-custom-auth` 发送。
    pub custom_auth: Option<String>,
    /// 可选的首选邮箱本地部分。
    pub address_name: Option<String>,
    /// 可选的首选邮箱域名。
    pub address_domain: Option<String>,
    /// 受保护部署所需的可选 Cloudflare Turnstile token。
    pub cf_token: Option<String>,
    /// 支持随机子域名部署的可选开关。
    pub enable_random_subdomain: Option<bool>,
}

impl CloudflareTempMailContext {
    /// 为 Cloudflare worker 客户端构建已校验的 HTTP 配置。
    pub fn api_config(&self) -> anyhow::Result<ApiConfig> {
        ApiConfig::try_from(self)
    }

    /// 根据当前上下文创建 Cloudflare worker 客户端。
    pub fn create_api(&self) -> anyhow::Result<CloudflareTempMailApi> {
        CloudflareTempMailApi::new(self.api_config()?)
    }

    /// 根据上下文默认值构建邮箱创建请求。
    #[must_use]
    pub fn create_mailbox_request(&self) -> CreateMailboxRequest {
        CreateMailboxRequest::from(self)
    }

    /// 根据上下文默认值构建原始 `/api/new_address` 请求。
    #[must_use]
    pub fn new_address_request(&self) -> NewAddressRequest {
        NewAddressRequest::from(self)
    }
}

impl TryFrom<&CloudflareTempMailContext> for ApiConfig {
    type Error = anyhow::Error;

    fn try_from(value: &CloudflareTempMailContext) -> Result<Self, Self::Error> {
        let mut builder = ApiConfig::builder(required_non_blank(&value.base_url, "base_url")?);
        if let Some(custom_auth) = trim_non_blank(value.custom_auth.as_deref()) {
            builder = builder.default_header("x-custom-auth", custom_auth);
        }
        builder.build()
    }
}

impl From<&CloudflareTempMailContext> for CreateMailboxRequest {
    fn from(value: &CloudflareTempMailContext) -> Self {
        let mut request = CreateMailboxRequest::random();
        request.name = trim_non_blank(value.address_name.as_deref()).map(str::to_owned);
        request.domain = trim_non_blank(value.address_domain.as_deref()).map(str::to_owned);
        request.cf_token = trim_non_blank(value.cf_token.as_deref()).map(str::to_owned);
        request.enable_random_subdomain = value.enable_random_subdomain.unwrap_or(false);
        request
    }
}

impl From<&CloudflareTempMailContext> for NewAddressRequest {
    fn from(value: &CloudflareTempMailContext) -> Self {
        Self {
            name: trim_non_blank(value.address_name.as_deref()).map(str::to_owned),
            domain: trim_non_blank(value.address_domain.as_deref()).map(str::to_owned),
            cf_token: trim_non_blank(value.cf_token.as_deref()).map(str::to_owned),
            enable_random_subdomain: value.enable_random_subdomain,
        }
    }
}
