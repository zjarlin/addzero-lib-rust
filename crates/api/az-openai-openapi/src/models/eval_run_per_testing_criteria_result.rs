// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalRunPerTestingCriteriaResult` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunPerTestingCriteriaResult {
    /// A description of the testing criteria.
    pub testing_criteria: String,
    /// Number of tests passed for this criteria.
    pub passed: i32,
    /// Number of tests failed for this criteria.
    pub failed: i32,
}
