//! 资产智能代理服务，负责内容采集、摘要生成与知识图谱边推断。
//!
//! 本 crate 封装了 [`AssetAgentService`]，为上层应用提供统一的
//! AI 内容处理接口：接收原始文本，自动推断标题、标签和关联边，
//! 输出结构化的 [`PromptRunOutput`]。
//!
//! ## 核心能力
//!
//! - `capture_asset` — 采集内容并结合提示模板生成结构化输出
//! - `summarize_asset` — 对原始文本进行本地摘要推断
//! - `extract_graph_edges` — 从文本中提取知识图谱关联边
//! - `run_prompt_button` — 执行自定义提示按钮的运行
//! - `default_model_for` — 根据 AI 提供商返回推荐默认模型名
//!
//! ## 设计约束
//!
//! 当前版本使用本地规则推断（标题提取、关键词匹配、边推断），
//! 预留了 `run_with_provider_secret` 接口供后续接入远程 LLM。

use anyhow::Result;
use az_assets::{
    AiModelProvider, AiPromptButton, AiProviderKind, AssetKind, AssetProviderSecret,
    PromptRunOutput, SuggestedEdge,
};
use az_derive_aliases::{apply, error_eq, plain_default_clone, serde_eq};

/// 采集内容并生成资产候选输出的请求。
#[apply(serde_eq)]
pub struct CaptureAssetRequest {
    /// 待处理的原始内容。
    pub raw_content: String,
    /// 希望生成的资产类型。
    pub target_kind: AssetKind,
    /// 可选 prompt 按钮配置。
    pub prompt: Option<AiPromptButton>,
    /// 可选 AI provider 配置，当前本地规则实现保留该字段供远程模型接入。
    pub provider: Option<AiModelProvider>,
}

/// 运行单个 prompt 按钮的请求。
#[apply(serde_eq)]
pub struct PromptButtonRun {
    /// prompt 按钮配置。
    pub prompt: AiPromptButton,
    /// 待处理的原始内容。
    pub raw_content: String,
}

/// 资产智能代理服务错误。
#[apply(error_eq)]
pub enum AssetAgentError {
    #[error("采集内容不能为空")]
    EmptyInput,
}

/// 资产智能代理服务。
///
/// 当前实现使用本地规则生成标题、标签和候选边；远程 LLM 接入点保留在
/// [`Self::run_with_provider_secret`]。
#[apply(plain_default_clone)]
pub struct AssetAgentService;

impl AssetAgentService {
    /// 创建资产智能代理服务。
    pub fn new() -> Self {
        Self
    }

    /// 返回编译进来的 `rig` provider 类型标记。
    pub fn rig_provider_markers(&self) -> [&'static str; 3] {
        rig_provider_markers()
    }

    /// 根据采集请求生成结构化资产输出。
    pub fn capture_asset(&self, input: CaptureAssetRequest) -> Result<PromptRunOutput> {
        let prompt = input.prompt.as_ref();
        self.run_local_summary(&input.raw_content, input.target_kind, prompt)
    }

    /// 对原始内容进行本地摘要和标签推断。
    pub fn summarize_asset(
        &self,
        raw_content: &str,
        target_kind: AssetKind,
    ) -> Result<PromptRunOutput> {
        self.run_local_summary(raw_content, target_kind, None)
    }

    /// 从原始内容中提取候选图谱边。
    pub fn extract_graph_edges(&self, raw_content: &str) -> Result<Vec<SuggestedEdge>> {
        let output = self.run_local_summary(raw_content, AssetKind::Note, None)?;
        Ok(output.suggested_edges)
    }

    /// 按 prompt 按钮配置运行内容处理。
    pub fn run_prompt_button(&self, input: PromptButtonRun) -> Result<PromptRunOutput> {
        self.run_local_summary(
            &input.raw_content,
            input.prompt.target_kind,
            Some(&input.prompt),
        )
    }

    /// 使用 provider 凭据运行内容处理。
    ///
    /// 这是远程 LLM 的依赖注入边界；当前实现仍调用本地规则，避免在无密钥环境下引入网络副作用。
    pub fn run_with_provider_secret(
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

/// 返回 `rig` 内置 provider 客户端类型名，作为依赖可用性的轻量探针。
pub fn rig_provider_markers() -> [&'static str; 3] {
    [
        std::any::type_name::<rig::providers::openai::Client>(),
        std::any::type_name::<rig::providers::anthropic::Client>(),
        std::any::type_name::<rig::providers::gemini::Client>(),
    ]
}

/// 返回每个 AI provider 的默认模型名。
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
