//! OpenAI-compatible chat client implementation.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{ChatClient, ChatError, ChatOptions, ChatResponse, ChatResult, Message, Role, Usage};

/// OpenAI-compatible chat completion client.
///
/// Works with any API that follows the OpenAI `/chat/completions` format,
/// including OpenAI, Azure OpenAI, local LLMs (Ollama, vLLM), etc.
pub struct OpenAiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl OpenAiClient {
    /// Create a new client with the given base URL and API key.
    ///
    /// The `base_url` should be like `"https://api.openai.com/v1"` (no trailing slash).
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            api_key: api_key.into(),
        }
    }

    /// Create a new client with a custom `reqwest::Client`.
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

#[derive(Serialize)]
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

#[derive(Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    model: Option<String>,
    choices: Option<Vec<OpenAiChoice>>,
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: Option<OpenAiChoiceMessage>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiChoiceMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
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
    ) -> ChatResult<ChatResponse> {
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
            return Err(ChatError::ProviderError {
                code: status.as_u16(),
                message: body,
            });
        }

        let raw: OpenAiResponse = resp.json().await?;

        let choice = raw
            .choices
            .and_then(|mut c| c.pop())
            .ok_or_else(|| ChatError::MissingField("choices".into()))?;

        let content = choice
            .message
            .and_then(|m| m.content)
            .ok_or_else(|| ChatError::MissingField("message.content".into()))?;

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
