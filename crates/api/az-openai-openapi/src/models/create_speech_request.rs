// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateSpeechRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VoiceIdsOrCustomVoice,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpeechRequest {
    /// One of the available [TTS models](/docs/models#tts): `tts-1`, `tts-1-hd`, `gpt-4o-mini-tts`, or
    /// `gpt-4o-mini-tts-2025-12-15`.
    pub model: String,
    /// The text to generate audio for. The maximum length is 4096 characters.
    pub input: String,
    /// Control the voice of your generated audio with additional instructions. Does not work with `tts-1`
    /// or `tts-1-hd`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// The voice to use when generating the audio. Supported built-in voices are `alloy`, `ash`, `ballad`,
    /// `coral`, `echo`, `fable`, `onyx`, `nova`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. You may
    /// also provide a custom voice object with an `id`, for example `{ "id": "voice_1234" }`. Previews of
    /// the voices are available in the [Text to speech guide](/docs/guides/text-to-speech#voice-options).
    pub voice: VoiceIdsOrCustomVoice,
    /// The format to audio in. Supported formats are `mp3`, `opus`, `aac`, `flac`, `wav`, and `pcm`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// The speed of the generated audio. Select a value from `0.25` to `4.0`. `1.0` is the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    /// The format to stream the audio in. Supported formats are `sse` and `audio`. `sse` is not supported
    /// for `tts-1` or `tts-1-hd`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_format: Option<String>,
}
