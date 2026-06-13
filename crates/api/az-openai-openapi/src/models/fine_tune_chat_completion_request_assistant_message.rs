// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuneChatCompletionRequestAssistantMessage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionMessageToolCalls,
    FineTuneChatCompletionRequestAssistantMessageAudio,
    FineTuneChatCompletionRequestAssistantMessageContent,
    FineTuneChatCompletionRequestAssistantMessageFunctionCall,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuneChatCompletionRequestAssistantMessage {
    /// Controls whether the assistant message is trained against (0 or 1)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<FineTuneChatCompletionRequestAssistantMessageContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    /// The role of the messages author, in this case `assistant`.
    pub role: String,
    /// An optional name for the participant. Provides the model information to differentiate between
    /// participants of the same role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<FineTuneChatCompletionRequestAssistantMessageAudio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<ChatCompletionMessageToolCalls>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FineTuneChatCompletionRequestAssistantMessageFunctionCall>,
}
