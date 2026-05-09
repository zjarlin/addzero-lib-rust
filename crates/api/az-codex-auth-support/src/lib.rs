#![forbid(unsafe_code)]

//! Safe Rust support code extracted from the Python `codex_auto_register` project.
//!
//! This crate intentionally does not implement automated OpenAI account registration,
//! Sentinel proof-of-work generation, browser fingerprint impersonation, or proxy-based
//! risk-control bypass flows. It provides the reusable and auditable pieces around
//! DuckMail inbox access, OTP parsing, PKCE generation, Codex auth-file formatting,
//! and optional CLIProxyAPI-compatible upload.

mod auth_file;
mod config;
mod cpa;
mod duckmail;
mod error;
mod http;
mod otp;
mod pkce;
mod random;
mod unsupported;

pub use auth_file::{
    AuthFileWriteOutcome, CodexAuthFile, OAuthTokens, decode_jwt_payload, safe_auth_filename,
};
pub use config::{CpaUploadConfig, DuckMailConfig};
pub use cpa::CpaClient;
pub use duckmail::{
    DuckMailAccount, DuckMailApi, DuckMailAttachment, DuckMailDomain, DuckMailMailbox,
    DuckMailMessageDetail, DuckMailMessageSummary, DuckMailToken, MailAddress,
};
pub use error::{CodexAuthSupportError, CodexAuthSupportResult};
pub use otp::extract_verification_code;
pub use pkce::{PkcePair, build_authorize_url, generate_pkce_pair, generate_state};
pub use unsupported::{BlockedCapability, unsupported_capability};
