// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ConversationItemList` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ConversationItem,
};

/// A list of Conversation items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationItemList {
    /// The type of object returned, must be `list`.
    pub object: String,
    /// A list of conversation items.
    pub data: Vec<ConversationItem>,
    /// Whether there are more items available.
    pub has_more: bool,
    /// The ID of the first item in the list.
    pub first_id: String,
    /// The ID of the last item in the list.
    pub last_id: String,
}
