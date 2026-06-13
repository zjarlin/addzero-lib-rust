// Generated from OpenAPI spec. Do not edit by hand.
//! `ValidateGraderRequestGrader` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    GraderMulti,
    GraderPython,
    GraderScoreModel,
    GraderStringCheck,
    GraderTextSimilarity,
};

/// The grader used for the fine-tuning job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValidateGraderRequestGrader {
    GraderStringCheck(GraderStringCheck),
    GraderTextSimilarity(GraderTextSimilarity),
    GraderPython(GraderPython),
    GraderScoreModel(GraderScoreModel),
    GraderMulti(GraderMulti),
}
