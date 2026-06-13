// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventInputAudioBufferSpeechStopped` DTO.

use serde::{Deserialize, Serialize};

/// Returned in `server_vad` mode when the server detects the end of speech in the audio buffer. The
/// server will also send an `conversation.item.created` event with the user message item that is
/// created from the audio buffer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventInputAudioBufferSpeechStopped {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `input_audio_buffer.speech_stopped`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Milliseconds since the session started when speech stopped. This will correspond to the end of audio
    /// sent to the model, and thus includes the `min_silence_duration_ms` configured in the Session.
    pub audio_end_ms: i32,
    /// The ID of the user message item that will be created.
    pub item_id: String,
}
