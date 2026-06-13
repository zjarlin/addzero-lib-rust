// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventConversationItemInputAudioTranscriptionCompletedUsage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TranscriptTextUsageDuration,
    TranscriptTextUsageTokens,
};

/// Usage statistics for the transcription.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeBetaServerEventConversationItemInputAudioTranscriptionCompletedUsage {
    TranscriptTextUsageTokens(TranscriptTextUsageTokens),
    TranscriptTextUsageDuration(TranscriptTextUsageDuration),
}
