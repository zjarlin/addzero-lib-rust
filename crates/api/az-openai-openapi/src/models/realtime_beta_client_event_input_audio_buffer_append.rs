// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaClientEventInputAudioBufferAppend` DTO.

use serde::{Deserialize, Serialize};

/// Send this event to append audio bytes to the input audio buffer. The audio buffer is temporary
/// storage you can write to and later commit. In Server VAD mode, the audio buffer is used to detect
/// speech and the server will decide when to commit. When Server VAD is disabled, you must commit the
/// audio buffer manually. The client may choose how much audio to place in each event up to a maximum
/// of 15 MiB, for example streaming smaller chunks from the client may allow the VAD to be more
/// responsive. Unlike made other client events, the server will not send a confirmation response to
/// this event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaClientEventInputAudioBufferAppend {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `input_audio_buffer.append`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Base64-encoded audio bytes. This must be in the format specified by the `input_audio_format` field
    /// in the session configuration.
    pub audio: String,
}
