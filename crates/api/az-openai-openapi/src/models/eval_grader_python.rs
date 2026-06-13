// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalGraderPython` DTO.

use serde::{Deserialize, Serialize};

/// PythonGrader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalGraderPython {
    /// The object type, which is always `python`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the grader.
    pub name: String,
    /// The source code of the python script.
    pub source: String,
    /// The image tag to use for the python script.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_tag: Option<String>,
    /// The threshold for the score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass_threshold: Option<f64>,
}
