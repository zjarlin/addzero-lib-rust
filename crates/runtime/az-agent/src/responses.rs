use async_openai::{Client, config::OpenAIConfig};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    structured::StructuredOutputSpec,
    tool::{FunctionToolCall, ToolRegistry, format_function_output},
    vision::VisionInput,
};

/// Request for a Responses API run.
#[derive(Debug, Clone)]
pub struct ResponsesRunRequest {
    /// Requested model ID.
    pub model: String,
    /// Optional developer/system instructions.
    pub instructions: Option<String>,
    /// User input.
    pub prompt: String,
    /// Optional image inputs for vision-capable providers.
    pub images: Vec<VisionInput>,
    /// Optional structured output schema.
    pub structured_output: Option<StructuredOutputSpec>,
    /// Optional concrete function tool choice.
    pub tool_choice: Option<String>,
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

/// Runner for Responses API calls, including function tool loops.
#[derive(Clone)]
pub struct ResponsesRunner {
    client: Client<OpenAIConfig>,
    tools: ToolRegistry,
}

impl ResponsesRunner {
    /// Creates a runner from an API client and tool registry.
    pub fn new(client: Client<OpenAIConfig>, tools: ToolRegistry) -> Self {
        Self { client, tools }
    }

    /// Sends a Responses API request, executes returned function calls, and submits tool outputs.
    pub async fn run(&self, request: ResponsesRunRequest) -> anyhow::Result<ResponsesResult> {
        let request_body = self.create_request_body(&request, request_input(&request));

        tracing::info!(
            model = request.model,
            api = "responses",
            "sending OpenAI request"
        );
        let first_response: TolerantResponse =
            self.client.responses().create_byot(request_body).await?;
        tracing::info!(
            requested_model = request.model,
            response_model = first_response.model,
            response_id = first_response.id,
            status = ?first_response.status,
            "responses api response received"
        );
        tracing::info!(
            requested_model = request.model,
            response_id = first_response.id,
            output_types = ?first_response.output_types(),
            "responses api output items"
        );

        let function_calls = first_response.function_calls();
        if function_calls.is_empty() {
            return self.provider_ignored_tool_fallback(&request, first_response);
        }

        tracing::info!(
            requested_model = request.model,
            response_id = first_response.id,
            tool_calls = function_calls.len(),
            "executing responses api tool calls"
        );
        let mut input_items = first_response.raw_output_items();
        for call in function_calls {
            input_items.push(json!({
                "type": "function_call_output",
                "call_id": call.call_id,
                "output": self.tools.execute(&call)?,
                "status": "completed",
            }));
        }

        let follow_up_body = self.create_request_body(&request, Value::Array(input_items));
        let response: TolerantResponse =
            self.client.responses().create_byot(follow_up_body).await?;
        tracing::info!(
            requested_model = request.model,
            response_model = response.model,
            response_id = response.id,
            status = ?response.status,
            "responses api follow-up response received"
        );

        responses_result(&request.model, response, None)
    }

    fn create_request_body(&self, request: &ResponsesRunRequest, input: Value) -> Value {
        let mut body = json!({
            "model": request.model,
            "instructions": request.instructions.clone().unwrap_or_default(),
            "input": input,
            "tools": self.tools.definitions(),
            "max_output_tokens": 2048,
        });

        if let Some(tool_choice) = &request.tool_choice {
            body["tool_choice"] = json!({
                "type": "function",
                "name": tool_choice,
            });
        }

        if let Some(structured_output) = &request.structured_output {
            body["text"] = structured_output.to_response_text_json();
        }

        body
    }

    fn provider_ignored_tool_fallback(
        &self,
        request: &ResponsesRunRequest,
        response: TolerantResponse,
    ) -> anyhow::Result<ResponsesResult> {
        let Some(tool_choice) = request.tool_choice.as_deref() else {
            return responses_result(&request.model, response, None);
        };
        let warning = format!(
            "Responses API did not return a function_call after forcing `{}`; output item types: {:?}. Local fallback executed.",
            tool_choice,
            response.output_types()
        );
        tracing::warn!("{warning}");
        let fallback = self.tools.current_time_fallback()?;
        Ok(ResponsesResult {
            requested_model: request.model.clone(),
            response_model: response.model,
            response_id: response.id,
            status: response.status,
            output_text: format_function_output(fallback)?,
            warning: Some(warning),
        })
    }
}

fn responses_result(
    model: &str,
    response: TolerantResponse,
    warning: Option<String>,
) -> anyhow::Result<ResponsesResult> {
    let output_text = response
        .output_text()
        .ok_or_else(|| anyhow::anyhow!("No output_text in response"))?;
    Ok(ResponsesResult {
        requested_model: model.to_string(),
        response_model: response.model,
        response_id: response.id,
        status: response.status,
        output_text,
        warning,
    })
}

fn request_input(request: &ResponsesRunRequest) -> Value {
    if request.images.is_empty() {
        return request.prompt.clone().into();
    }

    let mut content = vec![json!({
        "type": "input_text",
        "text": request.prompt,
    })];
    content.extend(request.images.iter().map(VisionInput::to_responses_content));
    json!([
        {
            "role": "user",
            "content": content,
        }
    ])
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

    fn function_calls(&self) -> Vec<FunctionToolCall> {
        self.output
            .iter()
            .filter(|item| item.get("type").and_then(Value::as_str) == Some("function_call"))
            .filter_map(|item| serde_json::from_value::<FunctionToolCall>(item.clone()).ok())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vision::VisionDetail;

    #[test]
    fn builds_multimodal_responses_input() {
        let request = ResponsesRunRequest {
            model: "vision-model".to_string(),
            instructions: None,
            prompt: "描述图片".to_string(),
            images: vec![
                VisionInput::image_url("https://example.com/a.png").with_detail(VisionDetail::Low),
            ],
            structured_output: None,
            tool_choice: None,
        };

        assert_eq!(
            request_input(&request),
            json!([
                {
                    "role": "user",
                    "content": [
                        {"type": "input_text", "text": "描述图片"},
                        {"type": "input_image", "image_url": "https://example.com/a.png", "detail": "low"}
                    ]
                }
            ])
        );
    }
}
