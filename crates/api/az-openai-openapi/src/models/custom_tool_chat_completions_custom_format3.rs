// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CustomToolChatCompletionsCustomFormat3` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CustomToolChatCompletionsCustomFormat3GrammarFormat,
    CustomToolChatCompletionsCustomFormat3TextFormat,
};

/// The input format for the custom tool. Default is unconstrained text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomToolChatCompletionsCustomFormat3 {
    TextFormat(CustomToolChatCompletionsCustomFormat3TextFormat),
    GrammarFormat(CustomToolChatCompletionsCustomFormat3GrammarFormat),
}
