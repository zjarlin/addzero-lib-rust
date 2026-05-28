//! 翻译 API 的请求选项与响应数据模型。

use az_derive_aliases::{apply, serde_eq_default, serde_partial_eq};

/// 翻译请求的附加选项。
#[apply(serde_eq_default)]
pub struct TranslateOptions {
    /// 是否尽量保留换行等原文格式。
    pub preserve_formatting: bool,
    /// 内容类型提示，例如 `text/plain` 或 `text/html`。
    pub content_type: Option<String>,
    /// 最多返回多少条候选译文。
    pub max_alternatives: Option<u32>,
}

impl TranslateOptions {
    /// 创建默认翻译选项。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否保留原文格式。
    pub fn with_preserve_formatting(mut self, preserve: bool) -> Self {
        self.preserve_formatting = preserve;
        self
    }

    /// 设置内容类型提示。
    pub fn with_content_type(mut self, ct: impl Into<String>) -> Self {
        self.content_type = Some(ct.into());
        self
    }

    /// 设置候选译文数量上限。
    pub fn with_max_alternatives(mut self, n: u32) -> Self {
        self.max_alternatives = Some(n);
        self
    }
}

/// 单次翻译请求的结果。
#[apply(serde_partial_eq)]
pub struct TranslateResult {
    /// 主译文。
    pub translated_text: String,
    /// provider 实际使用或检测到的源语言。
    pub source_language: String,
    /// 目标语言。
    pub target_language: String,
    /// provider 返回的置信度，范围通常为 `0.0..=1.0`。
    pub confidence: Option<f64>,
    /// 请求候选译文时返回的备选结果。
    pub alternatives: Vec<String>,
}

/// 语言检测结果。
#[apply(serde_partial_eq)]
pub struct DetectedLanguage {
    /// ISO 639-1 语言代码，或 provider 使用的兼容代码。
    pub language: String,
    /// 检测置信度，范围通常为 `0.0..=1.0`。
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_options_builder() {
        let opts = TranslateOptions::new()
            .with_preserve_formatting(true)
            .with_content_type("text/html")
            .with_max_alternatives(3);

        assert!(opts.preserve_formatting);
        assert_eq!(opts.content_type.as_deref(), Some("text/html"));
        assert_eq!(opts.max_alternatives, Some(3));
    }

    #[test]
    fn translate_result_serialization_roundtrip() {
        let result = TranslateResult {
            translated_text: "你好世界".into(),
            source_language: "en".into(),
            target_language: "zh-CN".into(),
            confidence: Some(0.98),
            alternatives: vec!["您好世界".into()],
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TranslateResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.translated_text, deserialized.translated_text);
        assert_eq!(result.source_language, deserialized.source_language);
        assert_eq!(result.target_language, deserialized.target_language);
        assert!(result.confidence.is_some());
        assert!(
            (result.confidence.unwrap() - deserialized.confidence.unwrap()).abs() < f64::EPSILON
        );
    }

    #[test]
    fn detected_language_fields() {
        let dl = DetectedLanguage {
            language: "en".into(),
            confidence: 0.95,
        };
        assert_eq!(dl.language, "en");
        assert!((dl.confidence - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn default_options() {
        let opts = TranslateOptions::default();
        assert!(!opts.preserve_formatting);
        assert!(opts.content_type.is_none());
        assert!(opts.max_alternatives.is_none());
    }
}
