// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalRunOutputItem` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

use crate::models::{
    EvalRunOutputItemResult,
    EvalRunOutputItemSample,
};

/// A schema representing an evaluation run output item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunOutputItem {
    /// The type of the object. Always "eval.run.output_item".
    pub object: String,
    /// Unique identifier for the evaluation run output item.
    pub id: String,
    /// The identifier of the evaluation run associated with this output item.
    pub run_id: String,
    /// The identifier of the evaluation group.
    pub eval_id: String,
    /// Unix timestamp (in seconds) when the evaluation run was created.
    pub created_at: i64,
    /// The status of the evaluation run.
    pub status: String,
    /// The identifier for the data source item.
    pub datasource_item_id: i32,
    /// Details of the input data source item.
    pub datasource_item: OpenAiJsonObject,
    /// A list of grader results for this output item.
    pub results: Vec<EvalRunOutputItemResult>,
    /// A sample containing the input and output of the evaluation run.
    pub sample: EvalRunOutputItemSample,
}
