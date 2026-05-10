#![forbid(unsafe_code)]
//! SMS 提供商抽象和提供商客户端。
//!
//! 此 crate 暴露一个小型提供商 trait，用于租用号码、检查短信投递和关闭订单。
//! 它刻意将账户创建或验证绕过工作流排除在库边界之外。
//!
//! # 快速开始
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
