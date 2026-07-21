# az-ai-agent

资产智能代理服务，负责内容采集、摘要生成与知识图谱边推断。

## 功能

- **内容采集** — `capture_asset` 接收原始文本，结合提示模板生成结构化输出
- **摘要推断** — `summarize_asset` 自动推断标题、标签和正文内容
- **图谱边提取** — `extract_graph_edges` 从文本关键词中推断知识图谱关联边
- **提示按钮执行** — `run_prompt_button` 运行用户自定义的提示按钮
- **模型推荐** — `default_model_for` 按 AI 提供商返回默认模型名称（OpenAI / Anthropic / Gemini）
- **提供商标记** — `rig_provider_markers` 返回 rig-core 集成的三大 LLM 提供商类型标识

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-ai-agent = { path = "../ai-agent" }       # workspace 内部引用
# 或发布后：
# az-ai-agent = "0.1"                            # crates.io 引用
```

## 用法

```rust
use az_ai_agent::api::AssetAgentService;
use az_assets::types::AssetKind;

let service = AssetAgentService::new();

// 采集并生成摘要
let output = service
    .summarize_asset("这是一段关于 Rust 技能同步到知识图谱的内容", AssetKind::Note)
    .unwrap();

println!("标题: {}", output.title);
println!("标签: {:?}", output.tags);
println!("关联边: {:?}", output.suggested_edges);
```

## 依赖的 crates

- `az-assets` — 资产类型定义（AssetKind、PromptRunOutput 等）
- `rig-core` — LLM 提供商集成（OpenAI / Anthropic / Gemini）
- `serde` / `serde_json` — 序列化
- `anyhow` — 错误返回与上下文
