// Generated from OpenAPI spec. Do not edit by hand.
//! `InputAudioInputAudio` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudioInputAudio {
    /// Base64-encoded audio data.
    pub data: String,
    /// The format of the audio data. Currently supported formats are `mp3` and `wav`.
    pub format: String,
}
