#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

automod::dir!("src");

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
