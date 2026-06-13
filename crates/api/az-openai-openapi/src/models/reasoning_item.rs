// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ReasoningItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ReasoningTextContent,
    SummaryTextContent,
};

/// A description of the chain of thought used by a reasoning model while generating a response. Be sure
/// to include these items in your `input` to the Responses API for subsequent turns of a conversation
/// if you are manually [managing context](/docs/guides/conversation-state).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningItem {
    /// The type of the object. Always `reasoning`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique identifier of the reasoning content.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
    /// Reasoning summary content.
    pub summary: Vec<SummaryTextContent>,
    /// Reasoning text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ReasoningTextContent>>,
    /// The status of the item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are
    /// returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}
