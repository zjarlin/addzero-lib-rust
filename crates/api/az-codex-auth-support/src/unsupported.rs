use crate::{CodexAuthSupportError, CodexAuthSupportResult};

/// Automation capabilities intentionally left out of this Rust port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockedCapability {
    /// Bulk creation of OpenAI or ChatGPT accounts.
    AutomatedOpenAiRegistration,
    /// Reimplementation of Sentinel proof-of-work or anti-abuse challenge generation.
    SentinelProofOfWork,
    /// Browser fingerprint impersonation or TLS/client-profile spoofing.
    BrowserFingerprintImpersonation,
    /// Bulk OAuth token generation against third-party accounts.
    BulkTokenGeneration,
}

impl BlockedCapability {
    /// Stable machine-readable reason label for this blocked capability.
    pub fn label(self) -> &'static str {
        match self {
            Self::AutomatedOpenAiRegistration => "automated_openai_registration",
            Self::SentinelProofOfWork => "sentinel_proof_of_work",
            Self::BrowserFingerprintImpersonation => "browser_fingerprint_impersonation",
            Self::BulkTokenGeneration => "bulk_token_generation",
        }
    }
}

/// Returns an explicit error for unsupported automation capabilities.
pub fn unsupported_capability(capability: BlockedCapability) -> CodexAuthSupportResult<()> {
    Err(CodexAuthSupportError::UnsupportedCapability(
        capability.label(),
    ))
}
