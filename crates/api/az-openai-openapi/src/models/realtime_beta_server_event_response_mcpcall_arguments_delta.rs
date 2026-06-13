// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaServerEventResponseMCPCallArgumentsDelta` DTO.

use serde::{Deserialize, Serialize};

/// Returned when MCP tool call arguments are updated during response generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventResponseMCPCallArgumentsDelta {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.mcp_call_arguments.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the response.
    pub response_id: String,
    /// The ID of the MCP tool call item.
    pub item_id: String,
    /// The index of the output item in the response.
    pub output_index: i32,
    /// The JSON-encoded arguments delta.
    pub delta: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obfuscation: Option<String>,
}
