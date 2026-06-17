use std::env;

use anyhow::{Context, bail};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
};

/// Sends a single chat completion request with an optional system prompt.
pub async fn chat_completions(
    model: &str,
    system: Option<&str>,
    prompt: &str,
) -> anyhow::Result<String> {
    let config = OpenAiRuntimeConfig::from_env()?;
    let client = Client::with_config(
        OpenAIConfig::new()
            .with_api_key(config.api_key)
            .with_api_base(config.api_base),
    );

    let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();
    if let Some(system) = system.filter(|value| !value.trim().is_empty()) {
        messages.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system)
                .build()?
                .into(),
        );
    }
    messages.push(
        ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into(),
    );

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages)
        .max_completion_tokens(2048u32)
        .build()?;

    let response = client.chat().create(request).await?;
    tracing::info!("chat completion response received");
    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

    Ok(content)
}

struct OpenAiRuntimeConfig {
    api_key: String,
    api_base: String,
}

impl OpenAiRuntimeConfig {
    fn from_env() -> anyhow::Result<Self> {
        let api_key = first_env(["OPENAI_API_KEY", "API_KEY"])
            .context("missing OPENAI_API_KEY or API_KEY for az-agent")?;
        if api_key.trim().is_empty() {
            bail!("OPENAI_API_KEY/API_KEY is empty");
        }
        let api_base = first_env(["OPENAI_BASE_URL", "OPENAI_BASEURL", "API_BASEURL"])
            .map(|api_base| normalize_openai_api_base(&api_base))
            .transpose()?
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        Ok(Self { api_key, api_base })
    }
}

fn normalize_openai_api_base(api_base: &str) -> anyhow::Result<String> {
    let trimmed = api_base.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        bail!("OpenAI API base URL is empty");
    }
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        bail!("OpenAI API base URL must start with http:// or https://");
    }
    if trimmed.ends_with("/v1") {
        Ok(trimmed.to_string())
    } else {
        Ok(format!("{trimmed}/v1"))
    }
}

fn first_env<const N: usize>(names: [&str; N]) -> Option<String> {
    names
        .into_iter()
        .find_map(|name| env::var(name).ok())
        .filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::normalize_openai_api_base;

    #[test]
    fn normalizes_gateway_root_to_openai_v1_base() {
        assert_eq!(
            normalize_openai_api_base("https://api.addzero.site").unwrap(),
            "https://api.addzero.site/v1"
        );
    }

    #[test]
    fn keeps_existing_openai_v1_base() {
        assert_eq!(
            normalize_openai_api_base("https://api.addzero.site/v1/").unwrap(),
            "https://api.addzero.site/v1"
        );
    }
}
