//! 支持多提供商的翻译 API 客户端。
//!
//! 提供通用的 [`TranslateClient`] trait 及配套类型，用于向云端翻译服务
//! 发送翻译请求。内置 MyMemory 免费翻译实现。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_api_translate::memory::MyMemoryClient;
//! use az_api_translate::TranslateClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = MyMemoryClient::new("user@example.com");
//! let result = client.translate("Hello, world!", "en", "zh-CN").await?;
//! println!("{}", result.translated_text);
//! # Ok(())
//! # }
//! ```

use anyhow::Result;

automod::dir!(pub "src");

use model::{DetectedLanguage, TranslateOptions, TranslateResult};

/// 所有翻译服务提供商需要实现的统一客户端接口。
///
/// 业务代码应依赖这个 trait 做依赖注入；具体 provider 负责处理认证、限流和服务端
/// 返回格式差异。
#[async_trait::async_trait]
pub trait TranslateClient: Send + Sync {
    /// 将文本从源语言翻译为目标语言。
    ///
    /// 语言代码遵循 ISO 639-1 或 provider 支持的扩展格式，例如 `en`、`zh-CN`、`ja`。
    async fn translate(&self, text: &str, from: &str, to: &str) -> Result<TranslateResult>;

    /// 使用附加选项执行翻译。
    async fn translate_with_options(
        &self,
        text: &str,
        from: &str,
        to: &str,
        _options: &TranslateOptions,
    ) -> Result<TranslateResult> {
        // 默认实现保持最小 provider 契约：不支持选项的客户端仍可只实现 translate。
        self.translate(text, from, to).await
    }

    /// 检测输入文本的语言。
    async fn detect_language(&self, text: &str) -> Result<DetectedLanguage>;

    /// 返回 provider 显式支持的语言对列表。
    fn supported_pairs(&self) -> Vec<(&str, &str)> {
        Vec::new()
    }
}
