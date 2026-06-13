// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalRunList` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalRun,
};

/// An object representing a list of runs for an evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunList {
    /// The type of this object. It is always set to "list".
    pub object: String,
    /// An array of eval run objects.
    pub data: Vec<EvalRun>,
    /// The identifier of the first eval run in the data array.
    pub first_id: String,
    /// The identifier of the last eval run in the data array.
    pub last_id: String,
    /// Indicates whether there are more evals available.
    pub has_more: bool,
}
