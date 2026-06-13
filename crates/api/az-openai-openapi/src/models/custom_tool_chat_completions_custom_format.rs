// Generated from OpenAPI spec. Do not edit by hand.
//! `CustomToolChatCompletionsCustomFormat` DTO.

use serde::{Deserialize, Serialize};

/// Unconstrained free-form text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat {
    /// Unconstrained text format. Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
