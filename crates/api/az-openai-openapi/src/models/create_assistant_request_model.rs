// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateAssistantRequestModel` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantSupportedModels,
};

/// ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see
/// all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateAssistantRequestModel {
    String(String),
    AssistantSupportedModels(AssistantSupportedModels),
}
