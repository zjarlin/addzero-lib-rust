// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateTranslationRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTranslationRequest {
    /// The audio file object (not file name) translate, in one of these formats: flac, mp3, mp4, mpeg,
    /// mpga, m4a, ogg, wav, or webm.
    pub file: OpenAiBinaryBody,
    /// ID of the model to use. Only `whisper-1` (which is powered by our open source Whisper V2 model) is
    /// currently available.
    pub model: String,
    /// An optional text to guide the model's style or continue a previous audio segment. The
    /// [prompt](/docs/guides/speech-to-text#prompting) should be in English.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// The format of the output, in one of these options: `json`, `text`, `srt`, `verbose_json`, or `vtt`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// The sampling temperature, between 0 and 1. Higher values like 0.8 will make the output more random,
    /// while lower values like 0.2 will make it more focused and deterministic. If set to 0, the model will
    /// use [log probability](https://en.wikipedia.org/wiki/Log_probability) to automatically increase the
    /// temperature until certain thresholds are hit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
}
