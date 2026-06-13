// Generated from OpenAPI spec. Do not edit by hand.
//! `ModifyAssistantRequestModel` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantSupportedModels,
};

/// ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see
/// all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModifyAssistantRequestModel {
    String(String),
    AssistantSupportedModels(AssistantSupportedModels),
}
