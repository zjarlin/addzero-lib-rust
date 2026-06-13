// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventConversationItemAdded` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeConversationItem,
};

/// Sent by the server when an Item is added to the default Conversation. This can happen in several
/// cases: - When the client sends a `conversation.item.create` event. - When the input audio buffer is
/// committed. In this case the item will be a user message containing the audio from the buffer. - When
/// the model is generating a Response. In this case the `conversation.item.added` event will be sent
/// when the model starts generating a specific Item, and thus it will not yet have any content (and
/// `status` will be `in_progress`). The event will include the full content of the Item (except when
/// model is generating a Response) except for audio data, which can be retrieved separately with a
/// `conversation.item.retrieve` event if necessary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventConversationItemAdded {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.item.added`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: Option<String>,
    pub item: RealtimeConversationItem,
}
