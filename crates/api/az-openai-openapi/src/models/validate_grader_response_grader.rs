// Generated from OpenAPI spec. Do not edit by hand.
//! `ValidateGraderResponseGrader` DTO.

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
pub enum ValidateGraderResponseGrader {
    GraderStringCheck(GraderStringCheck),
    GraderTextSimilarity(GraderTextSimilarity),
    GraderPython(GraderPython),
    GraderScoreModel(GraderScoreModel),
    GraderMulti(GraderMulti),
}
