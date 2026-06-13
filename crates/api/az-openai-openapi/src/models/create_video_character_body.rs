// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateVideoCharacterBody` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

/// Parameters for creating a character from an uploaded video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoCharacterBody {
    /// Video file used to create a character.
    pub video: OpenAiBinaryBody,
    /// Display name for this API character.
    pub name: String,
}
