// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateTranscriptionResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateTranscriptionResponseDiarizedJson,
    CreateTranscriptionResponseJson,
    CreateTranscriptionResponseVerboseJson,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateTranscriptionResponse {
    CreateTranscriptionResponseJson(CreateTranscriptionResponseJson),
    CreateTranscriptionResponseDiarizedJson(CreateTranscriptionResponseDiarizedJson),
    CreateTranscriptionResponseVerboseJson(CreateTranscriptionResponseVerboseJson),
}
