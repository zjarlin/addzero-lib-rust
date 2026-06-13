// Generated from OpenAPI spec. Do not edit by hand.
//! `ValidateGraderRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ValidateGraderRequestGrader,
};

/// ValidateGraderRequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateGraderRequest {
    /// The grader used for the fine-tuning job.
    pub grader: ValidateGraderRequestGrader,
}
