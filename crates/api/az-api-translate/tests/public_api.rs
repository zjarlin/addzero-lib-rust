use anyhow::Result;
use az_api_translate::TranslateClient;
use az_api_translate::model::{DetectedLanguage, TranslateOptions, TranslateResult};

#[test]
fn translate_result_alias_uses_anyhow_error_context() {
    let err = anyhow::anyhow!("unsupported language pair: xx -> yy");

    assert!(err.to_string().contains("xx"));
    assert!(err.to_string().contains("yy"));
}

#[test]
fn default_translate_with_options_preserves_minimal_provider_contract() {
    struct MockClient;

    #[async_trait::async_trait]
    impl TranslateClient for MockClient {
        async fn translate(&self, _text: &str, _from: &str, _to: &str) -> Result<TranslateResult> {
            Ok(TranslateResult {
                translated_text: "mocked".into(),
                source_language: "en".into(),
                target_language: "zh".into(),
                confidence: None,
                alternatives: vec![],
            })
        }

        async fn detect_language(&self, _text: &str) -> Result<DetectedLanguage> {
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
