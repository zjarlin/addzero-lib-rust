//! OpenAI 兼容聊天客户端实现。

use az_derive_aliases::{apply, deserialize_debug, serialize_debug};
use anyhow::{Context, Result, bail};
use reqwest::Client;

use crate::{ChatClient, ChatOptions, ChatResponse, Message, Role, Usage};

/// OpenAI 兼容的聊天补全客户端。
///
/// 只要服务端遵循 OpenAI `/chat/completions` 格式即可使用，
/// 包括 OpenAI、Azure OpenAI、本地 LLM 服务（Ollama、vLLM）等。
pub struct OpenAiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl OpenAiClient {
    /// 使用 base URL 和 API key 创建客户端。
    ///
    /// `base_url` 应类似 `"https://api.openai.com/v1"`，尾部是否带斜杠都会被规范化。
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            api_key: api_key.into(),
        }
    }

    /// 使用自定义 `reqwest::Client` 创建客户端。
    pub fn with_client(
        client: Client,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            client,
            base_url: base_url.into(),
            api_key: api_key.into(),
        }
    }

    fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
    }
}

#[apply(serialize_debug)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<&'a Vec<String>>,
}

#[apply(serialize_debug)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[apply(deserialize_debug)]
struct OpenAiResponse {
    model: Option<String>,
    choices: Option<Vec<OpenAiChoice>>,
    usage: Option<OpenAiUsage>,
}

#[apply(deserialize_debug)]
struct OpenAiChoice {
    message: Option<OpenAiChoiceMessage>,
    finish_reason: Option<String>,
}

#[apply(deserialize_debug)]
struct OpenAiChoiceMessage {
    content: Option<String>,
}

#[apply(deserialize_debug)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

fn role_str(role: Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
    }
}

#[async_trait::async_trait]
impl ChatClient for OpenAiClient {
    async fn chat(
        &self,
        model: &str,
        messages: &[Message],
        options: Option<&ChatOptions>,
    ) -> Result<ChatResponse> {
        let opts = options.cloned().unwrap_or_default();

        let api_messages: Vec<OpenAiMessage<'_>> = messages
            .iter()
            .map(|m| OpenAiMessage {
                role: role_str(m.role),
                content: &m.content,
            })
            .collect();

        let request = OpenAiRequest {
            model,
            messages: api_messages,
            temperature: opts.temperature,
            max_tokens: opts.max_tokens,
            top_p: opts.top_p,
            stop: opts.stop.as_ref(),
        };

        let resp = self
            .client
            .post(self.endpoint())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("provider error ({}): {body}", status.as_u16());
        }

        let raw: OpenAiResponse = resp.json().await?;

        let choice = raw
            .choices
            .and_then(|mut c| c.pop())
            .context("missing field in response: choices")?;

        let content = choice
            .message
            .and_then(|m| m.content)
            .context("missing field in response: message.content")?;

        Ok(ChatResponse {
            content,
            model: raw.model.unwrap_or_else(|| model.to_string()),
            usage: raw.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
            finish_reason: choice.finish_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_client_endpoint() {
        let client = OpenAiClient::new("https://api.openai.com/v1", "sk-test");
        assert_eq!(
            client.endpoint(),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn openai_client_endpoint_trailing_slash() {
        let client = OpenAiClient::new("https://api.openai.com/v1/", "sk-test");
        assert_eq!(
            client.endpoint(),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn openai_message_serialization() {
        let msg = OpenAiMessage {
            role: "user",
            content: "hello",
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"hello\""));
    }

    #[test]
    fn openai_request_skips_none_fields() {
        let request = OpenAiRequest {
            model: "gpt-4",
            messages: vec![],
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(!json.contains("temperature"));
        assert!(!json.contains("max_tokens"));
    }

    #[test]
    fn openai_response_parsing() {
        let json = r#"{
            "model": "gpt-4",
            "choices": [{"message": {"content": "Hi!"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 5, "completion_tokens": 3, "total_tokens": 8}
        }"#;
        let resp: OpenAiResponse = serde_json::from_str(json).unwrap();
        let choice = resp.choices.unwrap().into_iter().next().unwrap();
        assert_eq!(choice.message.unwrap().content.unwrap(), "Hi!");
        assert_eq!(resp.usage.unwrap().total_tokens, 8);
    }
}
