// Generated from OpenAPI spec. Do not edit by hand.
//! `ModelIdsCompaction` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ModelIdsResponses,
};

/// Model ID used to generate the response, like `gpt-5` or `o3`. OpenAI offers a wide range of models
/// with different capabilities, performance characteristics, and price points. Refer to the [model
/// guide](/docs/models) to browse and compare available models.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelIdsCompaction {
    ModelIdsResponses(ModelIdsResponses),
    String(String),
}
