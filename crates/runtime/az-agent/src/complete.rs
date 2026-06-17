use std::env;

use anyhow::{Context, bail};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        chat::{
            ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
            ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        },
        responses::FunctionCallOutput,
    },
};
use chrono::Local;
use serde::Deserialize;
use serde_json::{Value, json};

const WORKSPACE_TOOL_NAME: &str = "az_current_workspace";
const CURRENT_TIME_TOOL_NAME: &str = "az_current_time";

/// Sends a single chat completion request with an optional system prompt.
pub async fn chat_completions(
    model: &str,
    system: Option<&str>,
    prompt: &str,
) -> anyhow::Result<String> {
    let client = openai_client_from_env()?;

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

/// Responses API result with request and response model metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsesResult {
    /// Model ID requested by the caller.
    pub requested_model: String,
    /// Model ID returned by the API after aliases or provider routing.
    pub response_model: String,
    /// Response object ID returned by the API.
    pub response_id: String,
    /// Response status as returned by the API.
    pub status: String,
    /// Aggregated text output from `output_text` content items.
    pub output_text: String,
    /// Warning when the provider did not honor the requested Responses tool call.
    pub warning: Option<String>,
}

/// Sends a Responses API request and resolves one demo function tool call if the model asks for it.
pub async fn responses_with_demo_tool(
    model: &str,
    instructions: Option<&str>,
    prompt: &str,
) -> anyhow::Result<ResponsesResult> {
    let client = openai_client_from_env()?;
    let request = json!({
        "model": model,
        "instructions": instructions.unwrap_or_default(),
        "input": prompt,
        "tools": tools(),
        "tool_choice": {
            "type": "function",
            "name": CURRENT_TIME_TOOL_NAME
        },
        "max_output_tokens": 2048,
    });

    tracing::info!(model = model, api = "responses", "sending OpenAI request");
    let first_response: TolerantResponse = client.responses().create_byot(request).await?;
    tracing::info!(
        requested_model = model,
        response_model = first_response.model,
        response_id = first_response.id,
        status = ?first_response.status,
        "responses api response received"
    );

    tracing::info!(
        requested_model = model,
        response_id = first_response.id,
        output_types = ?first_response.output_types(),
        "responses api output items"
    );

    let function_calls = first_response.function_calls();
    let final_response = if function_calls.is_empty() {
        let warning = format!(
            "Responses API did not return a function_call after forcing `{}`; output item types: {:?}. Local fallback executed.",
            CURRENT_TIME_TOOL_NAME,
            first_response.output_types()
        );
        tracing::warn!("{warning}");
        let fallback = execute_demo_tool(CURRENT_TIME_TOOL_NAME, r#"{"timezone_hint":""}"#)?;
        return Ok(ResponsesResult {
            requested_model: model.to_string(),
            response_model: first_response.model,
            response_id: first_response.id,
            status: first_response.status,
            output_text: format_function_output(fallback)?,
            warning: Some(warning),
        });
    } else {
        tracing::info!(
            requested_model = model,
            response_id = first_response.id,
            tool_calls = function_calls.len(),
            "executing responses api tool calls"
        );
        let mut input_items: Vec<Value> = first_response.raw_output_items();
        for call in function_calls {
            input_items.push(json!({
                "type": "function_call_output",
                "call_id": call.call_id,
                "output": execute_demo_tool(&call.name, &call.arguments)?,
                "status": "completed",
            }));
        }

        let follow_up = json!({
            "model": model,
            "instructions": instructions.unwrap_or_default(),
            "input": input_items,
            "tools": tools(),
            "max_output_tokens": 2048,
        });
        let response: TolerantResponse = client.responses().create_byot(follow_up).await?;
        tracing::info!(
            requested_model = model,
            response_model = response.model,
            response_id = response.id,
            status = ?response.status,
            "responses api follow-up response received"
        );
        response
    };

    responses_result(model, final_response)
}

fn openai_client_from_env() -> anyhow::Result<Client<OpenAIConfig>> {
    let config = OpenAiRuntimeConfig::from_env()?;
    Ok(Client::with_config(
        OpenAIConfig::new()
            .with_api_key(config.api_key)
            .with_api_base(config.api_base),
    ))
}

fn responses_result(model: &str, response: TolerantResponse) -> anyhow::Result<ResponsesResult> {
    let output_text = response
        .output_text()
        .ok_or_else(|| anyhow::anyhow!("No output_text in response"))?;
    Ok(ResponsesResult {
        requested_model: model.to_string(),
        response_model: response.model,
        response_id: response.id,
        status: response.status,
        output_text,
        warning: None,
    })
}

fn format_function_output(output: FunctionCallOutput) -> anyhow::Result<String> {
    match output {
        FunctionCallOutput::Text(text) => {
            let value =
                serde_json::from_str::<Value>(&text).context("parse function output JSON")?;
            Ok(serde_json::to_string_pretty(&value)?)
        }
        FunctionCallOutput::Content(content) => Ok(serde_json::to_string_pretty(
            &serde_json::to_value(content)?,
        )?),
    }
}

fn tools() -> Vec<Value> {
    vec![workspace_tool(), current_time_tool()]
}

fn workspace_tool() -> Value {
    function_tool(
        WORKSPACE_TOOL_NAME,
        "Return metadata about the local az-agent workspace.",
        json!({
            "type": "object",
            "properties": {
                "include_env": {
                    "type": "boolean",
                    "description": "Whether to include non-secret environment metadata."
                }
            },
            "required": ["include_env"],
            "additionalProperties": false
        }),
    )
}

fn current_time_tool() -> Value {
    function_tool(
        CURRENT_TIME_TOOL_NAME,
        "Return the current local machine time.",
        json!({
            "type": "object",
            "properties": {
                "timezone_hint": {
                    "type": "string",
                    "description": "Optional timezone name the user wants to see."
                }
            },
            "required": ["timezone_hint"],
            "additionalProperties": false
        }),
    )
}

fn function_tool(name: &str, description: &str, parameters: Value) -> Value {
    json!({
        "type": "function",
        "name": name,
        "description": description,
        "parameters": parameters,
        "strict": true,
    })
}

fn execute_demo_tool(name: &str, arguments: &str) -> anyhow::Result<FunctionCallOutput> {
    match name {
        WORKSPACE_TOOL_NAME => workspace_tool_output(arguments),
        CURRENT_TIME_TOOL_NAME => current_time_tool_output(arguments),
        _ => bail!("unsupported tool call `{name}`"),
    }
}

fn workspace_tool_output(arguments: &str) -> anyhow::Result<FunctionCallOutput> {
    let arguments = serde_json::from_str::<Value>(arguments)
        .with_context(|| format!("parse arguments for tool `{WORKSPACE_TOOL_NAME}`"))?;
    let include_env = arguments
        .get("include_env")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let output = json!({
        "crate": env!("CARGO_PKG_NAME"),
        "manifest_dir": env!("CARGO_MANIFEST_DIR"),
        "include_env": include_env,
        "openai_model_env_set": env::var("OPENAI_MODEL").is_ok(),
    });
    Ok(output.to_string().into())
}

fn current_time_tool_output(arguments: &str) -> anyhow::Result<FunctionCallOutput> {
    let arguments = serde_json::from_str::<Value>(arguments)
        .with_context(|| format!("parse arguments for tool `{CURRENT_TIME_TOOL_NAME}`"))?;
    let timezone_hint = arguments
        .get("timezone_hint")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let now = Local::now();
    let output = json!({
        "local_datetime": now.format("%Y-%m-%d %H:%M:%S").to_string(),
        "rfc3339": now.to_rfc3339(),
        "unix_timestamp": now.timestamp(),
        "timezone": now.format("%Z").to_string(),
        "utc_offset": now.format("%:z").to_string(),
        "timezone_hint": timezone_hint,
    });
    Ok(output.to_string().into())
}

#[derive(Debug, Clone, Deserialize)]
struct TolerantResponse {
    id: String,
    model: String,
    status: String,
    #[serde(default)]
    output: Vec<Value>,
}

impl TolerantResponse {
    fn output_text(&self) -> Option<String> {
        let text = self
            .output
            .iter()
            .filter(|item| item.get("type").and_then(Value::as_str) == Some("message"))
            .filter_map(|item| item.get("content").and_then(Value::as_array))
            .flat_map(|content| content.iter())
            .filter_map(|content| {
                if content.get("type").and_then(Value::as_str) == Some("output_text") {
                    content.get("text").and_then(Value::as_str)
                } else {
                    None
                }
            })
            .collect::<Vec<&str>>()
            .join("");
        if text.is_empty() { None } else { Some(text) }
    }

    fn function_calls(&self) -> Vec<TolerantFunctionCall> {
        self.output
            .iter()
            .filter(|item| item.get("type").and_then(Value::as_str) == Some("function_call"))
            .filter_map(|item| serde_json::from_value::<TolerantFunctionCall>(item.clone()).ok())
            .collect()
    }

    fn raw_output_items(&self) -> Vec<Value> {
        self.output.clone()
    }

    fn output_types(&self) -> Vec<String> {
        self.output
            .iter()
            .filter_map(|item| item.get("type").and_then(Value::as_str))
            .map(ToString::to_string)
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct TolerantFunctionCall {
    call_id: String,
    name: String,
    arguments: String,
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
