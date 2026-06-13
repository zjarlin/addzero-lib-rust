// Generated from OpenAPI spec. Do not edit by hand.
//! `ComparisonFilter` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ComparisonFilterValue,
};

/// A filter used to compare a specified attribute key to a given value using a defined comparison
/// operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonFilter {
    /// Specifies the comparison operator: `eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `in`, `nin`. - `eq`: equals
    /// - `ne`: not equal - `gt`: greater than - `gte`: greater than or equal - `lt`: less than - `lte`:
    /// less than or equal - `in`: in - `nin`: not in
    #[serde(rename = "type")]
    pub type_value: String,
    /// The key to compare against the value.
    pub key: String,
    /// The value to compare against the attribute key; supports string, number, or boolean types.
    pub value: ComparisonFilterValue,
}
