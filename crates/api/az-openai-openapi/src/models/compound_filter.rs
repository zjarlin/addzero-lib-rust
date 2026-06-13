// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CompoundFilter` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CompoundFilterFilter,
};

/// Combine multiple filters using `and` or `or`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundFilter {
    /// Type of operation: `and` or `or`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Array of filters to combine. Items can be `ComparisonFilter` or `CompoundFilter`.
    pub filters: Vec<CompoundFilterFilter>,
}
