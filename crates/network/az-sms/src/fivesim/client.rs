use crate::error::{SmsError, SmsResult};
use crate::http::{
    build_client, default_user_agent, ensure_non_blank, ensure_non_zero_duration,
    looks_like_provider_message, provider_error,
};
use crate::model::{
    SmsActivationRequest, SmsHostingRequest, SmsInbox, SmsOrder, SmsProfile,
};
use crate::provider::SmsProvider;
use az_derive_aliases::{apply, plain_clone_debug, plain_eq};
use reqwest::Url;
use reqwest::header::ACCEPT;
use serde::de::DeserializeOwned;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://5sim.net/v1/";

/// 5sim v1 API 客户端配置。
#[apply(plain_eq)]
pub struct FivesimConfig {
    /// 5sim API token，会作为 bearer token 发送。
    pub api_token: String,
    /// 5sim API 基础 URL，通常是 `https://5sim.net/v1/`。
    pub base_url: String,
    /// HTTP 连接超时。
    pub connect_timeout: Duration,
    /// HTTP 请求超时。
    pub request_timeout: Duration,
    /// 可选 User-Agent。
    pub user_agent: Option<String>,
}

impl FivesimConfig {
    /// 使用默认 5sim v1 设置开始构建配置。
    pub fn builder(api_token: impl Into<String>) -> FivesimConfigBuilder {
        FivesimConfigBuilder {
            api_token: api_token.into(),
            base_url: DEFAULT_BASE_URL.to_owned(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            user_agent: Some(default_user_agent()),
        }
    }

    /// 校验本地配置不变量。
    pub fn validate(&self) -> SmsResult<()> {
        ensure_non_blank("api_token", &self.api_token)?;
        ensure_non_blank("base_url", &self.base_url)?;
        ensure_non_zero_duration("connect_timeout", self.connect_timeout)?;
        ensure_non_zero_duration("request_timeout", self.request_timeout)?;
        Ok(())
    }
}

/// [`FivesimConfig`] 的链式构建器。
#[apply(plain_eq)]
pub struct FivesimConfigBuilder {
    api_token: String,
    base_url: String,
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
}

impl FivesimConfigBuilder {
    /// 覆盖 API 基础 URL。
    pub fn base_url(mut self, value: impl Into<String>) -> Self {
        self.base_url = value.into();
        self
    }

    /// 设置 HTTP 连接超时。
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// 设置 HTTP 请求超时。
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// 设置自定义 User-Agent。
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    /// 移除默认 User-Agent。
    pub fn clear_user_agent(mut self) -> Self {
        self.user_agent = None;
        self
    }

    /// 构建并校验配置。
    pub fn build(self) -> SmsResult<FivesimConfig> {
        let config = FivesimConfig {
            api_token: self.api_token,
            base_url: self.base_url,
            connect_timeout: self.connect_timeout,
            request_timeout: self.request_timeout,
            user_agent: self.user_agent,
        };
        config.validate()?;
        Ok(config)
    }
}

/// 5sim v1 API 客户端。
#[apply(plain_clone_debug)]
pub struct FivesimClient {
    client: reqwest::Client,
    base_url: Url,
    api_token: String,
}

impl FivesimClient {
    /// 使用默认 5sim v1 基础 URL 创建客户端。
    pub fn from_token(api_token: impl Into<String>) -> SmsResult<Self> {
        Self::new(FivesimConfig::builder(api_token).build()?)
    }

    /// 使用显式配置创建客户端。
    pub fn new(config: FivesimConfig) -> SmsResult<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .map_err(|_| SmsError::InvalidBaseUrl(config.base_url.clone()))?;

        Ok(Self {
            client: build_client(
                config.connect_timeout,
                config.request_timeout,
                config.user_agent,
            )?,
            base_url,
            api_token: config.api_token,
        })
    }

    /// 获取已认证账号的 profile。
    pub async fn profile(&self) -> SmsResult<SmsProfile> {
        let url = self.endpoint(&["user", "profile"])?;
        self.get_json(url).await
    }

