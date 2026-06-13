// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AssistantMessageItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseOutputText,
};

/// Assistant-authored message within a thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessageItem {
    /// Identifier of the thread item.
    pub id: String,
    /// Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    /// Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    /// Identifier of the parent thread.
    pub thread_id: String,
    /// Type discriminator that is always `chatkit.assistant_message`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Ordered assistant response segments.
    pub content: Vec<ResponseOutputText>,
}
