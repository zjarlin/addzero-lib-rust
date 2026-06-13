// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionDeleted` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionDeleted {
    /// The type of object being deleted.
    pub object: String,
    /// The ID of the chat completion that was deleted.
    pub id: String,
    /// Whether the chat completion was deleted.
    pub deleted: bool,
}
