// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UserMessageItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Attachment,
    InferenceOptions,
    UserMessageItemContentItem,
};

/// User-authored messages within a thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessageItem {
    /// Identifier of the thread item.
    pub id: String,
    /// Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    /// Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    /// Identifier of the parent thread.
    pub thread_id: String,
    #[serde(rename = "type")]
    pub type_value: String,
    /// Ordered content elements supplied by the user.
    pub content: Vec<UserMessageItemContentItem>,
    /// Attachments associated with the user message. Defaults to an empty list.
    pub attachments: Vec<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inference_options: Option<InferenceOptions>,
}
