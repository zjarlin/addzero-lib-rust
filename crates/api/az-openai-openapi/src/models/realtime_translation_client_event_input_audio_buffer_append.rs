// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationClientEventInputAudioBufferAppend` DTO.

use serde::{Deserialize, Serialize};

/// Send this event to append audio bytes to the translation session input audio buffer. WebSocket
/// translation sessions accept base64-encoded 24 kHz PCM16 mono little-endian raw audio bytes.
/// Unsupported websocket audio formats return a validation error because lower-quality audio materially
/// degrades translation quality. Translation consumes 200 ms engine frames. For best realtime behavior,
/// append audio in 200 ms chunks. If a chunk is shorter, the server buffers it until it has enough
/// audio for one frame. If a chunk is longer, the server splits it into 200 ms frames and enqueues them
/// back-to-back. Keep appending silence while the session is active. If a client stops sending audio
/// and later resumes, model time treats the resumed audio as contiguous with the previous audio rather
/// than as a real-world pause.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationClientEventInputAudioBufferAppend {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `session.input_audio_buffer.append`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Base64-encoded 24 kHz PCM16 mono audio bytes.
    pub audio: String,
}
