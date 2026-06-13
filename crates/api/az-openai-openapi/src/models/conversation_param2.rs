// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ConversationParam2` DTO.

use serde::{Deserialize, Serialize};

/// The conversation that this response belongs to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationParam2 {
    /// The unique ID of the conversation.
    pub id: String,
}
