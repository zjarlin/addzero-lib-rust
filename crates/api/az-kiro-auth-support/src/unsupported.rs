use crate::{KiroAuthSupportError, KiroAuthSupportResult};
use az_derive_aliases::{apply, plain_code_display_no_default_enum};

/// 这个 Rust 迁移版本刻意排除的自动化能力。
#[apply(plain_code_display_no_default_enum)]
pub enum BlockedCapability {
    /// 全自动第三方账号创建。
    #[display("automated_kiro_registration")]
    AutomatedKiroRegistration,
    /// 浏览器指纹冒充或反检测浏览器控制。
    #[display("browser_fingerprint_impersonation")]
    BrowserFingerprintImpersonation,
    /// CAPTCHA、MFA 或 provider 挑战绕过。
    #[display("provider_challenge_bypass")]
    ProviderChallengeBypass,
}

/// 对不支持的自动化能力返回显式错误。
pub fn unsupported_capability(capability: BlockedCapability) -> KiroAuthSupportResult<()> {
    Err(KiroAuthSupportError::UnsupportedCapability { capability })
}
