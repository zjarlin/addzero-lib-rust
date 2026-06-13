// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CustomToolChatCompletionsCustomFormat2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CustomToolChatCompletionsCustomFormat2Grammar,
};

/// A grammar defined by the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat2 {
    /// Grammar format. Always `grammar`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Your chosen grammar.
    pub grammar: CustomToolChatCompletionsCustomFormat2Grammar,
}
