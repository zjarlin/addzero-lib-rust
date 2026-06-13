// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateVoiceConsentRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVoiceConsentRequest {
    /// The label to use for this consent recording.
    pub name: String,
    /// The consent audio recording file. Maximum size is 10 MiB. Supported MIME types: `audio/mpeg`,
    /// `audio/wav`, `audio/x-wav`, `audio/ogg`, `audio/aac`, `audio/flac`, `audio/webm`, `audio/mp4`.
    pub recording: OpenAiBinaryBody,
    /// The BCP 47 language tag for the consent phrase (for example, `en-US`).
    pub language: String,
}
