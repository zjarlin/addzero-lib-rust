// Generated from OpenAPI spec. Do not edit by hand.
//! `TranscriptTextUsageDuration` DTO.

use serde::{Deserialize, Serialize};

/// Usage statistics for models billed by audio input duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptTextUsageDuration {
    /// The type of the usage object. Always `duration` for this variant.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Duration of the input audio in seconds.
    pub seconds: f64,
}
