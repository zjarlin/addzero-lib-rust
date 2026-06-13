// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateModerationRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateModerationRequestInput,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationRequest {
    /// Input (or inputs) to classify. Can be a single string, an array of strings, or an array of multi-
    /// modal input objects similar to other models.
    pub input: CreateModerationRequestInput,
    /// The content moderation model you would like to use. Learn more in [the moderation
    /// guide](/docs/guides/moderation), and learn about available models [here](/docs/models#moderation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}
