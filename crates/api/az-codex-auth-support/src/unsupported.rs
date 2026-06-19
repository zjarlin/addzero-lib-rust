
/// 这个 Rust 迁移版本刻意排除的自动化能力。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, derive_more::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
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

impl BlockedCapability {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// 对不支持的自动化能力返回显式错误。
pub fn unsupported_capability(capability: BlockedCapability) -> anyhow::Result<()> {
    anyhow::bail!("unsupported capability: {capability}")
}

#[cfg(test)]
mod tests {
    use super::{BlockedCapability, unsupported_capability};

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
