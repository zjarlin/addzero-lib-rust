#![forbid(unsafe_code)]
//! Authorized Gmail API helpers for extracting verification codes from owned mailboxes.
//!
//! This crate wraps the Gmail `users.messages.list` and `users.messages.get` endpoints
//! for the narrow use case of reading one-time verification codes from a mailbox the
//! caller controls. Callers supply an OAuth access token; this crate does not perform
//! login, account creation, or unauthorized mailbox access. Use `az-oauth2` for the
//! standard OAuth2 authorization-code or device-flow steps.
//!
//! # Example
//!
//! ```no_run
//! use az_gmail_code::{GmailCodeClient, GmailCodeQuery};
//!
//! # fn example() -> az_gmail_code::GmailCodeResult<()> {
//! let client = GmailCodeClient::new("ya29.access-token")?;
//! let code = client.find_latest_code(
//!     GmailCodeQuery::new()
//!         .from("security@example.com")
//!         .subject("verification")
//!         .newer_than("10m")
//!         .unread(true),
//! )?;
//!
//! if let Some(code) = code {
//!     println!("{} from {}", code.code, code.message_id);
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod config;
mod error;
mod model;
mod parser;

pub use client::GmailCodeClient;
pub use config::{GmailCodeConfig, GmailCodeConfigBuilder, GmailCodeQuery};
pub use error::{GmailCodeError, GmailCodeResult};
pub use model::{
    ExtractedGmailCode, GmailMessage, GmailMessageHeader, GmailMessagePart, GmailMessagePartBody,
};
pub use parser::{
    ExtractCodeOptions, MessageBodyCandidate, collect_message_body_candidates,
    extract_verification_code, extract_verification_code_with_options,
};
