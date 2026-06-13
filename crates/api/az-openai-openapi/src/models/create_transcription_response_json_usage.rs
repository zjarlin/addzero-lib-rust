// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateTranscriptionResponseJsonUsage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TranscriptTextUsageDuration,
    TranscriptTextUsageTokens,
};

/// Token usage statistics for the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateTranscriptionResponseJsonUsage {
    TranscriptTextUsageTokens(TranscriptTextUsageTokens),
    TranscriptTextUsageDuration(TranscriptTextUsageDuration),
}
