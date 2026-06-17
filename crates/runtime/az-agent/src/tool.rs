use anyhow::Context;
use async_openai::types::responses::FunctionCallOutput;
use serde::Deserialize;
use serde_json::{Value, json};

pub mod current_time;
pub mod workspace;

/// 可被模型调用的 Responses API 函数工具。
pub trait AgentTool {
    /// 暴露给模型的稳定函数名。
    fn name(&self) -> &'static str;

    /// 暴露给模型的函数描述。
    fn description(&self) -> &'static str;

    /// 函数参数的 JSON Schema。
    fn parameters_schema(&self) -> Value;

    /// 使用模型返回的原始 JSON 参数执行函数调用。
    fn execute(&self, arguments: &str) -> anyhow::Result<FunctionCallOutput>;

    /// 把当前工具转换为 Responses API 的 `tools[]` 项。
    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "name": self.name(),
            "description": self.description(),
            "parameters": self.parameters_schema(),
            "strict": true,
        })
    }
}

/// agent 工具的插件式注册表。
#[derive(Clone)]
pub struct ToolRegistry {
    tools: Vec<std::sync::Arc<dyn AgentTool + Send + Sync>>,
}

impl ToolRegistry {
    /// 根据显式工具插件创建注册表。
    pub fn new(tools: Vec<std::sync::Arc<dyn AgentTool + Send + Sync>>) -> Self {
        Self { tools }
    }

    /// 返回 Responses API 函数工具定义。
    pub fn definitions(&self) -> Vec<Value> {
        self.tools.iter().map(|tool| tool.definition()).collect()
    }

    /// 按名称执行已注册工具。
    pub fn execute(&self, call: &FunctionToolCall) -> anyhow::Result<FunctionCallOutput> {
        let tool = self
            .tools
            .iter()
            .find(|tool| tool.name() == call.name)
            .ok_or_else(|| anyhow::anyhow!("unsupported tool call `{}`", call.name))?;
        tool.execute(&call.arguments)
            .with_context(|| format!("execute tool `{}`", call.name))
    }

    /// 返回默认示例工具名。
    pub fn default_tool_choice(&self) -> &'static str {
        current_time::TOOL_NAME
    }

    /// provider 忽略工具调用时，执行本地时间 fallback。
    pub fn current_time_fallback(&self) -> anyhow::Result<FunctionCallOutput> {
        let call = FunctionToolCall {
            call_id: "local_fallback".to_string(),
            name: current_time::TOOL_NAME.to_string(),
            arguments: r#"{"timezone_hint":""}"#.to_string(),
        };
        self.execute(&call)
    }
}

/// Responses API 返回的函数调用项。
#[derive(Debug, Clone, Deserialize)]
pub struct FunctionToolCall {
    /// `function_call_output` 所需的关联 ID。
    pub call_id: String,
    /// 模型选择的函数名。
    pub name: String,
    /// 模型生成的 JSON 字符串参数。
    pub arguments: String,
}

/// 格式化函数输出，用于 CLI fallback 直接展示。
pub fn format_function_output(output: FunctionCallOutput) -> anyhow::Result<String> {
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

pub(crate) fn parse_tool_arguments(arguments: &str, tool_name: &str) -> anyhow::Result<Value> {
    serde_json::from_str::<Value>(arguments)
        .with_context(|| format!("parse arguments for tool `{tool_name}`"))
}

pub(crate) fn argument_string(arguments: &Value, name: &str) -> String {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub(crate) fn argument_bool(arguments: &Value, name: &str) -> bool {
    arguments
        .get(name)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub(crate) fn json_output(value: Value) -> FunctionCallOutput {
    value.to_string().into()
}
