use az_api_translate::{
    DetectedLanguage, TranslateClient, TranslateError, TranslateOptions, TranslateResult,
};

#[test]
fn translate_error_display_includes_variant_context() {
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
fn default_translate_with_options_preserves_minimal_provider_contract() {
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

        async fn detect_language(&self, _text: &str) -> Result<DetectedLanguage, TranslateError> {
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
