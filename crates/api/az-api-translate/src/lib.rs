//! 支持多提供商的翻译 API 客户端。
//!
//! 提供通用的 [`TranslateClient`] trait 及配套类型，用于向云端翻译服务
//! 发送翻译请求。内置 MyMemory 免费翻译实现。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_api_translate::{MyMemoryClient, TranslateClient};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = MyMemoryClient::new("user@example.com");
//! let result = client.translate("Hello, world!", "en", "zh-CN").await?;
//! println!("{}", result.translated_text);
//! # Ok(())
//! # }
//! ```

use az_derive_aliases::{apply, error};

automod::dir!("src");

pub use memory::MyMemoryClient;
pub use model::{DetectedLanguage, TranslateOptions, TranslateResult};

/// 翻译请求过程中可能出现的错误。
#[apply(error)]
pub enum TranslateError {
    /// HTTP 请求失败。
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// 响应 JSON 解析失败。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// 翻译服务提供商返回业务错误。
    #[error("provider error: {0}")]
    ProviderError(String),

    /// 当前服务不支持请求的源语言与目标语言组合。
    #[error("unsupported language pair: {from} -> {to}")]
    UnsupportedLanguage { from: String, to: String },

    /// 源文本长度超过服务提供商限制。
    #[error("text too long: {length} chars (max {max})")]
    TextTooLong { length: usize, max: usize },

    /// API key 无效或认证失败。
    #[error("authentication failed: {0}")]
    AuthError(String),

    /// 请求触发服务限流。
    #[error("rate limit exceeded, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
}

/// 翻译接口统一返回类型别名。
pub type TranslateResult_ = Result<TranslateResult, TranslateError>;

/// 所有翻译服务提供商需要实现的统一客户端接口。
///
/// 业务代码应依赖这个 trait 做依赖注入；具体 provider 负责处理认证、限流和服务端
/// 返回格式差异。
#[async_trait::async_trait]
pub trait TranslateClient: Send + Sync {
    /// 将文本从源语言翻译为目标语言。
    ///
    /// 语言代码遵循 ISO 639-1 或 provider 支持的扩展格式，例如 `en`、`zh-CN`、`ja`。
    async fn translate(
        &self,
        text: &str,
        from: &str,
        to: &str,
    ) -> Result<TranslateResult, TranslateError>;

    /// 使用附加选项执行翻译。
    async fn translate_with_options(
        &self,
        text: &str,
        from: &str,
        to: &str,
        _options: &TranslateOptions,
    ) -> Result<TranslateResult, TranslateError> {
        // 默认实现保持最小 provider 契约：不支持选项的客户端仍可只实现 translate。
        self.translate(text, from, to).await
    }

    /// 检测输入文本的语言。
    async fn detect_language(&self, text: &str) -> Result<DetectedLanguage, TranslateError>;

    /// 返回 provider 显式支持的语言对列表。
    fn supported_pairs(&self) -> Vec<(&str, &str)> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_error_display() {
        let err = TranslateError::UnsupportedLanguage {
            from: "xx".into(),
            to: "yy".into(),
        };
        assert!(err.to_string().contains("xx"));
        assert!(err.to_string().contains("yy"));

        let err = TranslateError::TextTooLong {
            length: 5000,
            max: 1000,
        };
        assert!(err.to_string().contains("5000"));

        let err = TranslateError::RateLimited {
            retry_after_secs: 60,
        };
        assert!(err.to_string().contains("60"));
    }

    #[test]
    fn default_translate_with_options() {
        // 验证默认选项入口不改变 provider 的最小 translate 契约。
        struct MockClient;

        #[async_trait::async_trait]
        impl TranslateClient for MockClient {
            async fn translate(
                &self,
                _text: &str,
                _from: &str,
                _to: &str,
            ) -> Result<TranslateResult, TranslateError> {
                Ok(TranslateResult {
                    translated_text: "mocked".into(),
                    source_language: "en".into(),
                    target_language: "zh".into(),
                    confidence: None,
                    alternatives: vec![],
                })
            }

            async fn detect_language(
                &self,
                _text: &str,
            ) -> Result<DetectedLanguage, TranslateError> {
                Ok(DetectedLanguage {
                    language: "en".into(),
                    confidence: 0.95,
                })
            }
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = MockClient;
            let opts = TranslateOptions::default();
            let result = client
                .translate_with_options("hello", "en", "zh", &opts)
                .await
                .unwrap();
            assert_eq!(result.translated_text, "mocked");
        });
    }
}
