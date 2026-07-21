# az-ai-chat

统一的 AI/LLM 聊天接口，抽象 OpenAI、Claude、Gemini 等多种云端模型服务。

## 功能

- 定义通用 [`ChatClient`] trait，对不同 AI 提供商统一调用方式
- 内置 OpenAI 兼容接口实现（`OpenAiClient`），可直接对接 OpenAI 及兼容服务
- 结构化的 `Message` 与 `Role` 类型，支持系统提示词、用户消息与助手回复
- `ChatOptions` 配置项：控制温度、最大 token 数等生成参数
- 公开 API 返回 `anyhow::Result`，网络、JSON 和提供商错误直接带上下文返回

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-ai-chat = { path = "../ai-chat" }       # workspace 内部引用
# 或发布后：
# az-ai-chat = "0.1"                           # crates.io 引用
```

## 用法

```rust
use az_ai_chat::{ChatClient, Message};
use az_ai_chat::openai::OpenAiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAiClient::new("https://api.openai.com/v1", "sk-...");
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("Hello!"),
    ];
    let reply = client.chat("gpt-4", &messages, None).await?;
    println!("{}", reply.content);
    Ok(())
}
```

## 依赖的 crates

无外部 workspace 内部 crate 依赖；使用 `reqwest`、`serde`、`tokio` 等通用库。
