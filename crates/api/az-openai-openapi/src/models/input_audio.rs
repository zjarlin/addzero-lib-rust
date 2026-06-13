// Generated from OpenAPI spec. Do not edit by hand.
//! `InputAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputAudioInputAudio,
};

/// An audio input to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudio {
    /// The type of the input item. Always `input_audio`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub input_audio: InputAudioInputAudio,
}
