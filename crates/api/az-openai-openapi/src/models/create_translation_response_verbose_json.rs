// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateTranslationResponseVerboseJson` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TranscriptionSegment,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTranslationResponseVerboseJson {
    /// The language of the output translation (always `english`).
    pub language: String,
    /// The duration of the input audio.
    pub duration: f64,
    /// The translated text.
    pub text: String,
    /// Segments of the translated text and their corresponding details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<TranscriptionSegment>>,
}
