// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CustomToolChatCompletionsCustomFormat3GrammarFormatGrammar` DTO.

use serde::{Deserialize, Serialize};

/// Your chosen grammar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat3GrammarFormatGrammar {
    /// The grammar definition.
    pub definition: String,
    /// The syntax of the grammar definition. One of `lark` or `regex`.
    pub syntax: String,
}
