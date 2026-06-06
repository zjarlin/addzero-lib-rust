use crate::error::{SmsError, SmsResult};
use crate::http::{
    build_client, default_user_agent, ensure_non_blank, ensure_non_zero_duration,
    looks_like_provider_message, provider_error,
};
use crate::model::{
    SmsActivationRequest, SmsHostingRequest, SmsInbox, SmsMessage, SmsOrder, SmsOrderStatus,
    SmsProfile,
};
use crate::provider::SmsProvider;
use az_derive_aliases::{apply, deserialize_debug, plain_clone_debug, plain_eq};
use reqwest::Url;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Deserializer};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.grizzlysms.com/stubs/handler_api.php";
const PROVIDER_NAME: &str = "GrizzlySMS";

/// Grizzly SMS 兼容 sms-activate 的 API 客户端配置。
#[apply(plain_eq)]
pub struct GrizzlySmsConfig {
    /// Grizzly SMS API key，会作为 `api_key` 查询参数发送。
    pub api_key: String,
    /// API handler URL，通常是 `https://api.grizzlysms.com/stubs/handler_api.php`。
    pub base_url: String,
    /// HTTP 连接超时。
    pub connect_timeout: Duration,
    /// HTTP 请求超时。
    pub request_timeout: Duration,
    /// 可选 User-Agent。
    pub user_agent: Option<String>,
}

impl GrizzlySmsConfig {
    /// 使用默认 Grizzly SMS API 设置开始构建配置。
    pub fn builder(api_key: impl Into<String>) -> GrizzlySmsConfigBuilder {
        GrizzlySmsConfigBuilder {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_owned(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            user_agent: Some(default_user_agent()),
        }
    }

    /// 校验本地配置不变量。
    pub fn validate(&self) -> SmsResult<()> {
        ensure_non_blank("api_key", &self.api_key)?;
        ensure_non_blank("base_url", &self.base_url)?;
        ensure_non_zero_duration("connect_timeout", self.connect_timeout)?;
        ensure_non_zero_duration("request_timeout", self.request_timeout)?;
        Ok(())
    }
}

/// [`GrizzlySmsConfig`] 的链式构建器。
#[apply(plain_eq)]
pub struct GrizzlySmsConfigBuilder {
    api_key: String,
    base_url: String,
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
}

impl GrizzlySmsConfigBuilder {
    /// 覆盖 API handler URL。
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
    pub fn build(self) -> SmsResult<GrizzlySmsConfig> {
        let config = GrizzlySmsConfig {
            api_key: self.api_key,
            base_url: self.base_url,
            connect_timeout: self.connect_timeout,
            request_timeout: self.request_timeout,
            user_agent: self.user_agent,
        };
        config.validate()?;
        Ok(config)
    }
}

/// Grizzly SMS API 客户端。
#[apply(plain_clone_debug)]
pub struct GrizzlySmsClient {
    client: reqwest::Client,
    base_url: Url,
    api_key: String,
}

impl GrizzlySmsClient {
    /// 使用默认 Grizzly SMS API handler URL 创建客户端。
    pub fn from_api_key(api_key: impl Into<String>) -> SmsResult<Self> {
        Self::new(GrizzlySmsConfig::builder(api_key).build()?)
    }

