// Generated from OpenAPI spec. Do not edit by hand.
//! `ValidateGraderResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ValidateGraderResponseGrader,
};

/// ValidateGraderResponse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateGraderResponse {
    /// The grader used for the fine-tuning job.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grader: Option<ValidateGraderResponseGrader>,
}
