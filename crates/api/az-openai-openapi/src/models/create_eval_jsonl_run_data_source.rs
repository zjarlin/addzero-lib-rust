// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalJsonlRunDataSource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalJsonlRunDataSourceSource,
};

/// A JsonlRunDataSource object with that specifies a JSONL file that matches the eval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalJsonlRunDataSource {
    /// The type of data source. Always `jsonl`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Determines what populates the `item` namespace in the data source.
    pub source: CreateEvalJsonlRunDataSourceSource,
}
