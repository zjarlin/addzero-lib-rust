// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventConversationItemInputAudioTranscriptionCompletedUsage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TranscriptTextUsageDuration,
    TranscriptTextUsageTokens,
};

/// Usage statistics for the transcription, this is billed according to the ASR model's pricing rather
/// than the realtime model's pricing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeServerEventConversationItemInputAudioTranscriptionCompletedUsage {
    TranscriptTextUsageTokens(TranscriptTextUsageTokens),
    TranscriptTextUsageDuration(TranscriptTextUsageDuration),
}
