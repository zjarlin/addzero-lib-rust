use std::env;

use anyhow::{Context, bail};
use async_openai::{Client, config::OpenAIConfig};

/// Runtime configuration for OpenAI-compatible APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiRuntimeConfig {
    /// API key read from `OPENAI_API_KEY` or compatible aliases.
    pub api_key: String,
    /// Normalized API base ending with `/v1`.
    pub api_base: String,
}

impl OpenAiRuntimeConfig {
    /// Loads OpenAI-compatible runtime configuration from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
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

    /// Builds an `async-openai` client from this runtime configuration.
    pub fn client(&self) -> Client<OpenAIConfig> {
        Client::with_config(
            OpenAIConfig::new()
                .with_api_key(self.api_key.clone())
                .with_api_base(self.api_base.clone()),
        )
    }
}

/// Normalizes an OpenAI-compatible API base URL to the `/v1` API root.
pub fn normalize_openai_api_base(api_base: &str) -> anyhow::Result<String> {
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
