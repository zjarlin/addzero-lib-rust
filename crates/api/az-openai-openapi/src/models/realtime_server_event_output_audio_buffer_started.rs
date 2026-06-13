// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventOutputAudioBufferStarted` DTO.

use serde::{Deserialize, Serialize};

/// **WebRTC/SIP Only:** Emitted when the server begins streaming audio to the client. This event is
/// emitted after an audio content part has been added (`response.content_part.added`) to the response.
/// [Learn more](/docs/guides/realtime-conversations#client-and-server-events-for-audio-in-webrtc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventOutputAudioBufferStarted {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `output_audio_buffer.started`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the response that produced the audio.
    pub response_id: String,
}
