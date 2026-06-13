// Generated from OpenAPI spec. Do not edit by hand.
//! `TokenCountsBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ConversationParam,
    Reasoning,
    ResponseTextParam,
    TokenCountsBodyInput,
    Tool,
    ToolChoiceParam,
    TruncationEnum,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCountsBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<TokenCountsBodyInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<ResponseTextParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
    /// The truncation strategy to use for the model response. - `auto`: If the input to this Response
    /// exceeds the model's context window size, the model will truncate the response to fit the context
    /// window by dropping items from the beginning of the conversation. - `disabled` (default): If the
    /// input size will exceed the context window size for a model, the request will fail with a 400 error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: Option<TruncationEnum>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ConversationParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoiceParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
}
