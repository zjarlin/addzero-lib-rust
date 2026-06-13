#![forbid(unsafe_code)]

//! 从 Python `codex_auto_register` 项目提取的安全 Rust 支持代码。
//!
//! 此 crate 刻意不实现自动化 OpenAI 账户注册、Sentinel 工作量证明生成、
//! 浏览器指纹伪造或基于代理的风险控制绕过流程。它提供可复用且可审计的组件，
//! 包括 DuckMail 收件箱访问、OTP 解析、PKCE 生成、Codex 认证文件格式化，
//! 以及可选的兼容 CLIProxyAPI 上传。

automod::dir!("src");

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