    /// 使用显式配置创建客户端。
    pub fn new(config: GrizzlySmsConfig) -> SmsResult<Self> {
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
            api_key: config.api_key,
        })
    }

    /// 获取当前账号余额。
    pub async fn balance(&self) -> SmsResult<f64> {
        let body = self.send_text(self.action_url("getBalance")?).await?;
        parse_balance_response(&body)
    }

    /// 获取最小账号 profile。
    ///
    /// Grizzly SMS 通过公开 API 暴露余额，但不暴露账号 ID 或邮箱。
    pub async fn profile(&self) -> SmsResult<SmsProfile> {
        Ok(SmsProfile {
            id: 0,
            email: String::new(),
            balance: self.balance().await?,
            rating: 0.0,
            frozen_balance: None,
        })
    }

    fn action_url(&self, action: &str) -> SmsResult<Url> {
        let action = action.trim();
        if action.is_empty() {
            return Err(SmsError::InvalidEndpoint(
                "action cannot be blank".to_owned(),
            ));
        }

        let mut url = self.base_url.clone();
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("api_key", self.api_key.trim());
            query.append_pair("action", action);
        }
        Ok(url)
    }

    fn activation_url(&self, request: &SmsActivationRequest) -> SmsResult<Url> {
        request.validate()?;
        validate_activation_support(request)?;

        let mut url = self.action_url("getNumberV2")?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("service", request.product.trim());
            query.append_pair("country", request.country.trim());
        }
        Ok(url)
    }

    fn order_status_url(&self, order_id: u64) -> SmsResult<Url> {
        let mut url = self.action_url("getStatus")?;
        url.query_pairs_mut()
            .append_pair("id", order_id.to_string().as_str());
        Ok(url)
    }

    fn order_status_v2_url(&self, order_id: u64) -> SmsResult<Url> {
        let mut url = self.action_url("getStatusV2")?;
        url.query_pairs_mut()
            .append_pair("id", order_id.to_string().as_str());
        Ok(url)
    }

    fn set_status_url(&self, order_id: u64, status: i64) -> SmsResult<Url> {
        let mut url = self.action_url("setStatus")?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("status", status.to_string().as_str());
            query.append_pair("id", order_id.to_string().as_str());
        }
        Ok(url)
    }

    fn active_activations_url(&self) -> SmsResult<Url> {
        self.action_url("getActiveActivations")
    }

    async fn send_text(&self, url: Url) -> SmsResult<String> {
        let response = self
            .client
            .get(url)
            .header(ACCEPT, "application/json, text/plain;q=0.9")
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(provider_error(Some(status.as_u16()), body));
        }

        Ok(body.trim().to_owned())
    }

    async fn active_order(&self, order_id: u64) -> SmsResult<Option<SmsOrder>> {
        let body = self.send_text(self.active_activations_url()?).await?;
        if looks_like_provider_message(&body) {
            return Err(provider_error(None, body));
        }

        let activations = serde_json::from_str::<Vec<GrizzlyActiveActivation>>(&body)?;
        Ok(activations
            .into_iter()
            .find(|activation| activation.activation_id == order_id)
            .map(GrizzlyActiveActivation::into_order))
    }

    async fn status_v2_message(&self, order_id: u64) -> SmsResult<Option<SmsMessage>> {
        let body = self.send_text(self.order_status_v2_url(order_id)?).await?;
        if looks_like_provider_message(&body) {
            return Err(provider_error(None, body));
        }

        let response = serde_json::from_str::<GrizzlyStatusV2Response>(&body)?;
        Ok(response.sms.and_then(GrizzlyStatusSms::into_message))
    }

    async fn set_activation_status(
        &self,
        order_id: u64,
        provider_status: i64,
        order_status: SmsOrderStatus,
    ) -> SmsResult<SmsOrder> {
        let body = self
            .send_text(self.set_status_url(order_id, provider_status)?)
            .await?;
        parse_set_status_response(&body)?;
        Ok(synthetic_order(order_id, order_status))
    }
}

#[async_trait::async_trait]
impl SmsProvider for GrizzlySmsClient {
    async fn buy_activation_number(&self, request: SmsActivationRequest) -> SmsResult<SmsOrder> {
        let body = self.send_text(self.activation_url(&request)?).await?;
        parse_number_response(&body).map(|response| response.into_order(&request))
    }

    async fn buy_hosting_number(&self, _request: SmsHostingRequest) -> SmsResult<SmsOrder> {
        Err(unsupported("hosted/rented numbers"))
    }

    async fn check_order(&self, order_id: u64) -> SmsResult<SmsOrder> {
        let body = self.send_text(self.order_status_url(order_id)?).await?;
        let snapshot = parse_status_response(&body)?;
        let mut order = self
            .active_order(order_id)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| synthetic_order(order_id, snapshot.status.clone()));

        order.status = snapshot.status.clone();
        if matches!(snapshot.status, SmsOrderStatus::Received)
            && let Some(message) = self
                .status_v2_message(order_id)
                .await
                .ok()
                .flatten()
                .or_else(|| snapshot.message())
        {
            order.sms = vec![message];
        }

        Ok(order)
    }

    async fn finish_order(&self, order_id: u64) -> SmsResult<SmsOrder> {
        self.set_activation_status(order_id, 6, SmsOrderStatus::Finished)
            .await
    }

    async fn cancel_order(&self, order_id: u64) -> SmsResult<SmsOrder> {
        self.set_activation_status(order_id, 8, SmsOrderStatus::Canceled)
            .await
    }

    async fn ban_order(&self, order_id: u64) -> SmsResult<SmsOrder> {
        self.set_activation_status(order_id, 8, SmsOrderStatus::Banned)
            .await
    }

    async fn inbox(&self, _order_id: u64) -> SmsResult<SmsInbox> {
        Err(unsupported("hosted/rented inbox"))
    }
}

