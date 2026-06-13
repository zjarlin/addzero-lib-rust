// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalRequestTestingCriteriaItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalLabelModelGrader,
    EvalGraderPython,
    EvalGraderScoreModel,
    EvalGraderStringCheck,
    EvalGraderTextSimilarity,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalRequestTestingCriteriaItem {
    CreateEvalLabelModelGrader(CreateEvalLabelModelGrader),
    EvalGraderStringCheck(EvalGraderStringCheck),
    EvalGraderTextSimilarity(EvalGraderTextSimilarity),
    EvalGraderPython(EvalGraderPython),
    EvalGraderScoreModel(EvalGraderScoreModel),
}
