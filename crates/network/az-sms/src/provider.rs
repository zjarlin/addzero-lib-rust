use crate::error::{SmsError, SmsResult};
use crate::fivesim::{FivesimClient, FivesimConfig};
use crate::grizzlysms::{GrizzlySmsClient, GrizzlySmsConfig};
use crate::model::{
    SmsActivationRequest, SmsHostingRequest, SmsInbox, SmsOrder, WaitForSmsOptions,
};
use az_derive_aliases::{apply, plain_default_copy_eq, plain_eq, serde_code_enum};
use std::time::Instant;

/// Built-in SMS provider identifiers.
#[apply(serde_code_enum)]
pub enum SmsProviderKind {
    /// 5sim v1 API.
    #[serde(rename = "5sim")]
    #[strum(serialize = "5sim")]
    Fivesim,
    /// Grizzly SMS sms-activate-compatible API.
    GrizzlySms,
}

/// Configuration for one built-in SMS provider.
#[apply(plain_eq)]
pub enum SmsProviderConfig {
    /// 5sim v1 API config.
    Fivesim(FivesimConfig),
    /// Grizzly SMS API config.
    GrizzlySms(GrizzlySmsConfig),
}

impl SmsProviderConfig {
    /// Return the provider kind represented by this config.
    #[must_use]
    pub const fn kind(&self) -> SmsProviderKind {
        match self {
            Self::Fivesim(_) => SmsProviderKind::Fivesim,
            Self::GrizzlySms(_) => SmsProviderKind::GrizzlySms,
        }
    }
}

impl From<FivesimConfig> for SmsProviderConfig {
    fn from(value: FivesimConfig) -> Self {
        Self::Fivesim(value)
    }
}

impl From<GrizzlySmsConfig> for SmsProviderConfig {
    fn from(value: GrizzlySmsConfig) -> Self {
        Self::GrizzlySms(value)
    }
}

/// Boxed provider object used at application boundaries.
pub type BoxSmsProvider = Box<dyn SmsProvider + Send + Sync>;

/// Factory abstraction for dependency-injected SMS provider creation.
pub trait SmsProviderFactory: Send + Sync {
    /// Build a provider trait object from a provider-specific config.
    fn build_provider(&self, config: SmsProviderConfig) -> SmsResult<BoxSmsProvider>;
}

/// Factory for the providers compiled into this crate.
#[apply(plain_default_copy_eq)]
pub struct BuiltinSmsProviderFactory;

impl SmsProviderFactory for BuiltinSmsProviderFactory {
    fn build_provider(&self, config: SmsProviderConfig) -> SmsResult<BoxSmsProvider> {
        match config {
            SmsProviderConfig::Fivesim(config) => Ok(Box::new(FivesimClient::new(config)?)),
            SmsProviderConfig::GrizzlySms(config) => Ok(Box::new(GrizzlySmsClient::new(config)?)),
        }
    }
}

/// Build a provider trait object from a provider-specific config.
pub fn build_sms_provider(config: SmsProviderConfig) -> SmsResult<BoxSmsProvider> {
    BuiltinSmsProviderFactory.build_provider(config)
}

/// Common async interface implemented by SMS providers.
#[async_trait::async_trait]
pub trait SmsProvider: Send + Sync {
    /// Buy a one-time activation number.
    async fn buy_activation_number(&self, request: SmsActivationRequest) -> SmsResult<SmsOrder>;

    /// Buy a hosted/rented number when the provider supports it.
    async fn buy_hosting_number(&self, request: SmsHostingRequest) -> SmsResult<SmsOrder>;

    /// Fetch the current order state and attached SMS messages.
    async fn check_order(&self, order_id: u64) -> SmsResult<SmsOrder>;

    /// Mark an order as successfully finished.
    async fn finish_order(&self, order_id: u64) -> SmsResult<SmsOrder>;

    /// Cancel an order that should no longer be used.
    async fn cancel_order(&self, order_id: u64) -> SmsResult<SmsOrder>;

    /// Reject an order because the number or received SMS is unusable.
    async fn ban_order(&self, order_id: u64) -> SmsResult<SmsOrder>;

    /// Fetch SMS inbox messages for a hosted/rented order.
    async fn inbox(&self, order_id: u64) -> SmsResult<SmsInbox>;

    /// Poll `check_order` until an SMS arrives, the order closes, or the timeout expires.
    async fn wait_for_sms(&self, order_id: u64, options: WaitForSmsOptions) -> SmsResult<SmsOrder> {
        options.validate()?;
        let deadline = Instant::now() + options.timeout;

        loop {
            let order = self.check_order(order_id).await?;
            if !order.sms.is_empty() {
                return Ok(order);
            }
            if order.status.is_terminal() {
                return Err(SmsError::OrderClosed {
                    order_id,
                    status: order.status,
                });
            }
            if Instant::now() >= deadline {
                return Err(SmsError::Timeout {
                    order_id,
                    timeout_secs: options.timeout.as_secs(),
                });
            }
            tokio::time::sleep(options.interval).await;
        }
    }
}
