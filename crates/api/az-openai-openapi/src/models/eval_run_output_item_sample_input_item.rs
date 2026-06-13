// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalRunOutputItemSampleInputItem` DTO.

use serde::{Deserialize, Serialize};

/// An input message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunOutputItemSampleInputItem {
    /// The role of the message sender (e.g., system, user, developer).
    pub role: String,
    /// The content of the message.
    pub content: String,
}
