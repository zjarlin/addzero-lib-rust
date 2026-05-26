use crate::{CodexAuthSupportError, CodexAuthSupportResult};
use az_derive_aliases::{apply, plain_code_display_no_default_enum};

/// Automation capabilities intentionally left out of this Rust port.
#[apply(plain_code_display_no_default_enum)]
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

    #[test]
    fn unsupported_capabilities_expose_machine_codes() {
        assert_eq!(
            BlockedCapability::AutomatedOpenAiRegistration.code(),
            "automated_open_ai_registration"
        );
        assert_eq!(
            BlockedCapability::from_code("bulk_token_generation"),
            Some(BlockedCapability::BulkTokenGeneration)
        );
    }
}
