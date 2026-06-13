// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionMessageList` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionMessageListDataItem,
};

/// An object representing a list of chat completion messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionMessageList {
    /// The type of this object. It is always set to "list".
    pub object: String,
    /// An array of chat completion message objects.
    pub data: Vec<ChatCompletionMessageListDataItem>,
    /// The identifier of the first chat message in the data array.
    pub first_id: String,
    /// The identifier of the last chat message in the data array.
    pub last_id: String,
    /// Indicates whether there are more chat messages available.
    pub has_more: bool,
}
