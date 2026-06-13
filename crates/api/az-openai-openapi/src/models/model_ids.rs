// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ModelIds` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ModelIdsResponses,
    ModelIdsShared,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelIds {
    ModelIdsShared(ModelIdsShared),
    ModelIdsResponses(ModelIdsResponses),
}
