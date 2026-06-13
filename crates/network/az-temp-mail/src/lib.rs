#![forbid(unsafe_code)]
//! 已部署 [`cloudflare_temp_email`](https://github.com/dreamhunter2333/cloudflare_temp_email) Workers 的客户端。
//!
//! 上游项目是一个 Cloudflare Workers 临时邮箱应用，并非托管的全局 API。
//! 此 crate 包装了 Worker 的公共地址、收件箱和发信端点，同时保持部署 URL 的显式性。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_temp_mail::client::create_temp_mail_api;
//! use az_temp_mail::model::{NewAddressRequest, PageRequest};
//!
//! # fn example() -> anyhow::Result<()> {
//! let api = create_temp_mail_api("https://mail.example.com")?;
//! let address = api.new_address(&NewAddressRequest::new("demo", "example.com"))?;
//! let inbox = api.list_parsed_mails(&address.jwt, PageRequest::default())?;
//! println!("{} has {} messages", address.address, inbox.count);
//! # Ok(())
//! # }
//! ```
//!
//! ```no_run
//! use az_temp_mail::cloudflare::CloudflareTempMailContext;
//! use az_temp_mail::model::PageRequest;
//!
//! # fn example() -> anyhow::Result<()> {
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

automod::dir!(pub "src");
