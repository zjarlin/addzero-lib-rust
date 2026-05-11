#![forbid(unsafe_code)]

//! Safe Rust support code extracted from the Python `KiroRegister` project.
//!
//! This crate provides auditable building blocks: Kiro/AWS Builder ID device
//! flow requests, polling state, verification-code parsing, and local test data
//! generation. It intentionally does not port Camoufox-style browser
//! fingerprint impersonation or fully automated third-party account creation.

mod config;
mod device_flow;
mod error;
mod http;
mod identity;
mod otp;
mod unsupported;

pub use config::{KiroOidcConfig, KiroOidcConfigBuilder};
pub use device_flow::{
    KiroClientRegistration, KiroDeviceAuthorization, KiroDeviceFlow, KiroDeviceFlowClient,
    KiroDeviceFlowManager, KiroDeviceFlowSession, KiroDeviceFlowSessionSnapshot,
    KiroDeviceFlowSessionStatus, KiroLoginType, KiroTokenPoll, KiroTokenResponse,
};
pub use error::{KiroAuthSupportError, KiroAuthSupportResult};
pub use identity::{
    EnglishName, EnglishNameOptions, NameGender, PasswordPolicy, generate_english_name,
    generate_password,
};
pub use otp::extract_verification_code;
pub use unsupported::{BlockedCapability, unsupported_capability};
