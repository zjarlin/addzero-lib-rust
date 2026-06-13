// Generated from OpenAPI spec. Do not edit by hand.
//! `CustomToolChatCompletionsCustomFormat3GrammarFormat` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CustomToolChatCompletionsCustomFormat3GrammarFormatGrammar,
};

/// A grammar defined by the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat3GrammarFormat {
    /// Grammar format. Always `grammar`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Your chosen grammar.
    pub grammar: CustomToolChatCompletionsCustomFormat3GrammarFormatGrammar,
}
