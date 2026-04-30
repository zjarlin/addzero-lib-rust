//! Translation data models.

use serde::{Deserialize, Serialize};

/// Options for translation requests.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslateOptions {
    /// Whether to preserve formatting (line breaks, etc.).
    pub preserve_formatting: bool,
    /// Content type hint (e.g., "text/plain", "text/html").
    pub content_type: Option<String>,
    /// Maximum number of alternative translations to return.
    pub max_alternatives: Option<u32>,
}

impl TranslateOptions {
    /// Create default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable formatting preservation.
    pub fn with_preserve_formatting(mut self, preserve: bool) -> Self {
        self.preserve_formatting = preserve;
        self
    }

    /// Set the content type.
    pub fn with_content_type(mut self, ct: impl Into<String>) -> Self {
        self.content_type = Some(ct.into());
        self
    }

    /// Set the maximum number of alternatives.
    pub fn with_max_alternatives(mut self, n: u32) -> Self {
        self.max_alternatives = Some(n);
        self
    }
}

/// Result of a translation request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslateResult {
    /// The translated text.
    pub translated_text: String,
    /// The detected or specified source language.
    pub source_language: String,
    /// The target language.
    pub target_language: String,
    /// Confidence score (0.0–1.0) if available.
    pub confidence: Option<f64>,
    /// Alternative translations, if requested.
    pub alternatives: Vec<String>,
}

/// Result of language detection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectedLanguage {
    /// ISO 639-1 language code.
    pub language: String,
    /// Confidence score (0.0–1.0).
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
