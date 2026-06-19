use crate::http::{
    build_client, default_user_agent, ensure_non_blank, ensure_non_zero_duration,
    looks_like_provider_message, provider_error,
};
use crate::model::{SmsActivationRequest, SmsMessage, SmsOrder, SmsOrderStatus};
use crate::model::{SmsHostingRequest, SmsInbox};
use crate::provider::SmsProvider;
use anyhow::{Context, anyhow, bail};
use reqwest::Url;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use std::time::Duration;
use uuid::Uuid;

const DEFAULT_BASE_URL: &str = "https://www.dogesms.com/api/control/";
const PROVIDER_NAME: &str = "DogeSMS";

/// DogeSMS Control API client configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DogSmsConfig {
    /// DogeSMS static API key, sent with the `X-API-Key` header.
    pub api_key: String,
    /// Control API base URL, usually `https://www.dogesms.com/api/control/`.
    pub base_url: String,
    /// HTTP connect timeout.
    pub connect_timeout: Duration,
    /// HTTP request timeout.
    pub request_timeout: Duration,
    /// Optional User-Agent.
    pub user_agent: Option<String>,
}

impl DogSmsConfig {
    /// Start building a DogeSMS client config with default Control API settings.
    pub fn builder(api_key: impl Into<String>) -> DogSmsConfigBuilder {
        DogSmsConfigBuilder {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_owned(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            user_agent: Some(default_user_agent()),
        }
    }

    /// Validate local configuration invariants before any network IO.
    pub fn validate(&self) -> anyhow::Result<()> {
        ensure_non_blank("api_key", &self.api_key)?;
        ensure_non_blank("base_url", &self.base_url)?;
        ensure_non_zero_duration("connect_timeout", self.connect_timeout)?;
        ensure_non_zero_duration("request_timeout", self.request_timeout)?;
        Ok(())
    }
}

/// Builder for [`DogSmsConfig`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DogSmsConfigBuilder {
    api_key: String,
    base_url: String,
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
}

impl DogSmsConfigBuilder {
    /// Override the Control API base URL.
    pub fn base_url(mut self, value: impl Into<String>) -> Self {
        self.base_url = value.into();
        self
    }

    /// Set the HTTP connect timeout.
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// Set the HTTP request timeout.
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// Set a custom User-Agent.
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    /// Remove the default User-Agent.
    pub fn clear_user_agent(mut self) -> Self {
        self.user_agent = None;
        self
    }

