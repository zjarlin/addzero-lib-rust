// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateModerationRequestInput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateModerationRequestInputArray3Item3,
};

/// Input (or inputs) to classify. Can be a single string, an array of strings, or an array of multi-
/// modal input objects similar to other models.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateModerationRequestInput {
    String(String),
    Array(Vec<String>),
    Array3(Vec<CreateModerationRequestInputArray3Item3>),
}
