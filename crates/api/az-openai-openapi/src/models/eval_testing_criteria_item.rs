// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalTestingCriteriaItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalGraderLabelModel,
    EvalGraderPython,
    EvalGraderScoreModel,
    EvalGraderStringCheck,
    EvalGraderTextSimilarity,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EvalTestingCriteriaItem {
    EvalGraderLabelModel(EvalGraderLabelModel),
    EvalGraderStringCheck(EvalGraderStringCheck),
    EvalGraderTextSimilarity(EvalGraderTextSimilarity),
    EvalGraderPython(EvalGraderPython),
    EvalGraderScoreModel(EvalGraderScoreModel),
}
