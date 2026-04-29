//! Translation API client supporting multiple providers.
//!
//! Provides a common [`TranslateClient`] trait and supporting types to send
//! translation requests to cloud translation services. Ships with a built-in
//! MyMemory free translation implementation.
//!
//! # Quick Start
//!
//! ```no_run
//! use addzero_api_translate::{MyMemoryClient, TranslateClient};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = MyMemoryClient::new("user@example.com");
//! let result = client.translate("Hello, world!", "en", "zh-CN").await?;
//! println!("{}", result.translated_text);
//! # Ok(())
//! # }
//! ```

use thiserror::Error;

mod memory;
mod model;

pub use memory::MyMemoryClient;
pub use model::{DetectedLanguage, TranslateOptions, TranslateResult};

/// Errors that can occur during translation.
#[derive(Debug, Error)]
pub enum TranslateError {
    /// HTTP request failed.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// The provider returned an error.
    #[error("provider error: {0}")]
    ProviderError(String),

    /// The specified language pair is not supported.
    #[error("unsupported language pair: {from} -> {to}")]
    UnsupportedLanguage { from: String, to: String },

    /// The source text is too long for the provider.
    #[error("text too long: {length} chars (max {max})")]
    TextTooLong { length: usize, max: usize },

    /// Invalid API key or authentication failure.
    #[error("authentication failed: {0}")]
    AuthError(String),

    /// Rate limit exceeded.
    #[error("rate limit exceeded, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
}

/// Result alias for translation operations.
pub type TranslateResult_ = Result<TranslateResult, TranslateError>;

/// Trait implemented by all translation providers.
#[async_trait::async_trait]
pub trait TranslateClient: Send + Sync {
    /// Translate text from source language to target language.
    ///
    /// Language codes follow ISO 639-1 (e.g., "en", "zh-CN", "ja").
    async fn translate(
        &self,
        text: &str,
        from: &str,
        to: &str,
    ) -> Result<TranslateResult, TranslateError>;

    /// Translate text with additional options.
    async fn translate_with_options(
        &self,
        text: &str,
        from: &str,
        to: &str,
        _options: &TranslateOptions,
    ) -> Result<TranslateResult, TranslateError> {
        // Default implementation ignores options
        self.translate(text, from, to).await
    }

    /// Detect the language of the given text.
    async fn detect_language(&self, text: &str) -> Result<DetectedLanguage, TranslateError>;

    /// Get the list of supported language pairs.
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
        // Verify the default implementation delegates to translate
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
