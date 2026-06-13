// Generated from OpenAPI spec. Do not edit by hand.
//! `RunGraderResponseMetadata` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonValue,
};

use crate::models::{
    RunGraderResponseMetadataErrors,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunGraderResponseMetadata {
    pub name: String,
    #[serde(rename = "type")]
    pub type_value: String,
    pub errors: RunGraderResponseMetadataErrors,
    pub execution_time: f64,
    pub scores: std::collections::BTreeMap<String, OpenAiJsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampled_model_name: Option<String>,
}
