// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ModelIdsResponses` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ModelIdsShared,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelIdsResponses {
    ModelIdsShared(ModelIdsShared),
    ResponsesOnlyModel(String),
}
