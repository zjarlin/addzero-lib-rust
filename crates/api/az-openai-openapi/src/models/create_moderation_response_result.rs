// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateModerationResponseResult` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateModerationResponseResultCategories,
    CreateModerationResponseResultCategoryAppliedInputTypes,
    CreateModerationResponseResultCategoryScores,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationResponseResult {
    /// Whether any of the below categories are flagged.
    pub flagged: bool,
    /// A list of the categories, and whether they are flagged or not.
    pub categories: CreateModerationResponseResultCategories,
    /// A list of the categories along with their scores as predicted by model.
    pub category_scores: CreateModerationResponseResultCategoryScores,
    /// A list of the categories along with the input type(s) that the score applies to.
    pub category_applied_input_types: CreateModerationResponseResultCategoryAppliedInputTypes,
}
