// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalGraderTextSimilarity` DTO.

use serde::{Deserialize, Serialize};

/// TextSimilarityGrader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalGraderTextSimilarity {
    /// The type of grader.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the grader.
    pub name: String,
    /// The text being graded.
    pub input: String,
    /// The text being graded against.
    pub reference: String,
    /// The evaluation metric to use. One of `cosine`, `fuzzy_match`, `bleu`, `gleu`, `meteor`, `rouge_1`,
    /// `rouge_2`, `rouge_3`, `rouge_4`, `rouge_5`, or `rouge_l`.
    pub evaluation_metric: String,
    /// The threshold for the score.
    pub pass_threshold: f64,
}
