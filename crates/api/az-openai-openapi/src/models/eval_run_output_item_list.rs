// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalRunOutputItemList` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalRunOutputItem,
};

/// An object representing a list of output items for an evaluation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunOutputItemList {
    /// The type of this object. It is always set to "list".
    pub object: String,
    /// An array of eval run output item objects.
    pub data: Vec<EvalRunOutputItem>,
    /// The identifier of the first eval run output item in the data array.
    pub first_id: String,
    /// The identifier of the last eval run output item in the data array.
    pub last_id: String,
    /// Indicates whether there are more eval run output items available.
    pub has_more: bool,
}
