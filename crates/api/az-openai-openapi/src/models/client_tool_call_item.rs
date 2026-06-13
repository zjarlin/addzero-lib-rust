// Generated from OpenAPI spec. Do not edit by hand.
//! `ClientToolCallItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ClientToolCallStatus,
};

/// Record of a client side tool invocation initiated by the assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientToolCallItem {
    /// Identifier of the thread item.
    pub id: String,
    /// Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    /// Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    /// Identifier of the parent thread.
    pub thread_id: String,
    /// Type discriminator that is always `chatkit.client_tool_call`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Execution status for the tool call.
    pub status: ClientToolCallStatus,
    /// Identifier for the client tool call.
    pub call_id: String,
    /// Tool name that was invoked.
    pub name: String,
    /// JSON-encoded arguments that were sent to the tool.
    pub arguments: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}
