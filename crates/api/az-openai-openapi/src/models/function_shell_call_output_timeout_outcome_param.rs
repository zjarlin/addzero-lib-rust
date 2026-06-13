// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionShellCallOutputTimeoutOutcomeParam` DTO.

use serde::{Deserialize, Serialize};

/// Indicates that the shell call exceeded its configured time limit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionShellCallOutputTimeoutOutcomeParam {
    /// The outcome type. Always `timeout`.
    #[serde(rename = "type")]
    pub type_value: String,
}
