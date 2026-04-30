use addzero_assets::{
    AiModelProvider, AiPromptButton, AiProviderKind, AssetKind, AssetProviderSecret,
    PromptRunOutput, SuggestedEdge,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CaptureAssetRequest {
    pub raw_content: String,
    pub target_kind: AssetKind,
    pub prompt: Option<AiPromptButton>,
    pub provider: Option<AiModelProvider>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptButtonRun {
    pub prompt: AiPromptButton,
    pub raw_content: String,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AssetAgentError {
    #[error("采集内容不能为空")]
    EmptyInput,
}

#[derive(Clone, Default)]
pub struct AssetAgentService;

impl AssetAgentService {
    pub fn new() -> Self {
        Self
    }

    pub fn rig_provider_markers(&self) -> [&'static str; 3] {
        rig_provider_markers()
    }

    pub async fn capture_asset(&self, input: CaptureAssetRequest) -> Result<PromptRunOutput> {
        let prompt = input.prompt.as_ref();
        self.run_local_summary(&input.raw_content, input.target_kind, prompt)
    }

    pub async fn summarize_asset(
        &self,
        raw_content: &str,
        target_kind: AssetKind,
    ) -> Result<PromptRunOutput> {
        self.run_local_summary(raw_content, target_kind, None)
    }

    pub async fn extract_graph_edges(&self, raw_content: &str) -> Result<Vec<SuggestedEdge>> {
        let output = self.run_local_summary(raw_content, AssetKind::Note, None)?;
        Ok(output.suggested_edges)
    }

    pub async fn run_prompt_button(&self, input: PromptButtonRun) -> Result<PromptRunOutput> {
        self.run_local_summary(
            &input.raw_content,
            input.prompt.target_kind,
            Some(&input.prompt),
        )
    }

    pub async fn run_with_provider_secret(
        &self,
        raw_content: &str,
        target_kind: AssetKind,
        prompt: Option<&AiPromptButton>,
        _secret: Option<AssetProviderSecret>,
    ) -> Result<PromptRunOutput> {
        self.run_local_summary(raw_content, target_kind, prompt)
    }

    fn run_local_summary(
        &self,
        raw_content: &str,
        target_kind: AssetKind,
        prompt: Option<&AiPromptButton>,
    ) -> Result<PromptRunOutput> {
        let cleaned = raw_content.trim();
        if cleaned.is_empty() {
            return Err(AssetAgentError::EmptyInput.into());
        }
        let title = infer_title(cleaned, target_kind);
        let mut tags = infer_tags(cleaned, target_kind);
        if let Some(prompt) = prompt {
            tags.push(prompt.label.clone());
        }
        tags.sort();
        tags.dedup();
        Ok(PromptRunOutput {
            title,
            tags,
            body: normalize_body(cleaned),
            suggested_edges: infer_edges(cleaned),
        })
    }
}

pub fn rig_provider_markers() -> [&'static str; 3] {
    [
        std::any::type_name::<rig::providers::openai::Client>(),
        std::any::type_name::<rig::providers::anthropic::Client>(),
        std::any::type_name::<rig::providers::gemini::Client>(),
    ]
}

pub fn default_model_for(provider: AiProviderKind) -> &'static str {
    match provider {
        AiProviderKind::OpenAi => "gpt-4.1-mini",
        AiProviderKind::Anthropic => "claude-sonnet-4-5",
        AiProviderKind::Gemini => "gemini-2.5-flash",
    }
}

fn infer_title(content: &str, kind: AssetKind) -> String {
    let first_line = content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(content);
    let title = first_line
        .trim_start_matches(['#', '-', '*', ' '])
        .chars()
        .take(36)
        .collect::<String>();
    if title.is_empty() {
        match kind {
            AssetKind::Skill => "未命名 Skill".to_string(),
            AssetKind::Capture => "未命名采集".to_string(),
            _ => "未命名笔记".to_string(),
        }
    } else {
        title
    }
}

fn infer_tags(content: &str, kind: AssetKind) -> Vec<String> {
    let lower = content.to_lowercase();
    let mut tags = vec![match kind {
        AssetKind::Capture => "采集".to_string(),
        AssetKind::Note => "笔记".to_string(),
        AssetKind::Skill => "Skill".to_string(),
        AssetKind::Software => "软件".to_string(),
        AssetKind::Package => "安装包".to_string(),
    }];
    for (needle, tag) in [
        ("rust", "Rust"),
        ("postgres", "Postgres"),
        ("pg", "Postgres"),
        ("skill", "Skill"),
        ("agent", "Agent"),
        ("图谱", "图谱"),
        ("模型", "模型"),
        ("同步", "同步"),
    ] {
        if lower.contains(needle) || content.contains(needle) {
            tags.push(tag.to_string());
        }
    }
    tags
}

fn normalize_body(content: &str) -> String {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn infer_edges(content: &str) -> Vec<SuggestedEdge> {
    let mut edges = Vec::new();
    for (needle, target, relation) in [
        ("skill", "Skills 资产", "relates_to"),
        ("图谱", "知识图谱", "relates_to"),
        ("同步", "Agent 探针同步", "needs_sync"),
        ("postgres", "PostgreSQL", "stored_in"),
        ("pg", "PostgreSQL", "stored_in"),
    ] {
        if content.to_lowercase().contains(needle) || content.contains(needle) {
            edges.push(SuggestedEdge {
                target_title: target.to_string(),
                relation: relation.to_string(),
                confidence: 80,
            });
        }
    }
    edges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn capture_asset_should_generate_title_tags_and_edges() {
        let service = AssetAgentService::new();
        let output = service
            .summarize_asset("Rust skill 要同步到知识图谱", AssetKind::Note)
            .await
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
}
