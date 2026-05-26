use crate::error::{SmsError, SmsResult};
use az_derive_aliases::{apply, plain_copy_eq, serde_eq, serde_partial_eq, serde_upper_eq};
use serde::Deserialize;
use serde::de::{self, Deserializer};
use std::time::Duration;

/// Request for a one-time SMS activation number.
#[apply(serde_eq)]
pub struct SmsActivationRequest {
    /// Provider country code, or provider-specific `any`.
    pub country: String,
    /// Provider operator code, or provider-specific `any`.
    pub operator: String,
    /// Provider product or service name.
    pub product: String,
    /// Enable call forwarding when the provider supports it.
    pub forwarding: Option<bool>,
    /// Existing phone number for provider-specific reuse flows.
    pub number: Option<String>,
    /// Request a reusable number when the provider supports it.
    pub reuse: Option<bool>,
    /// Request voice verification when the provider supports it.
    pub voice: Option<bool>,
    /// Provider referral code.
    #[serde(rename = "ref")]
    pub ref_code: Option<String>,
}

impl SmsActivationRequest {
    /// Create a validated activation request.
    pub fn new(
        country: impl Into<String>,
        operator: impl Into<String>,
        product: impl Into<String>,
    ) -> SmsResult<Self> {
        let request = Self {
            country: country.into(),
            operator: operator.into(),
            product: product.into(),
            forwarding: None,
            number: None,
            reuse: None,
            voice: None,
            ref_code: None,
        };
        request.validate()?;
        Ok(request)
    }

    /// Set call forwarding.
    pub fn forwarding(mut self, value: bool) -> Self {
        self.forwarding = Some(value);
        self
    }

    /// Set the provider-specific phone number reuse value.
    pub fn number(mut self, value: impl Into<String>) -> Self {
        self.number = Some(value.into());
        self
    }

    /// Enable or disable number reuse.
    pub fn reuse(mut self, value: bool) -> Self {
        self.reuse = Some(value);
        self
    }

    /// Enable or disable voice verification.
    pub fn voice(mut self, value: bool) -> Self {
        self.voice = Some(value);
        self
    }

    /// Set the provider referral code.
    pub fn ref_code(mut self, value: impl Into<String>) -> Self {
        self.ref_code = Some(value.into());
        self
    }

    /// Validate local invariants before sending the request to a provider.
    pub fn validate(&self) -> SmsResult<()> {
        validate_non_blank("country", &self.country)?;
        validate_non_blank("operator", &self.operator)?;
        validate_non_blank("product", &self.product)?;
        validate_optional_non_blank("number", self.number.as_deref())?;
        validate_optional_non_blank("ref_code", self.ref_code.as_deref())?;
        Ok(())
    }
}

/// Request for a longer-lived hosted/rented SMS number.
#[apply(serde_eq)]
pub struct SmsHostingRequest {
    /// Provider country code, or provider-specific `any`.
    pub country: String,
    /// Provider operator code, or provider-specific `any`.
    pub operator: String,
    /// Provider product or service name.
    pub product: String,
}

impl SmsHostingRequest {
    /// Create a validated hosting request.
    pub fn new(
        country: impl Into<String>,
        operator: impl Into<String>,
        product: impl Into<String>,
    ) -> SmsResult<Self> {
        let request = Self {
            country: country.into(),
            operator: operator.into(),
            product: product.into(),
        };
        request.validate()?;
        Ok(request)
    }

    /// Validate local invariants before sending the request to a provider.
    pub fn validate(&self) -> SmsResult<()> {
        validate_non_blank("country", &self.country)?;
        validate_non_blank("operator", &self.operator)?;
        validate_non_blank("product", &self.product)?;
        Ok(())
    }
}

/// Provider order state.
#[apply(serde_upper_eq)]
pub enum SmsOrderStatus {
    /// Provider is preparing the number.
    Pending,
    /// SMS has been received or the order is waiting for receipt.
    Received,
    /// Order was canceled.
    Canceled,
    /// Provider timed out the order.
    Timeout,
    /// Order was successfully finished.
    Finished,
    /// Order was banned/rejected.
    Banned,
    /// Provider returned a status not known by this crate.
    #[serde(other)]
    Unknown,
}

