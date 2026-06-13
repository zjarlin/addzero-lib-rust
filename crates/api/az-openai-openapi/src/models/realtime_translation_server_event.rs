// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationServerEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeServerEventError,
    RealtimeTranslationServerEventSessionClosed,
    RealtimeTranslationServerEventSessionCreated,
    RealtimeTranslationServerEventSessionInputTranscriptDelta,
    RealtimeTranslationServerEventSessionOutputAudioDelta,
    RealtimeTranslationServerEventSessionOutputTranscriptDelta,
    RealtimeTranslationServerEventSessionUpdated,
};

/// A Realtime translation server event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeTranslationServerEvent {
    RealtimeServerEventError(RealtimeServerEventError),
    RealtimeTranslationServerEventSessionCreated(RealtimeTranslationServerEventSessionCreated),
    RealtimeTranslationServerEventSessionUpdated(RealtimeTranslationServerEventSessionUpdated),
    RealtimeTranslationServerEventSessionClosed(RealtimeTranslationServerEventSessionClosed),
    RealtimeTranslationServerEventSessionInputTranscriptDelta(RealtimeTranslationServerEventSessionInputTranscriptDelta),
    RealtimeTranslationServerEventSessionOutputTranscriptDelta(RealtimeTranslationServerEventSessionOutputTranscriptDelta),
    RealtimeTranslationServerEventSessionOutputAudioDelta(RealtimeTranslationServerEventSessionOutputAudioDelta),
}
