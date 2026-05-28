use crate::{CodexAuthSupportError, CodexAuthSupportResult};
use az_derive_aliases::{apply, plain_code_display_no_default_enum};

/// 这个 Rust 迁移版本刻意排除的自动化能力。
#[apply(plain_code_display_no_default_enum)]
pub enum BlockedCapability {
    /// 批量创建 OpenAI 或 ChatGPT 账号。
    #[display("automated_openai_registration")]
    AutomatedOpenAiRegistration,
    /// 重新实现 Sentinel 工作量证明或反滥用挑战生成。
    #[display("sentinel_proof_of_work")]
    SentinelProofOfWork,
    /// 浏览器指纹冒充或 TLS/client-profile 伪造。
    #[display("browser_fingerprint_impersonation")]
    BrowserFingerprintImpersonation,
    /// 面向第三方账号批量生成 OAuth token。
    #[display("bulk_token_generation")]
    BulkTokenGeneration,
}

/// 对不支持的自动化能力返回显式错误。
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
