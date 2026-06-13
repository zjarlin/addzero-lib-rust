// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalRunOutputItemSampleOutputItem` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunOutputItemSampleOutputItem {
    /// The role of the message (e.g. "system", "assistant", "user").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The content of the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}
