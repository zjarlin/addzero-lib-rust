use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use async_openai::types::responses::FunctionCallOutput;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    chat::{ChatBackend, ChatMessage, ChatRequest},
    responses::{ResponsesResult, ResponsesRunRequest},
    spi::AgentResponsesSpi,
    tool::{FunctionToolCall, ToolRegistry, format_function_output},
};

const DEFAULT_MAX_TOOL_ROUNDS: usize = 4;

/// 将纯聊天模型 API 包装成带本地工具的 Responses 风格 agent。
#[derive(Clone)]
pub struct ChatResponsesAgentRunner<B>
where
    B: ChatBackend,
{
    backend: B,
    tools: ToolRegistry,
    id_sequence: Arc<AtomicUsize>,
    max_tool_rounds: usize,
}

impl<B> ChatResponsesAgentRunner<B>
where
    B: ChatBackend,
{
    /// 创建暴露 Responses 风格工具行为的纯聊天适配器。
    pub fn new(backend: B, tools: ToolRegistry) -> Self {
        Self {
            backend,
            tools,
            id_sequence: Arc::new(AtomicUsize::new(1)),
            max_tool_rounds: DEFAULT_MAX_TOOL_ROUNDS,
        }
    }

    /// 在纯聊天后端之上运行 Responses 风格请求。
    pub async fn run(&self, request: ResponsesRunRequest) -> anyhow::Result<ResponsesResult> {
        let mut messages = vec![ChatMessage::system(self.protocol_prompt(&request))];
        if let Some(instructions) = request
            .instructions
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            messages.push(ChatMessage::system(instructions.clone()));
        }
        messages
            .push(ChatMessage::user(request.prompt.clone()).with_images(request.images.clone()));

        let mut last_response_model = request.model.clone();
        let mut last_response_id = self.next_response_id();
        let mut executed_tools = Vec::new();

        for round in 0..self.max_tool_rounds {
            let response = self
                .backend
                .chat(ChatRequest {
                    model: request.model.clone(),
                    messages: messages.clone(),
                })
                .await?;
            last_response_model = response.model;
            last_response_id = response.id;
            let adapter_output = match parse_adapter_output(&response.content) {
                Ok(output) => output,
                Err(error) => {
                    messages.push(ChatMessage::assistant(response.content));
                    messages.push(ChatMessage::user(format!(
                        "Protocol error: {error}. Return exactly one JSON object matching the requested tool protocol."
                    )));
                    continue;
                }
            };

            match adapter_output {
                ChatAdapterOutput::Final { answer } => {
                    if let Some(required_tool) = request.tool_choice.as_deref()
                        && !executed_tools
                            .iter()
                            .any(|executed_tool: &String| executed_tool == required_tool)
                    {
                        messages.push(ChatMessage::assistant(response.content));
                        messages.push(ChatMessage::user(format!(
                            "Protocol error: `{required_tool}` must be called before a final answer. Return exactly: {{\"type\":\"function_call\",\"call_id\":\"call_{}\",\"name\":\"{required_tool}\",\"arguments\":{{}}}}",
                            round + 1
                        )));
                        continue;
                    }
                    return Ok(ResponsesResult {
                        requested_model: request.model,
                        response_model: last_response_model,
                        response_id: last_response_id,
                        status: "completed".to_string(),
                        output_text: final_answer_text(answer)?,
                        warning: Some(
                            "Tool calling was emulated with a chat-only protocol adapter."
                                .to_string(),
                        ),
                    });
                }
                ChatAdapterOutput::ToolCall {
                    call_id,
                    name,
                    arguments,
                } => {
                    let call = FunctionToolCall {
                        call_id: call_id.unwrap_or_else(|| self.next_call_id(round)),
                        name,
                        arguments: arguments.to_string(),
                    };
                    let output = self.tools.execute(&call)?;
                    executed_tools.push(call.name.clone());
                    messages.push(ChatMessage::assistant(response.content));
                    messages.push(ChatMessage::user(tool_result_message(&call, output)?));
                }
            }
        }

        Ok(ResponsesResult {
            requested_model: request.model,
            response_model: last_response_model,
            response_id: last_response_id,
            status: "incomplete".to_string(),
            output_text: String::new(),
            warning: Some(format!(
                "Chat-only tool adapter stopped after {} tool rounds.",
                self.max_tool_rounds
            )),
        })
    }

    fn protocol_prompt(&self, request: &ResponsesRunRequest) -> String {
        let mut prompt = format!(
            r#"You are running inside an OpenAI Responses compatibility adapter.
You must output exactly one JSON object and no markdown.

If a tool is needed, output:
{{"type":"function_call","call_id":"call_1","name":"tool_name","arguments":{{}}}}

If no tool is needed or after tool results are provided, output:
{{"type":"final","answer":"final answer text"}}

Available tools:
{}"#,
            serde_json::to_string_pretty(&self.tools.definitions()).unwrap_or_default()
        );

        if let Some(tool_choice) = &request.tool_choice {
            prompt.push_str(&format!(
                "\nYou must call `{tool_choice}` before producing a final answer."
            ));
        }

        if let Some(structured_output) = &request.structured_output {
            prompt.push_str(&format!(
                "\nWhen producing final output, `answer` may be a JSON object. If structured output is requested, `answer` must satisfy this JSON schema: {}",
                serde_json::to_string(&structured_output.schema).unwrap_or_default()
            ));
        }

        prompt
    }

    fn next_response_id(&self) -> String {
        format!(
            "chat_resp_{}",
            self.id_sequence.fetch_add(1, Ordering::Relaxed)
        )
    }

    fn next_call_id(&self, round: usize) -> String {
        format!(
            "call_{}_{}",
            round + 1,
            self.id_sequence.fetch_add(1, Ordering::Relaxed)
        )
    }
}

