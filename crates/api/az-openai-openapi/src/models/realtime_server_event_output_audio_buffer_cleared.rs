// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventOutputAudioBufferCleared` DTO.

use serde::{Deserialize, Serialize};

/// **WebRTC/SIP Only:** Emitted when the output audio buffer is cleared. This happens either in VAD
/// mode when the user has interrupted (`input_audio_buffer.speech_started`), or when the client has
/// emitted the `output_audio_buffer.clear` event to manually cut off the current audio response. [Learn
/// more](/docs/guides/realtime-conversations#client-and-server-events-for-audio-in-webrtc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventOutputAudioBufferCleared {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `output_audio_buffer.cleared`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the response that produced the audio.
    pub response_id: String,
}
