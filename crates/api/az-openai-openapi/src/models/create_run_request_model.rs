// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateRunRequestModel` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantSupportedModels,
};

/// The ID of the [Model](/docs/api-reference/models) to be used to execute this run. If a value is
/// provided here, it will override the model associated with the assistant. If not, the model
/// associated with the assistant will be used.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateRunRequestModel {
    String(String),
    AssistantSupportedModels(AssistantSupportedModels),
}
