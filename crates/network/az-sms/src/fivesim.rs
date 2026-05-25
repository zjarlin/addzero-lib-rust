use crate::{
    SmsActivationRequest, SmsError, SmsHostingRequest, SmsInbox, SmsOrder, SmsProfile, SmsProvider,
    SmsResult, error::ProviderStatus,
};
use az_derive_aliases::{apply, plain_clone_debug, plain_eq};
use reqwest::Url;
use reqwest::header::ACCEPT;
use serde::de::DeserializeOwned;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://5sim.net/v1/";

/// Configuration for the 5sim v1 API client.
#[apply(plain_eq)]
pub struct FivesimConfig {
    /// 5sim API token. It is sent as a bearer token.
    pub api_token: String,
    /// Base URL for the 5sim API, normally `https://5sim.net/v1/`.
    pub base_url: String,
    /// HTTP connection timeout.
    pub connect_timeout: Duration,
    /// HTTP request timeout.
    pub request_timeout: Duration,
    /// Optional user agent.
    pub user_agent: Option<String>,
}

impl FivesimConfig {
    /// Start building a config with default 5sim v1 settings.
    pub fn builder(api_token: impl Into<String>) -> FivesimConfigBuilder {
        FivesimConfigBuilder {
            api_token: api_token.into(),
            base_url: DEFAULT_BASE_URL.to_owned(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            user_agent: Some(default_user_agent()),
        }
    }

    /// Validate local config invariants.
    pub fn validate(&self) -> SmsResult<()> {
        if self.api_token.trim().is_empty() {
            return Err(SmsError::InvalidConfig(
                "api_token cannot be blank".to_owned(),
            ));
        }
        if self.base_url.trim().is_empty() {
            return Err(SmsError::InvalidConfig(
                "base_url cannot be blank".to_owned(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(SmsError::InvalidConfig(
                "connect_timeout cannot be zero".to_owned(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(SmsError::InvalidConfig(
                "request_timeout cannot be zero".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Builder for [`FivesimConfig`].
#[apply(plain_eq)]
pub struct FivesimConfigBuilder {
    api_token: String,
    base_url: String,
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
}

impl FivesimConfigBuilder {
    /// Override the API base URL.
    pub fn base_url(mut self, value: impl Into<String>) -> Self {
        self.base_url = value.into();
        self
    }

    /// Set the HTTP connection timeout.
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// Set the HTTP request timeout.
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// Set a custom user agent.
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    /// Remove the default user agent.
    pub fn clear_user_agent(mut self) -> Self {
        self.user_agent = None;
        self
    }

    /// Build and validate the config.
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

/// 5sim v1 API client.
#[apply(plain_clone_debug)]
pub struct FivesimClient {
    client: reqwest::Client,
    base_url: Url,
    api_token: String,
}

impl FivesimClient {
    /// Create a client with the default 5sim v1 base URL.
    pub fn from_token(api_token: impl Into<String>) -> SmsResult<Self> {
        Self::new(FivesimConfig::builder(api_token).build()?)
    }

    /// Create a client from an explicit config.
    pub fn new(config: FivesimConfig) -> SmsResult<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .map_err(|_| SmsError::InvalidBaseUrl(config.base_url.clone()))?;

        let mut builder = reqwest::Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout);
        if let Some(user_agent) = config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        Ok(Self {
            client: builder.build()?,
            base_url,
            api_token: config.api_token,
        })
    }

    /// Fetch the authenticated account profile.
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

fn provider_error(status: Option<u16>, body: String) -> SmsError {
    SmsError::ProviderError {
        status: ProviderStatus(status),
        message: body.trim().to_owned(),
    }
}

fn looks_like_provider_message(body: &str) -> bool {
    let trimmed = body.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with('{')
        && !trimmed.starts_with('[')
        && trimmed.len() <= 256
}

fn default_user_agent() -> String {
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> FivesimClient {
        FivesimClient::new(
            FivesimConfig::builder("token")
                .base_url("https://example.test/v1/")
                .build()
                .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn config_rejects_blank_token() {
        let err = FivesimConfig::builder(" ").build().unwrap_err();
        assert!(err.to_string().contains("api_token cannot be blank"));
    }

    #[test]
    fn endpoint_preserves_v1_base_path() {
        let url = client().endpoint(&["user", "check", "42"]).unwrap();
        assert_eq!(url.as_str(), "https://example.test/v1/user/check/42");
    }

    #[test]
    fn activation_url_adds_only_requested_query_options() {
        let request = SmsActivationRequest::new("usa", "any", "telegram")
            .unwrap()
            .reuse(true)
            .voice(false);
        let url = client().activation_url(&request).unwrap();

        assert_eq!(
            url.as_str(),
            "https://example.test/v1/user/buy/activation/usa/any/telegram?reuse=true&voice=false"
        );
    }

    #[test]
    fn hosting_url_matches_provider_path() {
        let request = SmsHostingRequest::new("usa", "any", "3hours").unwrap();
        let url = client().hosting_url(&request).unwrap();

        assert_eq!(
            url.as_str(),
            "https://example.test/v1/user/buy/hosting/usa/any/3hours"
        );
    }

    #[test]
    fn provider_plain_text_errors_are_detected() {
        assert!(looks_like_provider_message("no free phones"));
        assert!(!looks_like_provider_message(r#"{"id":1}"#));
    }
}
