// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalRunOutputItemResult` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// A single grader result for an evaluation run output item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunOutputItemResult {
    /// The name of the grader.
    pub name: String,
    /// The grader type (for example, "string-check-grader").
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The numeric score produced by the grader.
    pub score: f64,
    /// Whether the grader considered the output a pass.
    pub passed: bool,
    /// Optional sample or intermediate data produced by the grader.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample: Option<OpenAiJsonObject>,
}
