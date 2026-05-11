#![forbid(unsafe_code)]
//! 已部署 [`cloudflare_temp_email`](https://github.com/dreamhunter2333/cloudflare_temp_email) Workers 的客户端。
//!
//! 上游项目是一个 Cloudflare Workers 临时邮箱应用，并非托管的全局 API。
//! 此 crate 包装了 Worker 的公共地址、收件箱和发信端点，同时保持部署 URL 的显式性。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_temp_mail::{NewAddressRequest, PageRequest, create_temp_mail_api};
//!
//! # fn example() -> az_temp_mail::TempMailResult<()> {
//! let api = create_temp_mail_api("https://mail.example.com")?;
//! let address = api.new_address(&NewAddressRequest::new("demo", "example.com"))?;
//! let inbox = api.list_parsed_mails(&address.jwt, PageRequest::default())?;
//! println!("{} has {} messages", address.address, inbox.count);
//! # Ok(())
//! # }
//! ```
//!
//! ```no_run
//! use az_temp_mail::{CloudflareTempMailContext, PageRequest};
//!
//! # fn example() -> az_temp_mail::TempMailResult<()> {
//! let context = CloudflareTempMailContext {
//!     base_url: "https://mail.example.com".to_owned(),
//!     custom_auth: Some("admin-secret".to_owned()),
//!     address_name: Some("demo".to_owned()),
//!     address_domain: Some("example.com".to_owned()),
//!     cf_token: None,
//!     enable_random_subdomain: None,
//! };
//! let api = context.create_api()?;
//! let address = api.new_address(&context.new_address_request())?;
//! let inbox = api.list_parsed_mails(&address.jwt, PageRequest::default())?;
//! println!("{} has {} messages", address.address, inbox.count);
//! # Ok(())
//! # }
//! ```

mod client;
mod cloudflare;
mod config;
mod emailnator;
mod error;
mod http;
mod mail_tm;
mod model;
mod provider;
mod util;

pub use client::{CloudflareTempMailApi, TempMailApi, create_temp_mail_api};
pub use cloudflare::CloudflareTempMailContext;
pub use config::{ApiConfig, ApiConfigBuilder};
pub use emailnator::{
    EmailnatorEmailMode, EmailnatorEmailRequest, EmailnatorTempMailApi, create_emailnator_api,
    extract_first_http_link,
};
pub use error::{TempMailError, TempMailResult};
pub use mail_tm::{MailTmDomain, MailTmTempMailApi, create_mail_tm_api};
pub use model::{
    AddressCredential, AddressLoginRequest, AddressSettings, CreateMailboxRequest, ListResponse,
    MailRow, NewAddressRequest, PageRequest, ParsedMailAttachment, ParsedMailRow, SendMailRequest,
    SuccessResponse, TempMailMailbox, TempMailMessageDetail, TempMailMessageSummary,
    TempMailProviderKind, TempMailRecipient, TempMailSettings,
};
pub use provider::TempMailProvider;

/// Namespace-style entry point for constructing temp-mail clients.
#[derive(Debug, Clone, Copy, Default)]
pub struct TempMail;

impl TempMail {
    /// Creates a client for a deployed Cloudflare Temp Email worker.
    pub fn cloudflare(base_url: impl Into<String>) -> TempMailResult<TempMailApi> {
        create_temp_mail_api(base_url)
    }

    /// Creates a client from explicit configuration.
    pub fn cloudflare_with_config(config: ApiConfig) -> TempMailResult<TempMailApi> {
        TempMailApi::new(config)
    }

    /// Creates a client from a higher-level Cloudflare worker context.
    pub fn cloudflare_with_context(
        context: &CloudflareTempMailContext,
    ) -> TempMailResult<TempMailApi> {
        context.create_api()
    }

    /// Creates a client for the hosted mail.tm-compatible provider.
    pub fn mail_tm() -> TempMailResult<MailTmTempMailApi> {
        create_mail_tm_api()
    }

    /// Creates a mail.tm-compatible client from explicit configuration.
    pub fn mail_tm_with_config(config: ApiConfig) -> TempMailResult<MailTmTempMailApi> {
        MailTmTempMailApi::new(config)
    }

    /// Creates a client for the hosted Emailnator temporary mailbox service.
    pub fn emailnator() -> TempMailResult<EmailnatorTempMailApi> {
        create_emailnator_api()
    }

    /// Creates an Emailnator client from explicit configuration.
    pub fn emailnator_with_config(config: ApiConfig) -> TempMailResult<EmailnatorTempMailApi> {
        EmailnatorTempMailApi::new(config)
    }
}
