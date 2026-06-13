// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ConversationResource` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonValue,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationResource {
    /// The unique ID of the conversation.
    pub id: String,
    /// The object type, which is always `conversation`.
    pub object: String,
    /// Set of 16 key-value pairs that can be attached to an object. This can be useful for storing
    /// additional information about the object in a structured format, and querying for objects via API or
    /// the dashboard. Keys are strings with a maximum length of 64 characters. Values are strings with a
    /// maximum length of 512 characters.
    pub metadata: OpenAiJsonValue,
    /// The time at which the conversation was created, measured in seconds since the Unix epoch.
    pub created_at: i64,
}
