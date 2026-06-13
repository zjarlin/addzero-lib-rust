// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateTranscriptionResponseDiarizedJsonUsage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TranscriptTextUsageDuration,
    TranscriptTextUsageTokens,
};

/// Token or duration usage statistics for the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateTranscriptionResponseDiarizedJsonUsage {
    TranscriptTextUsageTokens(TranscriptTextUsageTokens),
    TranscriptTextUsageDuration(TranscriptTextUsageDuration),
}
