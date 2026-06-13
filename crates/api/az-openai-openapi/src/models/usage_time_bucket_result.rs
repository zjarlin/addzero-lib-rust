// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UsageTimeBucketResult` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CostsResult,
    UsageAudioSpeechesResult,
    UsageAudioTranscriptionsResult,
    UsageCodeInterpreterSessionsResult,
    UsageCompletionsResult,
    UsageEmbeddingsResult,
    UsageImagesResult,
    UsageModerationsResult,
    UsageVectorStoresResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UsageTimeBucketResult {
    UsageCompletionsResult(UsageCompletionsResult),
    UsageEmbeddingsResult(UsageEmbeddingsResult),
    UsageModerationsResult(UsageModerationsResult),
    UsageImagesResult(UsageImagesResult),
    UsageAudioSpeechesResult(UsageAudioSpeechesResult),
    UsageAudioTranscriptionsResult(UsageAudioTranscriptionsResult),
    UsageVectorStoresResult(UsageVectorStoresResult),
    UsageCodeInterpreterSessionsResult(UsageCodeInterpreterSessionsResult),
    CostsResult(CostsResult),
}
