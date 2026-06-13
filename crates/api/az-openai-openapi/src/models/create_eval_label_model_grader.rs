// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalLabelModelGrader` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalItem,
};

/// A LabelModelGrader object which uses a model to assign labels to each item in the evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalLabelModelGrader {
    /// The object type, which is always `label_model`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the grader.
    pub name: String,
    /// The model to use for the evaluation. Must support structured outputs.
    pub model: String,
    /// A list of chat messages forming the prompt or context. May include variable references to the `item`
    /// namespace, ie {{item.name}}.
    pub input: Vec<CreateEvalItem>,
    /// The labels to classify to each item in the evaluation.
    pub labels: Vec<String>,
    /// The labels that indicate a passing result. Must be a subset of labels.
    pub passing_labels: Vec<String>,
}