fn validate_activation_support(request: &SmsActivationRequest) -> SmsResult<()> {
    if !request.operator.trim().eq_ignore_ascii_case("any") {
        return Err(unsupported("operator-specific activation requests"));
    }

    if request.forwarding.is_some() {
        return Err(unsupported("activation forwarding"));
    }
    if request.number.is_some() {
        return Err(unsupported("number reuse by explicit phone number"));
    }
    if request.reuse.is_some() {
        return Err(unsupported("number reuse flags"));
    }
    if request.voice.is_some() {
        return Err(unsupported("voice verification flags"));
    }
    if request.ref_code.is_some() {
        return Err(unsupported("referral code query parameters"));
    }

    Ok(())
}

fn parse_number_response(body: &str) -> SmsResult<GrizzlyNumberV2Response> {
    match serde_json::from_str::<GrizzlyNumberV2Response>(body) {
        Ok(response) => Ok(response),
        Err(error) => {
            if let Some((activation_id, phone_number)) = parse_access_number(body) {
                return Ok(GrizzlyNumberV2Response {
                    activation_id,
                    phone_number,
                    activation_cost: 0.0,
                    activation_time: None,
                    activation_end: None,
                    country_code: None,
                });
            }
            if looks_like_provider_message(body) {
                Err(provider_error(None, body))
            } else {
                Err(SmsError::Json(error))
            }
        }
    }
}

fn parse_access_number(body: &str) -> Option<(u64, String)> {
    let mut parts = body.trim().splitn(3, ':');
    if parts.next()? != "ACCESS_NUMBER" {
        return None;
    }
    let id = parts.next()?.parse().ok()?;
    let phone = parts.next()?.trim().to_owned();
    if phone.is_empty() {
        return None;
    }
    Some((id, phone))
}

fn parse_balance_response(body: &str) -> SmsResult<f64> {
    let Some(balance) = body.trim().strip_prefix("ACCESS_BALANCE:") else {
        return Err(provider_error(None, body));
    };
    balance
        .trim()
        .parse()
        .map_err(|_| provider_error(None, format!("invalid balance response: {body}")))
}

fn parse_set_status_response(body: &str) -> SmsResult<()> {
    match body.trim() {
        "ACCESS_READY" | "ACCESS_RETRY_GET" | "ACCESS_ACTIVATION" | "ACCESS_CANCEL" => Ok(()),
        _ => Err(provider_error(None, body)),
    }
}

fn parse_status_response(body: &str) -> SmsResult<GrizzlyStatusSnapshot> {
    let body = body.trim();
    if body == "STATUS_WAIT_CODE" || body == "STATUS_WAIT_RESEND" {
        return Ok(GrizzlyStatusSnapshot {
            status: SmsOrderStatus::Pending,
            code: None,
        });
    }
    if body.starts_with("STATUS_WAIT_RETRY") {
        return Ok(GrizzlyStatusSnapshot {
            status: SmsOrderStatus::Pending,
            code: None,
        });
    }
    if body == "STATUS_CANCEL" {
        return Ok(GrizzlyStatusSnapshot {
            status: SmsOrderStatus::Canceled,
            code: None,
        });
    }
    if let Some(code) = body
        .strip_prefix("STATUS_OK:")
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(GrizzlyStatusSnapshot {
            status: SmsOrderStatus::Received,
            code: Some(code.to_owned()),
        });
    }

    Err(provider_error(None, body))
}

#[apply(plain_eq)]
struct GrizzlyStatusSnapshot {
    status: SmsOrderStatus,
    code: Option<String>,
}

impl GrizzlyStatusSnapshot {
    fn message(&self) -> Option<SmsMessage> {
        self.code.as_deref().map(|code| SmsMessage {
            id: None,
            created_at: None,
            date: None,
            sender: String::new(),
            text: code.to_owned(),
            code: Some(code.to_owned()),
        })
    }
}

#[apply(deserialize_debug)]
struct GrizzlyNumberV2Response {
    #[serde(rename = "activationId", deserialize_with = "deserialize_u64ish")]
    activation_id: u64,
    #[serde(rename = "phoneNumber")]
    phone_number: String,
    #[serde(
        rename = "activationCost",
        default,
        deserialize_with = "deserialize_f64ish_default"
    )]
    activation_cost: f64,
    #[serde(rename = "activationTime", default)]
    activation_time: Option<String>,
    #[serde(rename = "activationEnd", default)]
    activation_end: Option<String>,
    #[serde(rename = "countryCode", default)]
    country_code: Option<String>,
}

impl GrizzlyNumberV2Response {
    fn into_order(self, request: &SmsActivationRequest) -> SmsOrder {
        SmsOrder {
            id: self.activation_id,
            phone: self.phone_number,
            operator: None,
            product: request.product.trim().to_owned(),
            price: self.activation_cost,
            status: SmsOrderStatus::Pending,
            expires: self.activation_end,
            sms: Vec::new(),
            created_at: self.activation_time,
            forwarding: None,
            forwarding_number: None,
            country: non_empty(self.country_code),
        }
    }
}

