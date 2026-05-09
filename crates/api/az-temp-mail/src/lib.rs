#![forbid(unsafe_code)]
//! Client for deployed [`cloudflare_temp_email`](https://github.com/dreamhunter2333/cloudflare_temp_email) workers.
//!
//! The upstream project is a Cloudflare Workers temp-email application, not a
//! hosted global API. This crate wraps the worker's public address, inbox, and
//! send-mail endpoints while keeping the deployment URL explicit.
//!
//! # Quick Start
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

mod client;
mod config;
mod error;
mod http;
mod model;
mod util;

pub use client::{PageRequest, TempMailApi, create_temp_mail_api};
pub use config::{ApiConfig, ApiConfigBuilder};
pub use error::{TempMailError, TempMailResult};
pub use model::{
    AddressCredential, AddressLoginRequest, AddressSettings, ListResponse, MailRow,
    NewAddressRequest, ParsedMailAttachment, ParsedMailRow, SendMailRequest, SuccessResponse,
    TempMailSettings,
};

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
}