    /// Build and validate the config.
    pub fn build(self) -> anyhow::Result<DogSmsConfig> {
        let config = DogSmsConfig {
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

/// DogeSMS Control API client.
///
/// The explicit call flow is:
///
/// 1. Read account and inventory with [`Self::balance`], [`Self::services`], or [`Self::inventory`].
/// 2. Create a paid order with [`Self::create_activation`] or [`Self::create_rental`].
/// 3. Track or stop the order with [`Self::activation`] and [`Self::cancel_activation`].
///
/// DogeSMS returns string identifiers such as `requestId`, so the native methods
/// are the primary API for this provider.
#[derive(Clone, Debug)]
pub struct DogSmsClient {
    client: reqwest::Client,
    base_url: Url,
    api_key: String,
}

impl DogSmsClient {
    /// Create a client from a static DogeSMS API key.
    pub fn from_api_key(api_key: impl Into<String>) -> anyhow::Result<Self> {
        Self::new(DogSmsConfig::builder(api_key).build()?)
    }

    /// Create a client from explicit config.
    pub fn new(config: DogSmsConfig) -> anyhow::Result<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .with_context(|| format!("invalid base url `{}`", config.base_url))?;

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

    /// 1. Query account balance with `GET /api/control/balance`.
    pub async fn balance(&self) -> anyhow::Result<DogSmsBalance> {
        self.get_json(self.endpoint(&["balance"])?).await
    }

    /// 1. Query supported services with `GET /api/control/services`.
    pub async fn services(&self) -> anyhow::Result<Vec<DogSmsService>> {
        let response: DogSmsListResponse<DogSmsService> =
            self.get_json(self.endpoint(&["services"])?).await?;
        Ok(response.into_items())
    }

    /// 1. Query inventory with `GET /api/control/inventory`.
    pub async fn inventory(
        &self,
        service_code: impl AsRef<str>,
        country_code: Option<&str>,
    ) -> anyhow::Result<Vec<DogSmsInventoryItem>> {
        validate_request_non_blank("service_code", service_code.as_ref())?;
        validate_optional_request_non_blank("country_code", country_code)?;

        let mut url = self.endpoint(&["inventory"])?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("serviceCode", service_code.as_ref().trim());
            if let Some(country_code) = country_code.map(str::trim).filter(|item| !item.is_empty())
            {
                query.append_pair("countryCode", country_code);
            }
        }

        let response: DogSmsListResponse<DogSmsInventoryItem> = self.get_json(url).await?;
        Ok(response.into_items())
    }

    /// 2. Create an OTP activation with `POST /api/control/activations`.
    ///
    /// This convenience method generates a fresh `Idempotency-Key`. Use
    /// [`Self::create_activation_with_idempotency_key`] when the caller needs a
    /// persisted key for retry safety across process boundaries.
    pub async fn create_activation(
        &self,
        request: DogSmsActivationRequest,
    ) -> anyhow::Result<DogSmsActivationOrder> {
        self.create_activation_with_idempotency_key(request, generated_idempotency_key())
            .await
    }

    /// 2. Create an OTP activation with an explicit `Idempotency-Key`.
    pub async fn create_activation_with_idempotency_key(
        &self,
        request: DogSmsActivationRequest,
        idempotency_key: impl AsRef<str>,
    ) -> anyhow::Result<DogSmsActivationOrder> {
        request.validate()?;
        self.post_json(
            self.endpoint(&["activations"])?,
            &request,
            idempotency_key.as_ref(),
        )
        .await
    }

    /// 2. Create a long-term rental with `POST /api/control/rentals`.
    pub async fn create_rental(
        &self,
        request: DogSmsRentalRequest,
    ) -> anyhow::Result<DogSmsRentalOrder> {
        self.create_rental_with_idempotency_key(request, generated_idempotency_key())
            .await
    }

    /// 2. Create a long-term rental with an explicit `Idempotency-Key`.
    pub async fn create_rental_with_idempotency_key(
        &self,
        request: DogSmsRentalRequest,
        idempotency_key: impl AsRef<str>,
    ) -> anyhow::Result<DogSmsRentalOrder> {
        request.validate()?;
        self.post_json(
            self.endpoint(&["rentals"])?,
            &request,
            idempotency_key.as_ref(),
        )
        .await
    }

    /// 3. Query activation status and SMS content with `GET /api/control/activations/{requestId}`.
    pub async fn activation(
        &self,
        request_id: impl AsRef<str>,
    ) -> anyhow::Result<DogSmsActivationOrder> {
        validate_request_non_blank("request_id", request_id.as_ref())?;
        self.get_json(self.endpoint(&["activations", request_id.as_ref().trim()])?)
            .await
    }

    /// 3. Cancel a pending activation with `PATCH /api/control/activations/{requestId}/cancel`.
    pub async fn cancel_activation(
        &self,
        request_id: impl AsRef<str>,
    ) -> anyhow::Result<DogSmsActivationOrder> {
        self.cancel_activation_with_idempotency_key(request_id, generated_idempotency_key())
            .await
    }

    /// 3. Cancel a pending activation with an explicit `Idempotency-Key`.
    pub async fn cancel_activation_with_idempotency_key(
        &self,
        request_id: impl AsRef<str>,
        idempotency_key: impl AsRef<str>,
    ) -> anyhow::Result<DogSmsActivationOrder> {
        validate_request_non_blank("request_id", request_id.as_ref())?;
        self.patch_json(
            self.endpoint(&["activations", request_id.as_ref().trim(), "cancel"])?,
            idempotency_key.as_ref(),
        )
        .await
    }

    fn endpoint(&self, segments: &[&str]) -> anyhow::Result<Url> {
        let mut url = self.base_url.clone();
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| anyhow!("invalid endpoint: {}", self.base_url))?;
            path.pop_if_empty();
            for segment in segments {
                if segment.trim().is_empty() {
                    bail!("invalid endpoint: path segment cannot be blank");
                }
                path.push(segment.trim_matches('/'));
            }
        }
        Ok(url)
    }

    async fn get_json<T: DeserializeOwned>(&self, url: Url) -> anyhow::Result<T> {
        let response = self
            .client
            .get(url)
            .header("X-API-Key", self.api_key.trim())
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        self.parse_json_response(response).await
    }

