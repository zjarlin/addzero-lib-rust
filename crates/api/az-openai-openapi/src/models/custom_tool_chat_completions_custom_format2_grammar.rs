// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CustomToolChatCompletionsCustomFormat2Grammar` DTO.

use serde::{Deserialize, Serialize};

/// Your chosen grammar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat2Grammar {
    /// The grammar definition.
    pub definition: String,
    /// The syntax of the grammar definition. One of `lark` or `regex`.
    pub syntax: String,
}
