// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateVoiceRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVoiceRequest {
    /// The name of the new voice.
    pub name: String,
    /// The sample audio recording file. Maximum size is 10 MiB. Supported MIME types: `audio/mpeg`,
    /// `audio/wav`, `audio/x-wav`, `audio/ogg`, `audio/aac`, `audio/flac`, `audio/webm`, `audio/mp4`.
    pub audio_sample: OpenAiBinaryBody,
    /// The consent recording ID (for example, `cons_1234`).
    pub consent: String,
}
