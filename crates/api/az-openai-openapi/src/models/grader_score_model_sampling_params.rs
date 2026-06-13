// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `GraderScoreModelSamplingParams` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ReasoningEffort,
};

/// The sampling parameters for the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraderScoreModelSamplingParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completions_tokens: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
}
