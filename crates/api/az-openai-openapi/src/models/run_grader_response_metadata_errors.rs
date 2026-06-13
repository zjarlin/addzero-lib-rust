// Generated from OpenAPI spec. Do not edit by hand.
//! `RunGraderResponseMetadataErrors` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunGraderResponseMetadataErrors {
    pub formula_parse_error: bool,
    pub sample_parse_error: bool,
    pub truncated_observation_error: bool,
    pub unresponsive_reward_error: bool,
    pub invalid_variable_error: bool,
    pub other_error: bool,
    pub python_grader_server_error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python_grader_server_error_type: Option<String>,
    pub python_grader_runtime_error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python_grader_runtime_error_details: Option<String>,
    pub model_grader_server_error: bool,
    pub model_grader_refusal_error: bool,
    pub model_grader_parse_error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_grader_server_error_details: Option<String>,
}