    async fn post_json<T, B>(&self, url: Url, body: &B, idempotency_key: &str) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        validate_request_non_blank("idempotency_key", idempotency_key)?;
        let response = self
            .client
            .post(url)
            .header("X-API-Key", self.api_key.trim())
            .header("Idempotency-Key", idempotency_key.trim())
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;
        self.parse_json_response(response).await
    }

    async fn patch_json<T>(&self, url: Url, idempotency_key: &str) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        validate_request_non_blank("idempotency_key", idempotency_key)?;
        let response = self
            .client
            .patch(url)
            .header("X-API-Key", self.api_key.trim())
            .header("Idempotency-Key", idempotency_key.trim())
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        self.parse_json_response(response).await
    }

    async fn parse_json_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> anyhow::Result<T> {
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            let status_code = Some(status.as_u16());
            let error = provider_error(status_code, body);

            return Err(error);
        }

        match serde_json::from_str::<T>(&body) {
            Ok(value) => Ok(value),
            Err(_error) if looks_like_provider_message(&body) => {
                let status_code = Some(status.as_u16());
                let error = provider_error(status_code, body);

                Err(error)
            }
            Err(error) => Err(error).context("failed to parse DogeSMS JSON payload"),
        }
    }
}

/// Native DogeSMS activation request body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DogSmsActivationRequest {
    /// DogeSMS service code, for example `whatsapp` or `telegram`.
    #[serde(rename = "serviceCode")]
    pub service_code: String,
    /// DogeSMS country code, for example `US`.
    #[serde(rename = "countryCode")]
    pub country_code: String,
    /// Prefer reusable inventory when DogeSMS supports it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reuse: Option<bool>,
}

impl DogSmsActivationRequest {
    /// Create a validated native DogeSMS activation request.
    pub fn new(
        service_code: impl Into<String>,
        country_code: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let request = Self {
            service_code: service_code.into(),
            country_code: country_code.into(),
            reuse: None,
        };
        request.validate()?;
        Ok(request)
    }

    /// Enable or disable reusable inventory.
    pub fn reuse(mut self, value: bool) -> Self {
        self.reuse = Some(value);
        self
    }

    /// Convert the crate-level request shape into DogeSMS native field names.
    pub fn from_sms_request(request: &SmsActivationRequest) -> anyhow::Result<Self> {
        request.validate()?;
        validate_activation_support(request)?;
        Ok(Self {
            service_code: request.product.trim().to_owned(),
            country_code: request.country.trim().to_owned(),
            reuse: request.reuse,
        })
    }

    /// Validate local request invariants before network IO.
    pub fn validate(&self) -> anyhow::Result<()> {
        validate_request_non_blank("service_code", &self.service_code)?;
        validate_request_non_blank("country_code", &self.country_code)?;
        Ok(())
    }
}

/// Native DogeSMS long-term rental request body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DogSmsRentalRequest {
    /// Country used to allocate rental inventory.
    #[serde(rename = "countryCode")]
    pub country_code: String,
    /// Rental duration in months, normally 1 through 12.
    pub months: u8,
    /// Optional business note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl DogSmsRentalRequest {
    /// Create a validated native DogeSMS rental request.
    pub fn new(country_code: impl Into<String>, months: u8) -> anyhow::Result<Self> {
        let request = Self {
            country_code: country_code.into(),
            months,
            note: None,
        };
        request.validate()?;
        Ok(request)
    }

    /// Attach a business note to the rental order.
    pub fn note(mut self, value: impl Into<String>) -> Self {
        self.note = Some(value.into());
        self
    }

    /// Validate local request invariants before network IO.
    pub fn validate(&self) -> anyhow::Result<()> {
        validate_request_non_blank("country_code", &self.country_code)?;
        if !(1..=12).contains(&self.months) {
            bail!("invalid request: months must be between 1 and 12");
        }
        validate_optional_request_non_blank("note", self.note.as_deref())?;
        Ok(())
    }
}

/// Account balance returned by `GET /api/control/balance`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DogSmsBalance {
    /// Available account balance.
    #[serde(
        alias = "available",
        alias = "availableBalance",
        deserialize_with = "deserialize_f64ish"
    )]
    pub balance: f64,
    /// Frozen balance when exposed by DogeSMS.
    #[serde(
        default,
        alias = "frozen",
        alias = "frozenBalance",
        deserialize_with = "deserialize_optional_f64ish"
    )]
    pub frozen_balance: Option<f64>,
    /// Balance currency when exposed by DogeSMS.
    #[serde(default)]
    pub currency: Option<String>,
}

