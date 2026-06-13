// Generated from OpenAPI spec. Do not edit by hand.
//! `GraderStringCheck` DTO.

use serde::{Deserialize, Serialize};

/// A StringCheckGrader object that performs a string comparison between input and reference using a
/// specified operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraderStringCheck {
    /// The object type, which is always `string_check`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the grader.
    pub name: String,
    /// The input text. This may include template strings.
    pub input: String,
    /// The reference text. This may include template strings.
    pub reference: String,
    /// The string check operation to perform. One of `eq`, `ne`, `like`, or `ilike`.
    pub operation: String,
}
