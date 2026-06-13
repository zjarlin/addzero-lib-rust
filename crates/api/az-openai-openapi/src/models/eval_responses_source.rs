// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalResponsesSource` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

use crate::models::{
    ReasoningEffort,
};

/// A EvalResponsesSource object describing a run data source configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResponsesSource {
    /// The type of run data source. Always `responses`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OpenAiJsonObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions_search: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_after: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_before: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
}
