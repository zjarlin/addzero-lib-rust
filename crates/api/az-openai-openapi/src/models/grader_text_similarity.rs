// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `GraderTextSimilarity` DTO.

use serde::{Deserialize, Serialize};

/// A TextSimilarityGrader object which grades text based on similarity metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraderTextSimilarity {
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
}