    fn endpoint(&self, segments: &[&str]) -> SmsResult<Url> {
        let mut url = self.base_url.clone();
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| SmsError::InvalidEndpoint(self.base_url.to_string()))?;
            path.pop_if_empty();
            for segment in segments {
                if segment.trim().is_empty() {
                    return Err(SmsError::InvalidEndpoint(
                        "path segment cannot be blank".to_owned(),
                    ));
                }
                path.push(segment.trim_matches('/'));
            }
        }
        Ok(url)
    }

    async fn get_json<T: DeserializeOwned>(&self, url: Url) -> SmsResult<T> {
        let response = self
            .client
            .get(url)
            .bearer_auth(self.api_token.trim())
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(provider_error(Some(status.as_u16()), body));
        }

        match serde_json::from_str::<T>(&body) {
            Ok(value) => Ok(value),
            Err(_error) if looks_like_provider_message(&body) => {
                Err(provider_error(Some(status.as_u16()), body))
            }
            Err(error) => Err(SmsError::Json(error)),
        }
    }

    fn activation_url(&self, request: &SmsActivationRequest) -> SmsResult<Url> {
        request.validate()?;
        let mut url = self.endpoint(&[
            "user",
            "buy",
            "activation",
            request.country.trim(),
            request.operator.trim(),
            request.product.trim(),
        ])?;
        {
            let mut query = url.query_pairs_mut();
            if let Some(value) = request.forwarding {
                query.append_pair("forwarding", bool_query_value(value));
            }
            if let Some(value) = request
                .number
                .as_deref()
                .map(str::trim)
                .filter(|item| !item.is_empty())
            {
                query.append_pair("number", value);
            }
            if let Some(value) = request.reuse {
                query.append_pair("reuse", bool_query_value(value));
            }
            if let Some(value) = request.voice {
                query.append_pair("voice", bool_query_value(value));
            }
            if let Some(value) = request
                .ref_code
                .as_deref()
                .map(str::trim)
                .filter(|item| !item.is_empty())
            {
                query.append_pair("ref", value);
            }
        }
        Ok(url)
    }

    fn hosting_url(&self, request: &SmsHostingRequest) -> SmsResult<Url> {
        request.validate()?;
        self.endpoint(&[
            "user",
            "buy",
            "hosting",
            request.country.trim(),
            request.operator.trim(),
            request.product.trim(),
        ])
    }

    fn order_url(&self, action: &'static str, order_id: u64) -> SmsResult<Url> {
        let id = order_id.to_string();
        self.endpoint(&["user", action, id.as_str()])
    }
}

#[async_trait::async_trait]
impl SmsProvider for FivesimClient {
    async fn buy_activation_number(&self, request: SmsActivationRequest) -> SmsResult<SmsOrder> {
        let url = self.activation_url(&request)?;
        self.get_json(url).await
    }

    async fn buy_hosting_number(&self, request: SmsHostingRequest) -> SmsResult<SmsOrder> {
        let url = self.hosting_url(&request)?;
        self.get_json(url).await
    }

    async fn check_order(&self, order_id: u64) -> SmsResult<SmsOrder> {
        self.get_json(self.order_url("check", order_id)?).await
    }

    async fn finish_order(&self, order_id: u64) -> SmsResult<SmsOrder> {
        self.get_json(self.order_url("finish", order_id)?).await
    }

    async fn cancel_order(&self, order_id: u64) -> SmsResult<SmsOrder> {
        self.get_json(self.order_url("cancel", order_id)?).await
    }

    async fn ban_order(&self, order_id: u64) -> SmsResult<SmsOrder> {
        self.get_json(self.order_url("ban", order_id)?).await
    }

    async fn inbox(&self, order_id: u64) -> SmsResult<SmsInbox> {
        let id = order_id.to_string();
        self.get_json(self.endpoint(&["user", "sms", "inbox", id.as_str()])?)
            .await
    }
}

fn bool_query_value(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
