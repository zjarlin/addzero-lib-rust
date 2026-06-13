// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseReasoningSummaryPartDoneEventPart` DTO.

use serde::{Deserialize, Serialize};

/// The completed summary part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseReasoningSummaryPartDoneEventPart {
    /// The type of the summary part. Always `summary_text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text of the summary part.
    pub text: String,
}
