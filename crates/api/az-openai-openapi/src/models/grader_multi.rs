// Generated from OpenAPI spec. Do not edit by hand.
//! `GraderMulti` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    GraderMultiGraders,
};

/// A MultiGrader object combines the output of multiple graders to produce a single score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraderMulti {
    /// The object type, which is always `multi`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the grader.
    pub name: String,
    pub graders: GraderMultiGraders,
    /// A formula to calculate the output based on grader results.
    pub calculate_output: String,
}