/// Service item returned by `GET /api/control/services`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DogSmsService {
    /// DogeSMS service code.
    #[serde(alias = "serviceCode")]
    pub code: String,
    /// Human readable service name.
    #[serde(default)]
    pub name: String,
    /// Service category when exposed by DogeSMS.
    #[serde(default)]
    pub category: Option<String>,
    /// Whether reusable inventory is supported.
    #[serde(default, alias = "supportsReuse")]
    pub supports_reuse: bool,
}

/// Inventory item returned by `GET /api/control/inventory`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DogSmsInventoryItem {
    /// Service code scoped by the inventory row.
    #[serde(default, alias = "serviceCode")]
    pub service_code: String,
    /// Country code scoped by the inventory row.
    #[serde(alias = "countryCode")]
    pub country_code: String,
    /// Starting price or selected price for the country-service pair.
    #[serde(
        default,
        alias = "price",
        alias = "cost",
        deserialize_with = "deserialize_f64ish_default"
    )]
    pub cost: f64,
    /// Price currency.
    #[serde(default)]
    pub currency: Option<String>,
    /// Currently available inventory count.
    #[serde(default, alias = "availableCount")]
    pub available_count: u64,
    /// Whether reusable inventory can be requested.
    #[serde(default, alias = "canReuse")]
    pub can_reuse: bool,
    /// Provider timestamp for the inventory snapshot.
    #[serde(default, alias = "lastUpdated")]
    pub last_updated: Option<String>,
}

/// DogeSMS activation lifecycle status.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum DogSmsActivationStatus {
    /// Order is allocated or waiting for SMS delivery.
    #[default]
    Pending,
    /// SMS content has arrived.
    Received,
    /// Order was cancelled before completion.
    Canceled,
    /// Order expired before SMS delivery.
    Expired,
    /// Provider marked the order complete.
    Finished,
    /// Provider returned a status this crate does not yet classify.
    Unknown(String),
}

impl DogSmsActivationStatus {
    /// Convert to the crate-level order status.
    pub fn as_sms_order_status(&self) -> SmsOrderStatus {
        match self {
            Self::Pending => SmsOrderStatus::Pending,
            Self::Received => SmsOrderStatus::Received,
            Self::Canceled => SmsOrderStatus::Canceled,
            Self::Expired => SmsOrderStatus::Timeout,
            Self::Finished => SmsOrderStatus::Finished,
            Self::Unknown(_) => SmsOrderStatus::Unknown,
        }
    }

    fn from_provider(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "pending" | "waiting" | "allocated" | "processing" => Self::Pending,
            "received" | "sms_received" | "delivered" => Self::Received,
            "cancelled" | "canceled" | "cancel" => Self::Canceled,
            "expired" | "timeout" | "timed_out" => Self::Expired,
            "finished" | "completed" | "done" => Self::Finished,
            _ => Self::Unknown(value.trim().to_owned()),
        }
    }
}

impl<'de> Deserialize<'de> for DogSmsActivationStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| Self::from_provider(&value))
    }
}

/// DogeSMS SMS payload.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DogSmsMessage {
    /// Provider message identifier when available.
    #[serde(default, deserialize_with = "deserialize_optional_stringish")]
    pub id: Option<String>,
    /// Provider created timestamp.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// Sender-side timestamp.
    #[serde(default, alias = "dateTime")]
    pub date: Option<String>,
    /// Sender name or phone number.
    #[serde(default)]
    pub sender: String,
    /// Raw SMS body.
    #[serde(default)]
    pub text: String,
    /// Provider-extracted OTP code.
    #[serde(default)]
    pub code: Option<String>,
}

impl DogSmsMessage {
    fn into_sms_message(self) -> SmsMessage {
        SmsMessage {
            id: self.id.as_deref().and_then(|id| id.trim().parse().ok()),
            created_at: self.created_at,
            date: self.date,
            sender: self.sender,
            text: self.text,
            code: self.code,
        }
    }
}

