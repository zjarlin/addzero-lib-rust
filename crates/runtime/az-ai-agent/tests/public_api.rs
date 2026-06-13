use az_ai_agent::{AssetAgentService, rig_provider_markers};
use az_assets::AssetKind;

#[test]
fn capture_asset_should_generate_title_tags_and_edges() {
    let service = AssetAgentService::new();
    let output = service
        .summarize_asset("Rust skill 要同步到知识图谱", AssetKind::Note)
        .unwrap();
    assert_eq!(output.title, "Rust skill 要同步到知识图谱");
    assert!(output.tags.contains(&"Rust".to_string()));
    assert!(!output.suggested_edges.is_empty());
}

#[test]
fn rig_markers_should_include_three_provider_clients() {
    let markers = rig_provider_markers();
    assert!(markers.iter().any(|marker| marker.contains("openai")));
    assert!(markers.iter().any(|marker| marker.contains("anthropic")));
    assert!(markers.iter().any(|marker| marker.contains("gemini")));
}
