// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseReasoningSummaryPartAddedEventPart` DTO.

use serde::{Deserialize, Serialize};

/// The summary part that was added.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseReasoningSummaryPartAddedEventPart {
    /// The type of the summary part. Always `summary_text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text of the summary part.
    pub text: String,
}
