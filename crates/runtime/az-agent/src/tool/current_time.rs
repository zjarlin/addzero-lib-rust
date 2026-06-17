use async_openai::types::responses::FunctionCallOutput;
use chrono::Local;
use serde_json::json;

use crate::tool::{AgentTool, argument_string, json_output, parse_tool_arguments};

/// 当前时间工具的函数名。
pub const TOOL_NAME: &str = "az_current_time";

/// 返回本机当前时间的工具插件。
#[derive(Debug, Clone, Default)]
pub struct CurrentTimeTool;

impl AgentTool for CurrentTimeTool {
    fn name(&self) -> &'static str {
        TOOL_NAME
    }

    fn description(&self) -> &'static str {
        "Return the current local machine time."
    }

    fn parameters_schema(&self) -> serde_json::Value {
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
        })
    }

    fn execute(&self, arguments: &str) -> anyhow::Result<FunctionCallOutput> {
        let arguments = parse_tool_arguments(arguments, TOOL_NAME)?;
        let timezone_hint = argument_string(&arguments, "timezone_hint");
        let now = Local::now();
        Ok(json_output(json!({
            "local_datetime": now.format("%Y-%m-%d %H:%M:%S").to_string(),
            "rfc3339": now.to_rfc3339(),
            "unix_timestamp": now.timestamp(),
            "timezone": now.format("%Z").to_string(),
            "utc_offset": now.format("%:z").to_string(),
            "timezone_hint": timezone_hint,
        })))
    }
}
