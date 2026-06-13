// Generated from OpenAPI spec. Do not edit by hand.
//! `GraderPython` DTO.

use serde::{Deserialize, Serialize};

/// A PythonGrader object that runs a python script on the input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraderPython {
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
}