/// Activation order returned by create, query, and cancel activation endpoints.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DogSmsActivationOrder {
    /// String request identifier returned by DogeSMS.
    #[serde(
        rename = "requestId",
        alias = "id",
        alias = "activationId",
        deserialize_with = "deserialize_stringish"
    )]
    pub request_id: String,
    /// Allocated phone number.
    #[serde(default, alias = "phone", alias = "phoneNumber")]
    pub number: String,
    /// Current activation status.
    #[serde(default)]
    pub status: DogSmsActivationStatus,
    /// Service code when exposed by DogeSMS.
    #[serde(default, alias = "serviceCode")]
    pub service_code: Option<String>,
    /// Country code when exposed by DogeSMS.
    #[serde(default, alias = "countryCode")]
    pub country_code: Option<String>,
    /// Order price when exposed by DogeSMS.
    #[serde(
        default,
        alias = "price",
        alias = "cost",
        deserialize_with = "deserialize_f64ish_default"
    )]
    pub price: f64,
    /// Expiration timestamp.
    #[serde(default, alias = "expiresAt")]
    pub expires_at: Option<String>,
    /// Reuse window timestamp.
    #[serde(default, alias = "canReuseUntil")]
    pub can_reuse_until: Option<String>,
    /// Creation timestamp when exposed by DogeSMS.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// SMS messages associated with the activation.
    #[serde(
        default,
        alias = "messages",
        alias = "latestSms",
        deserialize_with = "deserialize_sms_messages"
    )]
    pub sms: Vec<DogSmsMessage>,
}

impl DogSmsActivationOrder {
    /// Convert into the crate-level order model when `request_id` is numeric.
    ///
    /// DogeSMS documents string IDs such as `act-123`; callers that need full
    /// string-ID support should keep using this native type.
    pub fn try_into_sms_order(self) -> anyhow::Result<SmsOrder> {
        let id = self.request_id.trim().parse::<u64>().map_err(|_| {
            anyhow!(
                "invalid request: DogeSMS request_id `{}` is not numeric",
                self.request_id
            )
        })?;
        Ok(SmsOrder {
            id,
            phone: self.number,
            operator: None,
            product: self.service_code.unwrap_or_default(),
            price: self.price,
            status: self.status.as_sms_order_status(),
            expires: self.expires_at,
            sms: self
                .sms
                .into_iter()
                .map(DogSmsMessage::into_sms_message)
                .collect(),
            created_at: self.created_at,
            forwarding: None,
            forwarding_number: None,
            country: self.country_code,
        })
    }
}

#[async_trait::async_trait]
impl SmsProvider for DogSmsClient {
    async fn buy_activation_number(&self, request: SmsActivationRequest) -> anyhow::Result<SmsOrder> {
        let request = DogSmsActivationRequest::from_sms_request(&request)?;
        self.create_activation(request).await?.try_into_sms_order()
    }

    async fn buy_hosting_number(&self, _request: SmsHostingRequest) -> anyhow::Result<SmsOrder> {
        Err(unsupported(
            "hosted/rented numbers through numeric SmsProvider orders",
        ))
    }

    async fn check_order(&self, order_id: u64) -> anyhow::Result<SmsOrder> {
        self.activation(order_id.to_string())
            .await?
            .try_into_sms_order()
    }

    async fn finish_order(&self, _order_id: u64) -> anyhow::Result<SmsOrder> {
        Err(unsupported("finish activation"))
    }

    async fn cancel_order(&self, order_id: u64) -> anyhow::Result<SmsOrder> {
        self.cancel_activation(order_id.to_string())
            .await?
            .try_into_sms_order()
    }

    async fn ban_order(&self, _order_id: u64) -> anyhow::Result<SmsOrder> {
        Err(unsupported("ban activation"))
    }

    async fn inbox(&self, _order_id: u64) -> anyhow::Result<SmsInbox> {
        Err(unsupported("hosted/rented inbox"))
    }
}

