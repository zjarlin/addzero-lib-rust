#![forbid(unsafe_code)]
//! SMS provider abstraction and provider clients.
//!
//! The crate exposes a small provider trait for renting numbers, checking SMS
//! delivery, and closing orders. It intentionally keeps account-creation or
//! verification-bypass workflows out of the library boundary.
//!
//! # Quick Start
//!
//! ```no_run
//! use az_sms::{FivesimClient, SmsActivationRequest, SmsProvider};
//!
//! # async fn example() -> az_sms::SmsResult<()> {
//! let client = FivesimClient::from_token("token")?;
//! let order = client
//!     .buy_activation_number(SmsActivationRequest::new("usa", "any", "telegram")?)
//!     .await?;
//! let order = client.wait_for_sms(order.id, Default::default()).await?;
//! println!("{:?}", order.sms.first().and_then(|message| message.code.as_deref()));
//! # Ok(())
//! # }
//! ```

mod error;
mod fivesim;
mod model;
mod provider;

pub use error::{ProviderStatus, SmsError, SmsResult};
pub use fivesim::{FivesimClient, FivesimConfig, FivesimConfigBuilder};
pub use model::{
    SmsActivationRequest, SmsHostingRequest, SmsInbox, SmsMessage, SmsOrder, SmsOrderStatus,
    SmsProfile, WaitForSmsOptions,
};
pub use provider::SmsProvider;
