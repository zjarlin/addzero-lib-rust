// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionRequestAssistantMessage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionMessageToolCalls,
    ChatCompletionRequestAssistantMessageAudio,
    ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestAssistantMessageFunctionCall,
};

/// Messages sent by the model in response to user messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestAssistantMessage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<ChatCompletionRequestAssistantMessageContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    /// The role of the messages author, in this case `assistant`.
    pub role: String,
    /// An optional name for the participant. Provides the model information to differentiate between
    /// participants of the same role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<ChatCompletionRequestAssistantMessageAudio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<ChatCompletionMessageToolCalls>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_call: Option<ChatCompletionRequestAssistantMessageFunctionCall>,
}
