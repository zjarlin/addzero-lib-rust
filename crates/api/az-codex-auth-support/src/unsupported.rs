use crate::{CodexAuthSupportError, CodexAuthSupportResult};
use az_derive_aliases::{apply, plain_copy_eq_display};

/// Automation capabilities intentionally left out of this Rust port.
#[apply(plain_copy_eq_display)]
pub enum BlockedCapability {
    /// Bulk creation of OpenAI or ChatGPT accounts.
    #[display("automated_openai_registration")]
    AutomatedOpenAiRegistration,
    /// Reimplementation of Sentinel proof-of-work or anti-abuse challenge generation.
    #[display("sentinel_proof_of_work")]
    SentinelProofOfWork,
    /// Browser fingerprint impersonation or TLS/client-profile spoofing.
    #[display("browser_fingerprint_impersonation")]
    BrowserFingerprintImpersonation,
    /// Bulk OAuth token generation against third-party accounts.
    #[display("bulk_token_generation")]
    BulkTokenGeneration,
}

/// Returns an explicit error for unsupported automation capabilities.
pub fn unsupported_capability(capability: BlockedCapability) -> CodexAuthSupportResult<()> {
    Err(CodexAuthSupportError::UnsupportedCapability { capability })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_capabilities_display_machine_labels() {
        assert_eq!(
            BlockedCapability::AutomatedOpenAiRegistration.to_string(),
            "automated_openai_registration"
        );
        let error = unsupported_capability(BlockedCapability::AutomatedOpenAiRegistration)
            .expect_err("capability should be blocked")
            .to_string();
        assert_eq!(
            error,
            "unsupported capability: automated_openai_registration"
        );
    }
}
