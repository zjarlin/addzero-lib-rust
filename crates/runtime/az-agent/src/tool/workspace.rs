use async_openai::types::responses::FunctionCallOutput;
use serde_json::json;

use crate::tool::{AgentTool, argument_bool, json_output, parse_tool_arguments};

/// 工作区元数据工具的函数名。
pub const TOOL_NAME: &str = "az_current_workspace";

/// 返回本地工作区元数据的工具插件。
#[derive(Debug, Clone, Default)]
pub struct WorkspaceTool;

impl AgentTool for WorkspaceTool {
    fn name(&self) -> &'static str {
        TOOL_NAME
    }

    fn description(&self) -> &'static str {
        "Return metadata about the local az-agent workspace."
    }

    fn parameters_schema(&self) -> serde_json::Value {
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
        })
    }

    fn execute(&self, arguments: &str) -> anyhow::Result<FunctionCallOutput> {
        let arguments = parse_tool_arguments(arguments, TOOL_NAME)?;
        let include_env = argument_bool(&arguments, "include_env");
        Ok(json_output(json!({
            "crate": env!("CARGO_PKG_NAME"),
            "manifest_dir": env!("CARGO_MANIFEST_DIR"),
            "include_env": include_env,
            "openai_model_env_set": std::env::var("OPENAI_MODEL").is_ok(),
        })))
    }
}
