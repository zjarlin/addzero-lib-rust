// Generated from OpenAPI spec. Do not edit by hand.
//! `CustomToolChatCompletionsCustomFormat3TextFormat` DTO.

use serde::{Deserialize, Serialize};

/// Unconstrained free-form text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat3TextFormat {
    /// Unconstrained text format. Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
