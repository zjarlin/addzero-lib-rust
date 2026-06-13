// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunGraderResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonValue,
};

use crate::models::{
    RunGraderResponseMetadata,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunGraderResponse {
    pub reward: f64,
    pub metadata: RunGraderResponseMetadata,
    pub sub_rewards: std::collections::BTreeMap<String, OpenAiJsonValue>,
    pub model_grader_token_usage_per_model: std::collections::BTreeMap<String, OpenAiJsonValue>,
}
