// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageObjectAttachment,
    MessageObjectContentItem,
    MessageObjectIncompleteDetails,
    Metadata,
};

/// Represents a message within a [thread](/docs/api-reference/threads).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageObject {
    /// The identifier, which can be referenced in API endpoints.
    pub id: String,
    /// The object type, which is always `thread.message`.
    pub object: String,
    /// The Unix timestamp (in seconds) for when the message was created.
    pub created_at: i64,
    /// The [thread](/docs/api-reference/threads) ID that this message belongs to.
    pub thread_id: String,
    /// The status of the message, which can be either `in_progress`, `incomplete`, or `completed`.
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<MessageObjectIncompleteDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incomplete_at: Option<i64>,
    /// The entity that produced the message. One of `user` or `assistant`.
    pub role: String,
    /// The content of the message in array of text and/or images.
    pub content: Vec<MessageObjectContentItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<MessageObjectAttachment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
