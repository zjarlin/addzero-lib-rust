// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateChatCompletionRequestTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionTool,
    CustomToolChatCompletions,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateChatCompletionRequestTool {
    ChatCompletionTool(ChatCompletionTool),
    CustomToolChatCompletions(CustomToolChatCompletions),
}
