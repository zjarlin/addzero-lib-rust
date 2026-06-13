// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MCPListToolsTool` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// A tool available on an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPListToolsTool {
    /// The name of the tool.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The JSON schema describing the tool's input.
    pub input_schema: OpenAiJsonObject,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<OpenAiJsonObject>,
}