#[apply(deserialize_debug)]
struct GrizzlyActiveActivation {
    #[serde(rename = "activationId", deserialize_with = "deserialize_u64ish")]
    activation_id: u64,
    #[serde(
        rename = "activationCost",
        default,
        deserialize_with = "deserialize_f64ish_default"
    )]
    activation_cost: f64,
    #[serde(rename = "activationStatus", default)]
    activation_status: i64,
    #[serde(rename = "activationTime", default)]
    activation_time: Option<String>,
    #[serde(rename = "countryCode", default)]
    country_code: Option<String>,
    #[serde(rename = "countryName", default)]
    country_name: Option<String>,
    #[serde(rename = "phoneNumber", default)]
    phone_number: String,
    #[serde(rename = "serviceCode", default)]
    service_code: String,
    #[serde(rename = "smsCode", default)]
    sms_code: String,
    #[serde(rename = "smsText", default)]
    sms_text: String,
}

impl GrizzlyActiveActivation {
    fn into_order(self) -> SmsOrder {
        let sms = self.sms_message().into_iter().collect::<Vec<_>>();
        let status = if sms.is_empty() {
            map_activation_status(self.activation_status)
        } else {
            SmsOrderStatus::Received
        };

        SmsOrder {
            id: self.activation_id,
            phone: self.phone_number,
            operator: None,
            product: self.service_code,
            price: self.activation_cost,
            status,
            expires: None,
            sms,
            created_at: self.activation_time,
            forwarding: None,
            forwarding_number: None,
            country: non_empty(self.country_name).or_else(|| non_empty(self.country_code)),
        }
    }

    fn sms_message(&self) -> Option<SmsMessage> {
        let code = non_empty(Some(self.sms_code.clone()));
        let text = non_empty(Some(self.sms_text.clone()))
            .or_else(|| code.clone())
            .unwrap_or_default();
        if code.is_none() && text.is_empty() {
            return None;
        }

        Some(SmsMessage {
            id: None,
            created_at: self.activation_time.clone(),
            date: self.activation_time.clone(),
            sender: String::new(),
            text,
            code,
        })
    }
}

#[apply(deserialize_debug)]
struct GrizzlyStatusV2Response {
    #[serde(default)]
    sms: Option<GrizzlyStatusSms>,
}

#[apply(deserialize_debug)]
struct GrizzlyStatusSms {
    #[serde(rename = "dateTime", default)]
    date_time: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    text: String,
}

impl GrizzlyStatusSms {
    fn into_message(self) -> Option<SmsMessage> {
        let code = non_empty(self.code);
        let text = non_empty(Some(self.text))
            .or_else(|| code.clone())
            .unwrap_or_default();
        if code.is_none() && text.is_empty() {
            return None;
        }

        Some(SmsMessage {
            id: None,
            created_at: self.date_time.clone(),
            date: self.date_time,
            sender: String::new(),
            text,
            code,
        })
    }
}

#[apply(deserialize_debug)]
#[serde(untagged)]
enum StringOrNumber {
    String(String),
    Number(serde_json::Number),
}

fn deserialize_u64ish<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = StringOrNumber::deserialize(deserializer)?;
    match value {
        StringOrNumber::String(value) => value.trim().parse().map_err(serde::de::Error::custom),
        StringOrNumber::Number(value) => value
            .as_u64()
            .ok_or_else(|| serde::de::Error::custom("number is not a u64")),
    }
}

fn deserialize_f64ish_default<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<StringOrNumber>::deserialize(deserializer)?;
    match value {
        Some(StringOrNumber::String(value)) => {
            value.trim().parse().map_err(serde::de::Error::custom)
        }
        Some(StringOrNumber::Number(value)) => value
            .as_f64()
            .ok_or_else(|| serde::de::Error::custom("number is not an f64")),
        None => Ok(0.0),
    }
}

fn map_activation_status(value: i64) -> SmsOrderStatus {
    match value {
        1 | 3 => SmsOrderStatus::Pending,
        6 => SmsOrderStatus::Finished,
        8 | -1 => SmsOrderStatus::Canceled,
        _ => SmsOrderStatus::Unknown,
    }
}

fn synthetic_order(order_id: u64, status: SmsOrderStatus) -> SmsOrder {
    SmsOrder {
        id: order_id,
        phone: String::new(),
        operator: None,
        product: String::new(),
        price: 0.0,
        status,
        expires: None,
        sms: Vec::new(),
        created_at: None,
        forwarding: None,
        forwarding_number: None,
        country: None,
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_owned())
        .filter(|item| !item.is_empty())
}

fn unsupported(operation: &'static str) -> SmsError {
    SmsError::UnsupportedOperation {
        provider: PROVIDER_NAME,
        operation,
    }
}