/// Rental order returned by `POST /api/control/rentals`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DogSmsRentalOrder {
    /// String rental identifier returned by DogeSMS.
    #[serde(
        rename = "rentalId",
        alias = "id",
        deserialize_with = "deserialize_stringish"
    )]
    pub rental_id: String,
    /// Allocated rental phone number.
    #[serde(default, alias = "phone", alias = "phoneNumber")]
    pub number: String,
    /// Country code.
    #[serde(default, alias = "countryCode")]
    pub country_code: Option<String>,
    /// Rental duration in months.
    #[serde(default)]
    pub months: u8,
    /// Rental status.
    #[serde(default)]
    pub status: Option<String>,
    /// Rental expiration timestamp.
    #[serde(default, alias = "expiresAt")]
    pub expires_at: Option<String>,
    /// Whether auto-renew is enabled.
    #[serde(default, alias = "autoRenew")]
    pub auto_renew: Option<bool>,
    /// Bound service code when exposed by DogeSMS.
    #[serde(default, alias = "serviceCode")]
    pub service_code: Option<String>,
    /// Rental price.
    #[serde(
        default,
        alias = "price",
        alias = "cost",
        deserialize_with = "deserialize_f64ish_default"
    )]
    pub price: f64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DogSmsListResponse<T> {
    Raw(Vec<T>),
    Data { data: Vec<T> },
    Items { items: Vec<T> },
    Services { services: Vec<T> },
    Inventory { inventory: Vec<T> },
}

impl<T> DogSmsListResponse<T> {
    fn into_items(self) -> Vec<T> {
        match self {
            Self::Raw(items)
            | Self::Data { data: items }
            | Self::Items { items }
            | Self::Services { services: items }
            | Self::Inventory { inventory: items } => items,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrNumber {
    String(String),
    Number(serde_json::Number),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DogSmsSmsPayload {
    Many(Vec<DogSmsMessage>),
    One(DogSmsMessage),
}

fn validate_activation_support(request: &SmsActivationRequest) -> anyhow::Result<()> {
    if !request.operator.trim().eq_ignore_ascii_case("any") {
        let error = unsupported("operator-specific activation requests");

        return Err(error);
    }
    if request.forwarding.is_some() {
        let error = unsupported("activation forwarding");

        return Err(error);
    }
    if request.number.is_some() {
        let error = unsupported("number reuse by explicit phone number");

        return Err(error);
    }
    if request.voice.is_some() {
        let error = unsupported("voice verification flags");

        return Err(error);
    }
    if request.ref_code.is_some() {
        let error = unsupported("referral code query parameters");

        return Err(error);
    }
    Ok(())
}

fn validate_request_non_blank(name: &str, value: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        bail!("invalid request: {name} cannot be blank");
    }
    Ok(())
}

fn validate_optional_request_non_blank(name: &str, value: Option<&str>) -> anyhow::Result<()> {
    if value.is_some_and(|item| item.trim().is_empty()) {
        bail!("invalid request: {name} cannot be blank");
    }
    Ok(())
}

fn generated_idempotency_key() -> String {
    Uuid::new_v4().to_string()
}

fn unsupported(operation: &'static str) -> anyhow::Error {
    anyhow!("{PROVIDER_NAME} does not support {operation}")
}

fn deserialize_stringish<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(value) => Ok(value),
        StringOrNumber::Number(value) => Ok(value.to_string()),
    }
}

fn deserialize_optional_stringish<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<StringOrNumber>::deserialize(deserializer).map(|value| {
        value.map(|item| match item {
            StringOrNumber::String(value) => value,
            StringOrNumber::Number(value) => value.to_string(),
        })
    })
}

fn deserialize_f64ish<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(value) => value.trim().parse().map_err(serde::de::Error::custom),
        StringOrNumber::Number(value) => value
            .as_f64()
            .ok_or_else(|| serde::de::Error::custom("number is not an f64")),
    }
}

fn deserialize_f64ish_default<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<StringOrNumber>::deserialize(deserializer)?
        .map(|value| match value {
            StringOrNumber::String(value) => value.trim().parse().map_err(serde::de::Error::custom),
            StringOrNumber::Number(value) => value
                .as_f64()
                .ok_or_else(|| serde::de::Error::custom("number is not an f64")),
        })
        .unwrap_or(Ok(0.0))
}

fn deserialize_optional_f64ish<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<StringOrNumber>::deserialize(deserializer)?
        .map(|value| match value {
            StringOrNumber::String(value) => value.trim().parse().map_err(serde::de::Error::custom),
            StringOrNumber::Number(value) => value
                .as_f64()
                .ok_or_else(|| serde::de::Error::custom("number is not an f64")),
        })
        .transpose()
}

fn deserialize_sms_messages<'de, D>(deserializer: D) -> Result<Vec<DogSmsMessage>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(
        match Option::<DogSmsSmsPayload>::deserialize(deserializer)? {
            Some(DogSmsSmsPayload::Many(messages)) => messages,
            Some(DogSmsSmsPayload::One(message)) => vec![message],
            None => Vec::new(),
        },
    )
}