impl SmsOrderStatus {
    /// Whether the status closes the order lifecycle.
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Canceled | Self::Timeout | Self::Finished | Self::Banned
        )
    }
}

/// A single SMS message returned by a provider.
#[apply(serde_eq)]
pub struct SmsMessage {
    /// Provider SMS ID when supplied.
    #[serde(default, alias = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// Provider creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Sender-side message timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Sender name or phone number.
    #[serde(default)]
    pub sender: String,
    /// Raw SMS body.
    #[serde(default)]
    pub text: String,
    /// Provider-extracted verification code, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// SMS provider order data.
#[apply(serde_partial_eq)]
pub struct SmsOrder {
    /// Provider order ID.
    pub id: u64,
    /// Rented phone number.
    pub phone: String,
    /// Provider operator name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    /// Provider product or service name.
    pub product: String,
    /// Provider order price.
    pub price: f64,
    /// Current provider order status.
    pub status: SmsOrderStatus,
    /// Provider expiry timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
    /// SMS messages attached to the order.
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub sms: Vec<SmsMessage>,
    /// Provider creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Whether forwarding was enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forwarding: Option<bool>,
    /// Forwarding destination when enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forwarding_number: Option<String>,
    /// Provider country name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

/// SMS inbox for providers that expose rented-number inboxes.
#[apply(serde_eq)]
pub struct SmsInbox {
    /// SMS messages returned by the provider.
    #[serde(rename = "Data", default)]
    pub messages: Vec<SmsMessage>,
    /// Total messages according to the provider.
    #[serde(rename = "Total", default)]
    pub total: usize,
}

/// Minimal provider account profile.
#[apply(serde_partial_eq)]
pub struct SmsProfile {
    /// Provider account ID.
    pub id: u64,
    /// Provider account email.
    #[serde(default)]
    pub email: String,
    /// Current account balance.
    pub balance: f64,
    /// Provider rating.
    #[serde(default)]
    pub rating: f64,
    /// Frozen balance when the provider exposes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen_balance: Option<f64>,
}

/// Polling options used by [`crate::provider::SmsProvider::wait_for_sms`].
#[apply(plain_copy_eq)]
pub struct WaitForSmsOptions {
    /// Maximum time spent polling.
    pub timeout: Duration,
    /// Delay between `check_order` calls.
    pub interval: Duration,
}

impl Default for WaitForSmsOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300),
            interval: Duration::from_secs(5),
        }
    }
}

impl WaitForSmsOptions {
    /// Create polling options and validate that both durations are non-zero.
    pub fn new(timeout: Duration, interval: Duration) -> SmsResult<Self> {
        let options = Self { timeout, interval };
        options.validate()?;
        Ok(options)
    }

    /// Validate local polling invariants.
    pub fn validate(&self) -> SmsResult<()> {
        if self.timeout.is_zero() {
            return Err(SmsError::InvalidRequest(
                "timeout cannot be zero".to_owned(),
            ));
        }
        if self.interval.is_zero() {
            return Err(SmsError::InvalidRequest(
                "interval cannot be zero".to_owned(),
            ));
        }
        Ok(())
    }
}

fn validate_non_blank(name: &str, value: &str) -> SmsResult<()> {
    if value.trim().is_empty() {
        return Err(SmsError::InvalidRequest(format!("{name} cannot be blank")));
    }
    Ok(())
}

fn validate_optional_non_blank(name: &str, value: Option<&str>) -> SmsResult<()> {
    if value.is_some_and(|item| item.trim().is_empty()) {
        return Err(SmsError::InvalidRequest(format!("{name} cannot be blank")));
    }
    Ok(())
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<Vec<T>>::deserialize(deserializer).map(|items| items.unwrap_or_default())
}

#[allow(dead_code)]
fn reject_unknown<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.trim().is_empty() {
        Err(de::Error::custom("value cannot be blank"))
    } else {
        Ok(value)
    }
}
