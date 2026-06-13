// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2ChatMessage` DTO.

use serde::{Deserialize, Serialize};

/// ChatMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2ChatMessage {
    /// The role of the message (e.g. "system", "assistant", "user").
    pub role: String,
    /// The content of the message.
    pub content: String,
}