#[async_trait::async_trait]
impl<B> AgentResponsesSpi for ChatResponsesAgentRunner<B>
where
    B: ChatBackend,
{
    async fn run_responses(&self, request: ResponsesRunRequest) -> anyhow::Result<ResponsesResult> {
        self.run(request).await
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type")]
enum ChatAdapterOutput {
    #[serde(rename = "function_call")]
    ToolCall {
        call_id: Option<String>,
        name: String,
        #[serde(default)]
        arguments: Value,
    },
    #[serde(rename = "final")]
    Final { answer: Value },
}

fn parse_adapter_output(content: &str) -> anyhow::Result<ChatAdapterOutput> {
    let value = serde_json::from_str::<ChatAdapterOutput>(content.trim()).map_err(|error| {
        anyhow::anyhow!(
            "chat-only backend did not return the required tool protocol JSON: {error}; raw output: {content}"
        )
    })?;
    Ok(value)
}

fn tool_result_message(
    call: &FunctionToolCall,
    output: FunctionCallOutput,
) -> anyhow::Result<String> {
    Ok(json!({
        "type": "function_call_output",
        "call_id": call.call_id,
        "name": call.name,
        "output": format_function_output(output)?,
        "status": "completed"
    })
    .to_string())
}

fn final_answer_text(answer: Value) -> anyhow::Result<String> {
    if let Some(answer) = answer.as_str() {
        Ok(answer.to_string())
    } else {
        Ok(serde_json::to_string(&answer)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use crate::tool::{AgentTool, current_time::CurrentTimeTool};

    #[test]
    fn parses_function_call_protocol_output() {
        let output = parse_adapter_output(
            r#"{"type":"function_call","call_id":"call_1","name":"az_current_time","arguments":{"timezone_hint":""}}"#,
        )
        .unwrap();

        assert_eq!(
            output,
            ChatAdapterOutput::ToolCall {
                call_id: Some("call_1".to_string()),
                name: "az_current_time".to_string(),
                arguments: json!({"timezone_hint": ""})
            }
        );
    }

    #[test]
    fn rejects_plain_text_protocol_output() {
        let error = parse_adapter_output("现在是下午六点").unwrap_err();

        assert!(
            error
                .to_string()
                .contains("did not return the required tool protocol JSON")
        );
    }

    #[tokio::test]
    async fn wraps_chat_only_backend_into_tool_loop() {
        #[derive(Clone)]
        struct ScriptedBackend {
            outputs: Arc<Mutex<Vec<String>>>,
        }

        #[async_trait::async_trait]
        impl ChatBackend for ScriptedBackend {
            async fn chat(
                &self,
                request: ChatRequest,
            ) -> anyhow::Result<crate::chat::ChatResponse> {
                let mut outputs = self.outputs.lock().expect("scripted outputs lock");
                let content = outputs.remove(0);
                Ok(crate::chat::ChatResponse {
                    id: format!("chat_{}", request.messages.len()),
                    model: request.model,
                    content,
                })
            }
        }

        let backend = ScriptedBackend {
            outputs: Arc::new(Mutex::new(vec![
                r#"{"type":"function_call","call_id":"call_1","name":"az_current_time","arguments":{"timezone_hint":""}}"#
                    .to_string(),
                r#"{"type":"final","answer":"已通过工具获取当前时间。"}"#.to_string(),
            ])),
        };
        let tools = ToolRegistry::new(vec![
            Arc::new(CurrentTimeTool) as Arc<dyn AgentTool + Send + Sync>
        ]);
        let runner = ChatResponsesAgentRunner::new(backend, tools);

        let result = runner
            .run(ResponsesRunRequest {
                model: "chat-only-model".to_string(),
                instructions: None,
                prompt: "现在几点?".to_string(),
                images: Vec::new(),
                structured_output: None,
                tool_choice: Some("az_current_time".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(result.requested_model, "chat-only-model");
        assert_eq!(result.response_model, "chat-only-model");
        assert_eq!(result.status, "completed");
        assert_eq!(result.output_text, "已通过工具获取当前时间。");
        assert!(
            result
                .warning
                .as_deref()
                .unwrap_or_default()
                .contains("emulated")
        );
    }

    #[tokio::test]
    async fn rejects_final_until_required_tool_is_called() {
        #[derive(Clone)]
        struct ScriptedBackend {
            outputs: Arc<Mutex<Vec<String>>>,
        }

        #[async_trait::async_trait]
        impl ChatBackend for ScriptedBackend {
            async fn chat(
                &self,
                request: ChatRequest,
            ) -> anyhow::Result<crate::chat::ChatResponse> {
                let mut outputs = self.outputs.lock().expect("scripted outputs lock");
                let content = outputs.remove(0);
                Ok(crate::chat::ChatResponse {
                    id: format!("chat_{}", request.messages.len()),
                    model: request.model,
                    content,
                })
            }
        }

        let backend = ScriptedBackend {
            outputs: Arc::new(Mutex::new(vec![
                r#"{"type":"final","answer":"绕过工具的回答"}"#.to_string(),
                r#"{"type":"function_call","call_id":"call_1","name":"az_current_time","arguments":{"timezone_hint":""}}"#
                    .to_string(),
                r#"{"type":"final","answer":"已通过工具获取当前时间。"}"#.to_string(),
            ])),
        };
        let tools = ToolRegistry::new(vec![
            Arc::new(CurrentTimeTool) as Arc<dyn AgentTool + Send + Sync>
        ]);
        let runner = ChatResponsesAgentRunner::new(backend, tools);

        let result = runner
            .run(ResponsesRunRequest {
                model: "chat-only-model".to_string(),
                instructions: None,
                prompt: "现在几点?".to_string(),
                images: Vec::new(),
                structured_output: None,
                tool_choice: Some("az_current_time".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(result.output_text, "已通过工具获取当前时间。");
    }

    #[test]
    fn final_answer_can_be_structured_json() {
        let output = parse_adapter_output(
            r#"{"type":"final","answer":{"answer":"现在","utc_offset":"+08:00"}}"#,
        )
        .unwrap();

        let ChatAdapterOutput::Final { answer } = output else {
            panic!("expected final output");
        };

        assert_eq!(
            final_answer_text(answer).unwrap(),
            r#"{"answer":"现在","utc_offset":"+08:00"}"#
        );
    }
}
