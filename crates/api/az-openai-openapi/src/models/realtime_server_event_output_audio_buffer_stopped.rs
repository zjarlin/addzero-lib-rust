// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventOutputAudioBufferStopped` DTO.

use serde::{Deserialize, Serialize};

/// **WebRTC/SIP Only:** Emitted when the output audio buffer has been completely drained on the server,
/// and no more audio is forthcoming. This event is emitted after the full response data has been sent
/// to the client (`response.done`). [Learn more](/docs/guides/realtime-conversations#client-and-server-
/// events-for-audio-in-webrtc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventOutputAudioBufferStopped {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `output_audio_buffer.stopped`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the response that produced the audio.
    pub response_id: String,
}
