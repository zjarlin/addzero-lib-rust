// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalGraderLabelModel` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalItem,
};

/// LabelModelGrader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalGraderLabelModel {
    /// The object type, which is always `label_model`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the grader.
    pub name: String,
    /// The model to use for the evaluation. Must support structured outputs.
    pub model: String,
    pub input: Vec<EvalItem>,
    /// The labels to assign to each item in the evaluation.
    pub labels: Vec<String>,
    /// The labels that indicate a passing result. Must be a subset of labels.
    pub passing_labels: Vec<String>,
}
