// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseFormatTextGrammar` DTO.

use serde::{Deserialize, Serialize};

/// A custom grammar for the model to follow when generating text. Learn more in the [custom grammars
/// guide](/docs/guides/custom-grammars).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormatTextGrammar {
    /// The type of response format being defined. Always `grammar`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The custom grammar for the model to follow.
    pub grammar: String,
}
