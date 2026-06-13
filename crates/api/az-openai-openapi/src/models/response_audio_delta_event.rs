// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseAudioDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when there is a partial audio response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseAudioDeltaEvent {
    /// The type of the event. Always `response.audio.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A sequence number for this chunk of the stream response.
    pub sequence_number: i32,
    /// A chunk of Base64 encoded response audio bytes.
    pub delta: String,
}
