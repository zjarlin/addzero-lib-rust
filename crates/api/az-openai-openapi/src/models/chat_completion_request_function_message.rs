// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestFunctionMessage` DTO.

use serde::{Deserialize, Serialize};

/// Function message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestFunctionMessage {
    /// The role of the messages author, in this case `function`.
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The name of the function to call.
    pub name: String,
}
