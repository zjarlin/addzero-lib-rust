// Generated from OpenAPI spec. Do not edit by hand.
//! `SummaryTextContent` DTO.

use serde::{Deserialize, Serialize};

/// A summary text from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryTextContent {
    /// The type of the object. Always `summary_text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A summary of the reasoning output from the model so far.
    pub text: String,
}
