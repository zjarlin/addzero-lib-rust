use crate::{KiroAuthSupportError, KiroAuthSupportResult};
use az_derive_aliases::{apply, plain_copy_eq_display};

/// Automation capabilities intentionally left out of this Rust port.
#[apply(plain_copy_eq_display)]
pub enum BlockedCapability {
    /// Fully automated third-party account creation.
    #[display("automated_kiro_registration")]
    AutomatedKiroRegistration,
    /// Browser fingerprint impersonation or anti-detection browser control.
    #[display("browser_fingerprint_impersonation")]
    BrowserFingerprintImpersonation,
    /// CAPTCHA, MFA, or provider challenge bypass.
    #[display("provider_challenge_bypass")]
    ProviderChallengeBypass,
}

/// Returns an explicit error for unsupported automation capabilities.
pub fn unsupported_capability(capability: BlockedCapability) -> KiroAuthSupportResult<()> {
    Err(KiroAuthSupportError::UnsupportedCapability { capability })
}
