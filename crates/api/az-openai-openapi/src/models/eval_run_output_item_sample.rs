// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalRunOutputItemSample` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalApiError,
    EvalRunOutputItemSampleInputItem,
    EvalRunOutputItemSampleOutputItem,
    EvalRunOutputItemSampleUsage,
};

/// A sample containing the input and output of the evaluation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunOutputItemSample {
    /// An array of input messages.
    pub input: Vec<EvalRunOutputItemSampleInputItem>,
    /// An array of output messages.
    pub output: Vec<EvalRunOutputItemSampleOutputItem>,
    /// The reason why the sample generation was finished.
    pub finish_reason: String,
    /// The model used for generating the sample.
    pub model: String,
    /// Token usage details for the sample.
    pub usage: EvalRunOutputItemSampleUsage,
    pub error: EvalApiError,
    /// The sampling temperature used.
    pub temperature: f64,
    /// The maximum number of tokens allowed for completion.
    pub max_completion_tokens: i32,
    /// The top_p value used for sampling.
    pub top_p: f64,
    /// The seed used for generating the sample.
    pub seed: i32,
}
