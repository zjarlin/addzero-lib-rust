// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateTranscriptionResponseStreamEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TranscriptTextDeltaEvent,
    TranscriptTextDoneEvent,
    TranscriptTextSegmentEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateTranscriptionResponseStreamEvent {
    TranscriptTextSegmentEvent(TranscriptTextSegmentEvent),
    TranscriptTextDeltaEvent(TranscriptTextDeltaEvent),
    TranscriptTextDoneEvent(TranscriptTextDoneEvent),
}
