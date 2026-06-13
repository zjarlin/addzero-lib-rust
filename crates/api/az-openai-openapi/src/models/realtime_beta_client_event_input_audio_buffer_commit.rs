// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaClientEventInputAudioBufferCommit` DTO.

use serde::{Deserialize, Serialize};

/// Send this event to commit the user input audio buffer, which will create a new user message item in
/// the conversation. This event will produce an error if the input audio buffer is empty. When in
/// Server VAD mode, the client does not need to send this event, the server will commit the audio
/// buffer automatically. Committing the input audio buffer will trigger input audio transcription (if
/// enabled in session configuration), but it will not create a response from the model. The server will
/// respond with an `input_audio_buffer.committed` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaClientEventInputAudioBufferCommit {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `input_audio_buffer.commit`.
    #[serde(rename = "type")]
    pub type_value: String,
}
