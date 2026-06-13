// Generated from OpenAPI spec. Do not edit by hand.
//! `GraderMultiGraders` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    GraderLabelModel,
    GraderPython,
    GraderScoreModel,
    GraderStringCheck,
    GraderTextSimilarity,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GraderMultiGraders {
    GraderStringCheck(GraderStringCheck),
    GraderTextSimilarity(GraderTextSimilarity),
    GraderPython(GraderPython),
    GraderScoreModel(GraderScoreModel),
    GraderLabelModel(GraderLabelModel),
}
