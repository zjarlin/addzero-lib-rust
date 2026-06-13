// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalList` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Eval,
};

/// An object representing a list of evals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalList {
    /// The type of this object. It is always set to "list".
    pub object: String,
    /// An array of eval objects.
    pub data: Vec<Eval>,
    /// The identifier of the first eval in the data array.
    pub first_id: String,
    /// The identifier of the last eval in the data array.
    pub last_id: String,
    /// Indicates whether there are more evals available.
    pub has_more: bool,
}
