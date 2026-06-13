// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeClientEventOutputAudioBufferClear` DTO.

use serde::{Deserialize, Serialize};

/// **WebRTC/SIP Only:** Emit to cut off the current audio response. This will trigger the server to
/// stop generating audio and emit a `output_audio_buffer.cleared` event. This event should be preceded
/// by a `response.cancel` client event to stop the generation of the current response. [Learn
/// more](/docs/guides/realtime-conversations#client-and-server-events-for-audio-in-webrtc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeClientEventOutputAudioBufferClear {
    /// The unique ID of the client event used for error handling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `output_audio_buffer.clear`.
    #[serde(rename = "type")]
    pub type_value: String,
}
