// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FunctionCallOutputItemParamOutput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FunctionCallOutputItemParamOutputArrayItem,
};

/// Text, image, or file output of the function tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCallOutputItemParamOutput {
    String(String),
    Array(Vec<FunctionCallOutputItemParamOutputArrayItem>),
}
