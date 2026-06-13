// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateSpeechResponseStreamEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    SpeechAudioDeltaEvent,
    SpeechAudioDoneEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateSpeechResponseStreamEvent {
    SpeechAudioDeltaEvent(SpeechAudioDeltaEvent),
    SpeechAudioDoneEvent(SpeechAudioDoneEvent),
}
