// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `TranscriptionChunkingStrategy` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// Controls how the audio is cut into chunks. When set to `"auto"`, the server first normalizes
/// loudness and then uses voice activity detection (VAD) to choose boundaries. `server_vad` object can
/// be provided to tweak VAD detection parameters manually. If unset, the audio is transcribed as a
/// single block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionChunkingStrategy {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
