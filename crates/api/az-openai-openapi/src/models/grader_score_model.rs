// Generated from OpenAPI spec. Do not edit by hand.
//! `GraderScoreModel` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalItem,
    GraderScoreModelSamplingParams,
};

/// A ScoreModelGrader object that uses a model to assign a score to the input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraderScoreModel {
    /// The object type, which is always `score_model`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the grader.
    pub name: String,
    /// The model to use for the evaluation.
    pub model: String,
    /// The sampling parameters for the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling_params: Option<GraderScoreModelSamplingParams>,
    /// The input messages evaluated by the grader. Supports text, output text, input image, and input audio
    /// content blocks, and may include template strings.
    pub input: Vec<EvalItem>,
    /// The range of the score. Defaults to `[0, 1]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<Vec<f64>>,
}
