use crate::{KiroAuthSupportError, KiroAuthSupportResult};

/// Automation capabilities intentionally left out of this Rust port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockedCapability {
    /// Fully automated third-party account creation.
    AutomatedKiroRegistration,
    /// Browser fingerprint impersonation or anti-detection browser control.
    BrowserFingerprintImpersonation,
    /// CAPTCHA, MFA, or provider challenge bypass.
    ProviderChallengeBypass,
}

impl BlockedCapability {
    /// Stable machine-readable reason label for this blocked capability.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::AutomatedKiroRegistration => "automated_kiro_registration",
            Self::BrowserFingerprintImpersonation => "browser_fingerprint_impersonation",
            Self::ProviderChallengeBypass => "provider_challenge_bypass",
        }
    }
}

/// Returns an explicit error for unsupported automation capabilities.
pub fn unsupported_capability(capability: BlockedCapability) -> KiroAuthSupportResult<()> {
    Err(KiroAuthSupportError::UnsupportedCapability(
        capability.label(),
    ))
}
