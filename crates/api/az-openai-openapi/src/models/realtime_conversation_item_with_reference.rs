// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeConversationItemWithReference` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeConversationItemWithReferenceContentItem,
};

/// The item to add to the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeConversationItemWithReference {
    /// For an item of type (`message` | `function_call` | `function_call_output`) this field allows the
    /// client to assign the unique ID of the item. It is not required because the server will generate one
    /// if not provided. For an item of type `item_reference`, this field is required and is a reference to
    /// any item that has previously existed in the conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The type of the item (`message`, `function_call`, `function_call_output`, `item_reference`).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// Identifier for the API object being returned - always `realtime.item`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The status of the item (`completed`, `incomplete`, `in_progress`). These have no effect on the
    /// conversation, but are accepted for consistency with the `conversation.item.created` event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// The role of the message sender (`user`, `assistant`, `system`), only applicable for `message` items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The content of the message, applicable for `message` items. - Message items of role `system` support
    /// only `input_text` content - Message items of role `user` support `input_text` and `input_audio`
    /// content - Message items of role `assistant` support `text` content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<RealtimeConversationItemWithReferenceContentItem>>,
    /// The ID of the function call (for `function_call` and `function_call_output` items). If passed on a
    /// `function_call_output` item, the server will check that a `function_call` item with the same ID
    /// exists in the conversation history.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    /// The name of the function being called (for `function_call` items).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The arguments of the function call (for `function_call` items).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    /// The output of the function call (for `function_call_output` items).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}
