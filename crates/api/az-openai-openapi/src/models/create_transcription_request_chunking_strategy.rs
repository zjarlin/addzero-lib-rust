// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateTranscriptionRequestChunkingStrategy` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VadConfig,
};

/// Controls how the audio is cut into chunks. When set to `"auto"`, the server first normalizes
/// loudness and then uses voice activity detection (VAD) to choose boundaries. `server_vad` object can
/// be provided to tweak VAD detection parameters manually. If unset, the audio is transcribed as a
/// single block. Required when using `gpt-4o-transcribe-diarize` for inputs longer than 30 seconds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateTranscriptionRequestChunkingStrategy {
    Auto(String),
    VadConfig(VadConfig),
}
