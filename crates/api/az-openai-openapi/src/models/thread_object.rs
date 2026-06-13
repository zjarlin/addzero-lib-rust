// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ThreadObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Metadata,
    ThreadObjectToolResources,
};

/// Represents a thread that contains [messages](/docs/api-reference/messages).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadObject {
    /// The identifier, which can be referenced in API endpoints.
    pub id: String,
    /// The object type, which is always `thread`.
    pub object: String,
    /// The Unix timestamp (in seconds) for when the thread was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<ThreadObjectToolResources>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
