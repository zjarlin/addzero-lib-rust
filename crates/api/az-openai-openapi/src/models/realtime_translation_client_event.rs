// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationClientEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationClientEventInputAudioBufferAppend,
    RealtimeTranslationClientEventSessionClose,
    RealtimeTranslationClientEventSessionUpdate,
};

/// A Realtime translation client event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeTranslationClientEvent {
    RealtimeTranslationClientEventSessionUpdate(RealtimeTranslationClientEventSessionUpdate),
    RealtimeTranslationClientEventInputAudioBufferAppend(RealtimeTranslationClientEventInputAudioBufferAppend),
    RealtimeTranslationClientEventSessionClose(RealtimeTranslationClientEventSessionClose),
}
