use addzero_ai_agent::{AssetAgentService, default_model_for};
use addzero_assets::{AiProviderKind, AssetKind};

#[tokio::test]
async fn public_api_should_summarize_without_external_key() {
    let output = AssetAgentService::new()
        .summarize_asset(
            "用户不断发送采集内容，系统归纳成笔记图谱。",
            AssetKind::Note,
        )
        .await
        .unwrap();
    assert!(output.tags.contains(&"笔记".to_string()));
    assert!(!output.title.is_empty());
}

#[test]
fn public_api_should_name_default_models() {
    assert!(default_model_for(AiProviderKind::OpenAi).contains("gpt"));
    assert!(default_model_for(AiProviderKind::Anthropic).contains("claude"));
    assert!(default_model_for(AiProviderKind::Gemini).contains("gemini"));
}
